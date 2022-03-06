use crate::p2p::pogprotocol;
use crate::state::ChampStateArc;
use anyhow::Result;
use libp2p::core::{identity, upgrade, Transport};
use libp2p::mplex::MplexConfig;
use libp2p::noise::{Keypair, NoiseConfig, X25519Spec};
use libp2p::swarm::SwarmBuilder;
use libp2p::tcp::TokioTcpConfig;
use libp2p::{PeerId, Swarm};
use tracing::info;

pub struct P2PServer {
    #[allow(dead_code)]
    state: ChampStateArc,
}

// WORKING CODE THIS IS INSPIRED BY
// https://sourcegraph.com/github.com/airalab/robonomics/-/blob/protocol/src/reqres.rs

// PROJECTS USING A SIMILAR SETUP FOR INSPIRATION
// https://sourcegraph.com/search?q=context:global+RequestResponse::new&patternType=literal

// Keys: Primary Wallet keys
// PeerID: Public key of the Primary Wallet
// Topics: ?

// #[derive(Debug, libp2p::NetworkBehaviour)]
// struct PogBehaviour {
//     pog: RequestResponse<PogCodec>,
// }

impl P2PServer {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("P2P ID: ");

        let id_keys = identity::Keypair::generate_ed25519();
        let dh_keys = Keypair::<X25519Spec>::new().into_authentic(&id_keys).unwrap();
        let noise = NoiseConfig::xx(dh_keys).into_authenticated();

        let peer_id = PeerId::from(id_keys.public());

        let transp = TokioTcpConfig::new()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise)
            .multiplex(MplexConfig::new())
            .boxed();

        let mut swarm = SwarmBuilder::new(transp, pogprotocol::new(), peer_id).build();
        Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;
        Ok(())
    }
}
