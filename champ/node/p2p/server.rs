#![allow(dead_code, unused_variables)]

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::p2p::metrics;
use crate::p2p::types::Peer;
use crate::state::ChampStateArc;
use crate::wallets::{Wallet, WalletPrivateKey};
use anyhow::{anyhow, Result};
use crypto::rand::seq::IteratorRandom;
use crypto::signatures::ed25519::verify_signature;
use dashmap::DashMap;
use libp2p::core::ConnectedPoint;
use libp2p::dns::TokioDnsConfig;
use libp2p::identity::{self, ed25519};
use libp2p::Multiaddr;

use pog_proto::Message;

use libp2p::{
    core::{upgrade, Transport},
    futures::StreamExt,
    noise::{self},
    request_response::{RequestId, RequestResponseEvent, ResponseChannel},
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    yamux::YamuxConfig,
    {PeerId, Swarm},
};

use super::methods;
use super::protocol::{PogBehavior, PogMessage, PogRequest, PogResponse};
use super::types::{protocol, Event, Failure, NodeKeypair, Peers};
use super::types::{request_body, RequestBody, RequestBodyData, RequestHeader};
use super::types::{ResponseBody, ResponseBodyData, ResponseHeader};

const NR_OF_PEERS_SENT: usize = 10;

pub fn timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as u64
}

pub struct P2PServer {
    pub peers: Peers,
    pub state: ChampStateArc,
    pub node_wallet: Wallet,
    swarm: Swarm<PogBehavior>,
    keypair: NodeKeypair,
}

impl P2PServer {
    pub async fn new(state: ChampStateArc) -> Result<Self> {
        let node_wallet = {
            let wallet_manager = state.wallet_manager.read().await;
            let wallet = wallet_manager.primary_wallet().await.ok_or_else(|| anyhow!("no primary wallet found"))?;
            wallet.clone()
        };

        let id_keys = {
            let secret_key: ed25519::SecretKey =
                ed25519::SecretKey::from_bytes(node_wallet.private_key().expect("node wallet needs to be unlocked"))
                    .unwrap();
            let keypair = ed25519::Keypair::from(secret_key);
            identity::Keypair::Ed25519(keypair)
        };
        let dh_keys = noise::Keypair::<noise::X25519Spec>::new().into_authentic(&id_keys).unwrap();

        // unique id of this machine for libp2p
        let peer_id = PeerId::from(id_keys.public());
        // custom protobuf/TLV based request-response protocol
        let pog_protocol = protocol::PogProtocol::new();

        // encryption protocol similar to tls
        let noise = noise::NoiseConfig::xx(dh_keys.clone()).into_authenticated();
        // create transport configuration using TokioTCP, authenticating it using noise
        // and adds multiplexing to allow multiple protocols
        let transp = TokioDnsConfig::system(TokioTcpConfig::new())?
            .upgrade(upgrade::Version::V1)
            .authenticate(noise)
            .multiplex(YamuxConfig::default())
            .timeout(Duration::from_secs(10))
            .boxed();

        // instantiates the libp2p transport layer using a swarm, our protocol and our id
        let swarm = SwarmBuilder::new(transp, pog_protocol.behavior(), peer_id)
            .executor(Box::new(|f| {
                tokio::task::spawn(f);
            }))
            .build();

        // concurrent hashmap to store all connected peers
        let peers = Arc::new(DashMap::new());

        Ok(Self {
            state,
            swarm,
            keypair: dh_keys,
            peers,
            node_wallet,
        })
    }

    /// Dials the initial peers defined in the config
    async fn connect_to_initial_peers(&mut self) {
        let peers = &self.state.config.read().await.consensus.initial_peers;
        for peer in peers {
            if let Ok(addr) = peer.parse::<Multiaddr>() {
                let peer_ = self.swarm.dial(addr);
            }
        }
        tracing::debug!("{peers:?}");
    }

    /// Randomly selects a number of online peers
    fn get_random_peer_ids(&mut self, max_nr_of_peers: usize) -> Vec<PeerId> {
        let mut r = crypto::rand::thread_rng();
        self.peers.iter().choose_multiple(&mut r, max_nr_of_peers).iter().map(|p| p.id).collect()
    }

    /// Sends a ping to a peer with payload of random peers
    fn send_ping(&mut self, peer_id: PeerId) -> Result<()> {
        let peers = self.get_random_peer_ids(10).iter().map(|p| p.to_bytes()).collect();

        self.send_request(
            &peer_id,
            RequestBodyData::Ping(request_body::Ping {
                peers,
            }),
        )
        .map(|_| ())
    }

    /// Event handeler for libp2p swarm events
    async fn handle_event(&mut self, event: Event) {
        match event {
            SwarmEvent::Behaviour(RequestResponseEvent::Message {
                peer,
                message,
            }) => self.process_message(peer, message).await,
            SwarmEvent::Behaviour(RequestResponseEvent::ResponseSent {
                ..
            }) => {}
            SwarmEvent::Behaviour(RequestResponseEvent::InboundFailure {
                peer,
                request_id,
                error,
            }) => tracing::error!("inbound message failure: {peer:?}: {error:?}: {request_id}"),
            SwarmEvent::Behaviour(RequestResponseEvent::OutboundFailure {
                peer,
                request_id,
                error,
            }) => tracing::error!("outbound message failure: {peer:?}: {error:?}: {request_id}"),
            SwarmEvent::ConnectionEstablished {
                peer_id,
                endpoint,
                num_established,
                ..
            } => self.handle_new_connection(peer_id, endpoint, num_established),
            other => {
                tracing::debug!("Unhandled {:?}", other);
            }
        }
    }

    /// Adds new connection to our HashMap and pings the peer
    fn handle_new_connection(&mut self, peer_id: PeerId, endpoint: ConnectedPoint, num_established: NonZeroU32) {
        tracing::debug!("connection established: {peer_id}");
        let addr = endpoint.get_remote_address();

        if !self.peers.contains_key(&peer_id) {
            self.peers.insert(
                peer_id,
                Peer {
                    id: peer_id,
                    ip: addr.clone(),
                    last_ping: None,
                    voting_power: None,
                },
            );
        }

        tracing::debug!("sending initial ping to {peer_id}");
        if let Err(e) = self.send_ping(peer_id) {
            tracing::error!("ping failed: {e}")
        }
    }

    /// Ping peers every tick and update metric server
    fn handle_tick(&mut self) {
        metrics::update(self.peers.clone());
        tracing::debug!("pinging all peers");
        let peers: Vec<PeerId> = self.peers.iter().map(|p| p.id).collect();
        for peer in peers {
            if let Err(e) = self.send_ping(peer) {
                tracing::error!("ping failed: {e}");
            }
        }
    }

    /// Runs the server
    pub async fn start(&mut self) -> Result<()> {
        // Binds the port to the Socket and starts listening
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/50052".parse()?)?;
        self.connect_to_initial_peers().await;
        metrics::update(self.peers.clone());

        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                    event = self.swarm.select_next_some() => self.handle_event(event).await,
                    _ = interval.tick() => self.handle_tick(),
            }
        }
    }

    /// Splits messages as Request or Response message
    async fn process_message(&mut self, peer: PeerId, message: PogMessage) {
        let res = match message {
            protocol::RequestMessage {
                channel,
                request,
                request_id,
            } => self.process_request(channel, request, request_id, peer).await,
            protocol::ResponseMessage {
                request_id,
                response,
            } => self.process_response(request_id, response, peer),
        };

        if let Err(err) = res {
            tracing::error!("error while processing request for peer {peer}: {err}");
        }
    }

    async fn process_request(
        &mut self,
        channel: ResponseChannel<PogResponse>,
        request: PogRequest,
        request_id: RequestId,
        peer_id: PeerId,
    ) -> Result<()> {
        tracing::debug!("processing request from {peer_id}");

        let header = match RequestHeader::decode(&*request.header) {
            Ok(header) => header,
            Err(err) => {
                self.send_response(channel, ResponseBodyData::Failure(Failure::MalformedRequest.into()))?;
                return Err(err.into());
            }
        };

        if let Err(e) = verify_signature(&request.data, &*header.public_key, &*header.signature) {
            self.send_response(channel, ResponseBodyData::Failure(Failure::MalformedRequest.into()))?;
            return Err(e.into());
        }

        let body = match RequestBody::decode(&*request.data) {
            Ok(body) => body,
            Err(err) => {
                self.send_response(channel, ResponseBodyData::Failure(Failure::MalformedRequest.into()))?;
                return Err(err.into());
            }
        };

        let data = match body.data {
            Some(d) => d,
            None => {
                self.send_response(channel, ResponseBodyData::Failure(Failure::MalformedRequest.into()))?;
                return Err(anyhow!("data was none"));
            }
        };

        tracing::trace!("got a request: {data:?}");

        let result = match data {
            request_body::Data::FinalVote(data) => methods::process_final_vote(self, data, peer_id),
            request_body::Data::VoteProposal(data) => methods::process_vote_proposal(self, data, peer_id).await,
            request_body::Data::Forward(data) => methods::process_forward(self, *data, peer_id),
            request_body::Data::Ping(data) => return methods::process_ping(self, data, channel, peer_id),
        };

        match result {
            Err(e) => {
                tracing::error!("error while processing request: {e}");
                self.send_response(channel, ResponseBodyData::Failure(Failure::MalformedRequest.into()))
            }
            Ok(()) => {
                self.send_response(channel, ResponseBodyData::Success(pog_proto::p2p::response_body::Success {}))
            }
        }
    }

    fn process_response(&self, request_id: RequestId, response: PogResponse, peer_id: PeerId) -> Result<()> {
        tracing::debug!("processing response from {peer_id}");
        Ok(())
    }

    fn get_prime_delegates(&self) -> Vec<PeerId> {
        todo!("write a fn that gets all online prime delegates")
    }
}

pub trait RequestResponse {
    fn standard_send(&mut self, request: RequestBodyData) -> Result<()>;
    fn send_request(&mut self, peer: &PeerId, request: RequestBodyData) -> Result<RequestId>;
    fn send_response(&mut self, channel: ResponseChannel<PogResponse>, response: ResponseBodyData) -> Result<()>;
}

impl RequestResponse for P2PServer {
    // standard send means that a request is sent to all Prime Delegates + 10 random non Prime Delegates
    fn standard_send(&mut self, request: RequestBodyData) -> Result<()> {
        let prime_delegates = self.get_prime_delegates();
        let random_peers = self.get_random_peer_ids(NR_OF_PEERS_SENT);
        let all_peers = prime_delegates.iter().chain(random_peers.iter()).collect::<Vec<&PeerId>>();

        for peers in all_peers {
            let _ = self.send_request(peers, request.to_owned());
        }

        Ok(())
    }

    fn send_request(&mut self, peer: &PeerId, request: RequestBodyData) -> Result<RequestId> {
        let request_body = RequestBody {
            data: Some(request),
            signature_type: 0,
            timestamp: timestamp(),
        }
        .encode_to_vec();

        let header = RequestHeader {
            signature: self.node_wallet.sign(&request_body)?.to_vec(),
            public_key: self.node_wallet.public_key()?.to_vec(),
        };

        let request = PogRequest {
            data: request_body,
            header: header.encode_to_vec(),
        };

        Ok(self.swarm.behaviour_mut().send_request(peer, request))
    }

    fn send_response(&mut self, channel: ResponseChannel<PogResponse>, response: ResponseBodyData) -> Result<()> {
        let response_body = ResponseBody {
            timestamp: timestamp(),
            signature_type: 0,
            data: Some(response),
        }
        .encode_to_vec();

        let header = ResponseHeader {
            signature: self.node_wallet.sign(&response_body)?.to_vec(),
            public_key: self.node_wallet.public_key()?.to_vec(),
        };

        let response = PogResponse {
            data: response_body,
            header: header.encode_to_vec(),
        };

        self.swarm.behaviour_mut().send_response(channel, response).map_err(|_| anyhow!("response failed"))
    }
}
