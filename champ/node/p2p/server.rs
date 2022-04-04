#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::consensus::voting_power;
use crate::p2p::pogprotocol::{self, PogProtocol};
use crate::state::ChampStateArc;
use crate::wallets::Wallet;
use anyhow::{anyhow, Result};
use crypto::rand::seq::IteratorRandom;
use crypto::signatures::ed25519::{create_signature, verify_signature};
use libp2p::noise::AuthenticKeypair;
use pog_proto::p2p::request_body::{FinalVote, Forward, VoteProposal};
use pog_proto::p2p::{request_body, response_body, Failure};
use pog_proto::p2p::{
    request_body::Data as RequestBodyData, response_body::Data as ResponseBodyData, RequestBody, RequestHeader,
    ResponseBody, ResponseHeader,
};
use pog_proto::Message;

use libp2p::{
    core::{identity, upgrade, Transport},
    futures::StreamExt,
    mplex::MplexConfig,
    noise::{Keypair, NoiseConfig, X25519Spec},
    request_response::{RequestId, RequestResponseEvent, ResponseChannel},
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    {PeerId, Swarm},
};

use super::pogprotocol::{PogBehavior, PogMessage, PogRequest, PogResponse};

#[derive(Clone)]
pub struct Peer {
    voting_power: u64,
    ip: libp2p::Multiaddr,
    last_ping: std::time::Duration,
    id: PeerId,
}

pub struct P2PServer {
    state: ChampStateArc,
    swarm: Swarm<PogBehavior>,
    keypair: AuthenticKeypair<X25519Spec>,
    peers: HashMap<[u8; 32], Peer>,
    primary_wallet: Wallet,
}

const NR_OF_PEERS_SENT: usize = 10;

fn timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as u64
}

fn get_random_peers_id(peers: HashMap<[u8; 32], Peer>, max_nr_of_peers: usize) -> Vec<PeerId> {
    let mut r = crypto::rand::thread_rng();
    peers.values().choose_multiple(&mut r, max_nr_of_peers).iter().map(|p| p.id).collect::<Vec<PeerId>>()
}

impl P2PServer {
    pub fn new(state: ChampStateArc) -> Result<Self> {
        let id_keys = identity::Keypair::generate_ed25519();
        let dh_keys = Keypair::<X25519Spec>::new().into_authentic(&id_keys).unwrap();
        let peer_id = PeerId::from(id_keys.public());
        let pog_protocol = PogProtocol::new();

        let noise = NoiseConfig::xx(dh_keys.clone()).into_authenticated();
        let transp = TokioTcpConfig::new()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise)
            .multiplex(MplexConfig::new())
            .boxed();

        let swarm = SwarmBuilder::new(transp, pog_protocol.behavior(), peer_id).build();
        // TODO: create logic to save these in case of crash
        let peers = HashMap::new();

        let primary_wallet = P2PServer::get_primary_wallet(&state)?;

        Ok(Self {
            state,
            swarm,
            keypair: dh_keys,
            peers,
            primary_wallet,
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Dialing(peer_id) => {
                    println!("dialing {peer_id}")
                }
                SwarmEvent::ConnectionEstablished {
                    peer_id,
                    ..
                } => {
                    println!("connection established: {peer_id}")
                }
                SwarmEvent::Behaviour(event) => match event {
                    RequestResponseEvent::Message {
                        peer,
                        message,
                    } => self.process_message(peer, message),
                    RequestResponseEvent::ResponseSent {
                        ..
                    } => {}
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn process_message(&mut self, peer: PeerId, message: PogMessage) {
        let res = match message {
            pogprotocol::RequestMessage {
                channel,
                request,
                request_id,
            } => self.process_request(channel, request, request_id),
            pogprotocol::ResponseMessage {
                request_id,
                response,
            } => self.process_response(request_id, response),
        };

        if let Err(err) = res {
            tracing::error!("error while processing request for peer {peer}: {err}");
        }
    }

    fn process_request(
        &mut self,
        channel: ResponseChannel<PogResponse>,
        request: PogRequest,
        request_id: RequestId,
    ) -> Result<()> {
        let header = match RequestHeader::decode(&*request.header) {
            Ok(header) => header,
            Err(err) => {
                self.send_response(channel, ResponseBodyData::Failure(Failure::MalformedRequest.into()))?;
                return Err(err.into());
            }
        };

        match verify_signature(&request.data, &self.public_key(), &*header.signature) {
            Ok(_) => (),
            Err(err) => {
                self.send_response(channel, ResponseBodyData::Failure(Failure::MalformedRequest.into()))?;
                return Err(err.into());
            }
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

        let result = match data {
            request_body::Data::Forward(data) => self.process_forward(*data),
            request_body::Data::FinalVote(data) => self.process_final_vote(data),
            request_body::Data::VoteProposal(data) => self.process_vote_proposal(data),
            request_body::Data::Ping(data) => return self.process_ping(data, channel),
        };

        match result {
            Ok(_) => self.send_response(channel, ResponseBodyData::Success(pog_proto::p2p::response_body::Success {})),
            Err(err) => self.send_response(channel, ResponseBodyData::Failure(Failure::MalformedRequest.into())),
        }
    }

    fn process_forward(&self, body: Forward) -> Result<()> {
        todo!("do smth")
    }

    #[tokio::main]
    async fn process_final_vote(&mut self, data: FinalVote) -> Result<()> {
        // all nodes who calculate a 60% quorum from the vote proposal need to send a final vote
        let raw_block = match data.block {
            Some(block) => block,
            None => return Err(anyhow!("block was none")),
        };

        if self.state.blockpool_client.process_block(raw_block.clone(), data.vote).await.is_err() {
            return Err(anyhow!("error during processing vote"));
        }

        let data = request_body::FinalVote {
            block: Some(raw_block),
            vote: voting_power::get_active_power(&self.state, self.primary_wallet.account_address_bytes).await?,
        };

        self.standard_send(RequestBodyData::FinalVote(data))
    }

    #[tokio::main]
    async fn process_vote_proposal(&mut self, data: VoteProposal) -> Result<()> {
        // if prime delegate: cast own vote and send to all other prime delegates and 10 non prime delegates
        let raw_block = match data.block {
            Some(block) => block,
            None => return Err(anyhow!("block was none")),
        };

        if self.state.blockpool_client.process_vote(raw_block.clone(), data.vote).await.is_err() {
            return Err(anyhow!("error during processing vote"));
        }

        let data = request_body::VoteProposal {
            block: Some(raw_block),
            vote: voting_power::get_active_power(&self.state, self.primary_wallet.account_address_bytes).await?,
        };

        self.standard_send(RequestBodyData::VoteProposal(data))
    }

    fn process_ping(&mut self, data: request_body::Ping, channel: ResponseChannel<PogResponse>) -> Result<()> {
        // choose a number of random peers
        let mut r = crypto::rand::thread_rng();
        let peers = self
            .peers
            .values()
            .choose_multiple(&mut r, NR_OF_PEERS_SENT)
            .iter()
            .map(|p| p.ip.to_vec())
            .collect::<Vec<Vec<u8>>>();
        let pong = response_body::Pong {
            peers,
        };
        self.send_response(channel, ResponseBodyData::Pong(pong))
    }

    fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(create_signature(data, self.keypair.secret().as_ref())?.to_vec())
    }

    fn public_key(&mut self) -> Vec<u8> {
        self.keypair.public().as_ref().to_vec()
    }

    fn process_response(&self, request_id: RequestId, response: PogResponse) -> Result<()> {
        Ok(())
    }

    fn get_prime_delegates(&self) -> Vec<PeerId> {
        todo!("write a fn that gets all online prime delegates")
    }

    // standard send means that a request is sent to all Prime Delegates + 10 random non Prime Delegates
    fn standard_send(&mut self, request: RequestBodyData) -> Result<()> {
        let mut r = crypto::rand::thread_rng();
        let prime_delegates = self.get_prime_delegates();
        let random_peers = self
            .peers
            .values()
            .choose_multiple(&mut r, NR_OF_PEERS_SENT)
            .iter()
            .map(|p| p.id)
            .collect::<Vec<PeerId>>();

        let all_peers = prime_delegates.iter().chain(random_peers.iter()).collect::<Vec<&PeerId>>();

        for peers in all_peers {
            let _ = self.send_request(peers, request.to_owned());
        }

        Ok(())
    }

    pub fn send_request(&mut self, peer: &PeerId, request: RequestBodyData) -> Result<RequestId> {
        let request_body = RequestBody {
            data: Some(request),
            signature_type: 0,
            timestamp: timestamp(),
        }
        .encode_to_vec();

        let header = RequestHeader {
            signature: self.sign(&request_body)?,
            public_key: self.public_key(),
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
            signature: self.sign(&response_body)?,
            public_key: self.public_key(),
        };

        let response = PogResponse {
            data: response_body,
            header: header.encode_to_vec(),
        };

        self.swarm.behaviour_mut().send_response(channel, response).map_err(|_| anyhow!("response failed"))
    }

    #[tokio::main]
    async fn get_primary_wallet(state: &ChampStateArc) -> Result<Wallet> {
        let wallet_manager = state.wallet_manager.read().await;
        let wallet = wallet_manager.primary_wallet().await.ok_or(anyhow!("no primary wallet found"))?;
        Ok(wallet.clone())
    }
}
