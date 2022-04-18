#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::ChampStateArc;
use crate::wallets::{Wallet, WalletPrivateKey};
use anyhow::{anyhow, Result};
use crypto::rand::seq::IteratorRandom;
use crypto::signatures::ed25519::verify_signature;
use libp2p::identity::{self, ed25519};
use libp2p::noise::AuthenticKeypair;

use pog_proto::Message;

use libp2p::{
    core::{upgrade, Transport},
    futures::StreamExt,
    mplex::MplexConfig,
    noise::{self},
    request_response::{RequestId, RequestResponseEvent, ResponseChannel},
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    {PeerId, Swarm},
};

use super::methods;
use super::protocol::{PogBehavior, PogMessage, PogRequest, PogResponse};
use super::types::{protocol, Failure};
use super::types::{request_body, RequestBody, RequestBodyData, RequestHeader};
use super::types::{ResponseBody, ResponseBodyData, ResponseHeader};

#[derive(Clone)]
pub struct Peer {
    pub voting_power: u64,
    pub ip: libp2p::Multiaddr,
    pub last_ping: std::time::SystemTime,
    pub id: PeerId,
}

pub struct P2PServer {
    pub peers: HashMap<[u8; 32], Peer>,
    state: ChampStateArc,
    swarm: Swarm<PogBehavior>,
    keypair: NodeKeypair,
    node_wallet: Wallet,
}

pub type NodeKeypair = AuthenticKeypair<noise::X25519Spec>;
const NR_OF_PEERS_SENT: usize = 10;

fn timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as u64
}

fn get_random_peers_id(peers: HashMap<[u8; 32], Peer>, max_nr_of_peers: usize) -> Vec<PeerId> {
    let mut r = crypto::rand::thread_rng();
    peers.values().choose_multiple(&mut r, max_nr_of_peers).iter().map(|p| p.id).collect::<Vec<PeerId>>()
}

impl P2PServer {
    pub async fn new(state: ChampStateArc) -> Result<Self> {
        let node_wallet = {
            let wallet_manager = state.wallet_manager.read().await;
            let wallet = wallet_manager.primary_wallet().await.ok_or_else(|| anyhow!("no primary wallet found"))?;
            wallet.clone()
        };

        println!("4");

        let id_keys = {
            let secret_key: ed25519::SecretKey =
                ed25519::SecretKey::from_bytes(node_wallet.private_key().unwrap()).unwrap();
            let keypair = ed25519::Keypair::from(secret_key);
            identity::Keypair::Ed25519(keypair)
        };

        let dh_keys = noise::Keypair::<noise::X25519Spec>::new().into_authentic(&id_keys).unwrap();

        let peer_id = PeerId::from(id_keys.public());
        let pog_protocol = protocol::PogProtocol::new();

        let noise = noise::NoiseConfig::xx(dh_keys.clone()).into_authenticated();
        let transp = TokioTcpConfig::new()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise)
            .multiplex(MplexConfig::new())
            .boxed();

        let swarm = SwarmBuilder::new(transp, pog_protocol.behavior(), peer_id).build();
        let peers = HashMap::new();

        Ok(Self {
            state,
            swarm,
            keypair: dh_keys,
            peers,
            node_wallet,
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
            protocol::RequestMessage {
                channel,
                request,
                request_id,
            } => self.process_request(channel, request, request_id),
            protocol::ResponseMessage {
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

        if let Err(e) = verify_signature(&request.data, &self.node_wallet.public_key()?, &*header.signature) {
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

        let result: Result<()> = match data {
            // request_body::Data::Forward(data) => self.process_forward(*data),
            // request_body::Data::FinalVote(data) => self.process_final_vote(data),
            // request_body::Data::VoteProposal(data) => self.process_vote_proposal(data),
            request_body::Data::Ping(data) => return methods::process_ping(self, data, channel),
            _ => Ok(()),
        };

        match result {
            Ok(_) => self.send_response(channel, ResponseBodyData::Success(pog_proto::p2p::response_body::Success {})),
            Err(err) => self.send_response(channel, ResponseBodyData::Failure(Failure::MalformedRequest.into())),
        }
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
            signature: self.node_wallet.sign(&request_body)?.to_vec(),
            public_key: self.node_wallet.public_key()?.to_vec(),
        };

        let request = PogRequest {
            data: request_body,
            header: header.encode_to_vec(),
        };

        Ok(self.swarm.behaviour_mut().send_request(peer, request))
    }

    pub fn send_response(&mut self, channel: ResponseChannel<PogResponse>, response: ResponseBodyData) -> Result<()> {
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

// #[tokio::main]
// async fn process_final_vote(&mut self, data: FinalVote) -> Result<()> {
//     // all nodes who calculate a 60% quorum from the vote proposal need to send a final vote
//     let raw_block = match data.block {
//         Some(block) => block,
//         None => return Err(anyhow!("block was none")),
//     };

//     if self.state.blockpool_client.process_vote(raw_block.clone(), data.vote, true).await.is_err() {
//         return Err(anyhow!("error during processing vote"));
//     }

//     let data = request_body::FinalVote {
//         block: Some(raw_block),
//         vote: voting_power::get_active_power(&self.state, self.primary_wallet.account_address_bytes).await?,
//     };

//     self.standard_send(RequestBodyData::FinalVote(data))
// }

// #[tokio::main]
// async fn process_vote_proposal(&mut self, data: VoteProposal) -> Result<()> {
//     // if prime delegate: cast own vote and send to all other prime delegates and 10 non prime delegates
//     let raw_block = match data.block {
//         Some(block) => block,
//         None => return Err(anyhow!("block was none")),
//     };

//     if self.state.blockpool_client.process_vote(raw_block.clone(), data.vote, false).await.is_err() {
//         return Err(anyhow!("error during processing vote"));
//     }

//     //TODO: Only add vote if this is a prime delegate
//     let data = request_body::VoteProposal {
//         block: Some(raw_block),
//         vote: voting_power::get_active_power(&self.state, self.primary_wallet.account_address_bytes).await?,
//     };

//     //TODO: If quorum has been reached, send FinalVote
//     self.standard_send(RequestBodyData::VoteProposal(data))
// }
