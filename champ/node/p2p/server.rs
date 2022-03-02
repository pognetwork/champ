use crate::state::ChampStateArc;
use anyhow::Result;
use libp2p::core::{identity, upgrade, Transport};
use libp2p::noise::{Keypair, NoiseConfig, X25519Spec};
use libp2p::tcp::TokioTcpConfig;
use pog_proto::api::SignedBlock;
use tracing::info;

pub struct P2PServer {
    state: ChampStateArc,
}

// Keys: Primary Wallet keys
// PeerID: Public key of the Primary Wallet
// Topics: ?

struct PogNetworking {}
#[derive(Debug, libp2p::NetworkBehaviour)]
struct PogBehaviour {
    PogNetwork: PogNetworking,
}

#[derive(Debug)]
pub enum PogEvent {
    Message(String),
}

impl libp2p::swarm::NetworkBehaviour for PogNetworking {
    type ConnectionHandler;

    type OutEvent;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        todo!()
    }

    fn inject_event(
        &mut self,
        peer_id: libp2p::PeerId,
        connection: libp2p::core::connection::ConnectionId,
        event: <<Self::ConnectionHandler as libp2p::swarm::IntoConnectionHandler>::Handler as libp2p::swarm::ConnectionHandler>::OutEvent,
    ) {
        todo!()
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
        params: &mut impl libp2p::swarm::PollParameters,
    ) -> std::task::Poll<libp2p::swarm::NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        todo!()
    }
}

impl P2PServer {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }

    pub async fn start(&self) -> Result<Box<dyn std::error::Error>> {
        info!("P2P ID: ");

        let id_keys = identity::Keypair::generate_ed25519();
        let dh_keys = Keypair::<X25519Spec>::new().into_authentic(&id_keys).unwrap();
        let noise = NoiseConfig::xx(dh_keys).into_authenticated();

        let transp = TokioTcpConfig::new().upgrade(upgrade::Version::V1).authenticate(noise);

        let mut behaviour = PogBehaviour {};
        Ok(())
    }
}
