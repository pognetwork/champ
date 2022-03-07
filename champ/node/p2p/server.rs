#![allow(dead_code, unused_variables)]

use std::time::{SystemTime, UNIX_EPOCH};

use crate::p2p::pogprotocol::{self, PogProtocol};
use crate::state::ChampStateArc;
use anyhow::{anyhow, Result};
use crypto::signatures::ed25519::create_signature;
use libp2p::noise::AuthenticKeypair;
use pog_proto::p2p::Failure;
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

pub struct P2PServer {
    state: ChampStateArc,
    swarm: Swarm<PogBehavior>,
    keypair: AuthenticKeypair<X25519Spec>,
}

fn timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as u64
}

impl P2PServer {
    pub fn new(state: ChampStateArc) -> Self {
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

        Self {
            state,
            swarm,
            keypair: dh_keys,
        }
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

        let data = request.data;
        Ok(())
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

    pub fn send_request(&mut self, peer: &PeerId, request: RequestBodyData) -> Result<RequestId> {
        let request_body = RequestBody {
            data: Some(request),
            signature_type: 0,
            timestamp: timestamp(),
        }
        .encode_to_vec();

        let header = ResponseHeader {
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
}
