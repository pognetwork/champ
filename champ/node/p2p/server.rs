#![allow(dead_code, unused_variables)]

use crate::p2p::pogprotocol::{self, PogProtocol};
use crate::state::ChampStateArc;
use anyhow::Result;

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
}

impl P2PServer {
    pub fn new(state: ChampStateArc) -> Self {
        let id_keys = identity::Keypair::generate_ed25519();
        let dh_keys = Keypair::<X25519Spec>::new().into_authentic(&id_keys).unwrap();
        let peer_id = PeerId::from(id_keys.public());

        let pog_protocol = PogProtocol::new();

        let noise = NoiseConfig::xx(dh_keys).into_authenticated();
        let transp = TokioTcpConfig::new()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise)
            .multiplex(MplexConfig::new())
            .boxed();

        let swarm = SwarmBuilder::new(transp, pog_protocol.behavior(), peer_id).build();

        Self {
            state,
            swarm,
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

    fn process_message(&self, peer: PeerId, message: PogMessage) {
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

    fn process_request(&self, channel: ResponseChannel<PogResponse>, request: PogRequest, request_id: RequestId) {}
    fn process_response(&self, request_id: RequestId, response: PogResponse) {}
}
