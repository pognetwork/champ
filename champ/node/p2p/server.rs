#![allow(dead_code, unused_variables)]

use std::time::{SystemTime, UNIX_EPOCH};

use crate::p2p::pogprotocol::{self, PogProtocol};
use crate::state::ChampStateArc;
use anyhow::Result;
use crypto::signatures::ed25519::create_signature;
use libp2p::noise::AuthenticKeypair;
use pog_proto::p2p::{RequestData, RequestHeader, ResponseBody, ResponseHeader};
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
        match message {
            pogprotocol::RequestMessage {
                channel,
                request,
                request_id,
            } => self.process_request(channel, request, request_id),
            pogprotocol::ResponseMessage {
                request_id,
                response,
            } => self.process_response(request_id, response),
        }
    }

    // How to send resonses/requests:
    //
    // self.swarm.behaviour_mut().send_response(
    //     channel,
    //     PogResponse {
    //         data: vec![],
    //         header: vec![],
    //     },
    // )

    fn process_request(&mut self, channel: ResponseChannel<PogResponse>, request: PogRequest, request_id: RequestId) {
        let header_result = RequestHeader::decode(&*request.header);
        let header = match header_result {
            Ok(h) => h,
            _ => {
                tracing::error!("could not decode header");
                self.send_response(channel, pog_proto::p2p::response_body::Data::Failure(0));
                return;
            }
        };

        let data = request.data;
    }
    fn process_response(&self, request_id: RequestId, response: PogResponse) {}
    fn send_response(&mut self, channel: ResponseChannel<PogResponse>, response: pog_proto::p2p::response_body::Data) {
        let data = ResponseBody {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as u64,
            data: Some(response),
        }
        .encode_to_vec();

        let signature = match create_signature(&*data, self.keypair.secret().as_ref()) {
            Ok(s) => s,
            _ => {
                tracing::error!("could not create signature");
                return;
            }
        }
        .to_vec();
        let public_key = self.keypair.public().as_ref().to_vec();

        let header = ResponseHeader {
            signature,
            public_key,
        }
        .encode_to_vec();

        let result = self.swarm.behaviour_mut().send_response(
            channel,
            PogResponse {
                data,
                header,
            },
        );

        if let Err(error) = result {
            tracing::error!("error during response sending");
        }
    }
}
