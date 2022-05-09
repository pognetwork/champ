use std::sync::Arc;

pub use crate::p2p::protocol;
use dashmap::DashMap;
use libp2p::noise::AuthenticKeypair;
use libp2p::request_response::RequestResponseEvent;
use libp2p::swarm::{ConnectionHandlerUpgrErr, SwarmEvent};
use libp2p::PeerId;
pub use pog_proto::p2p::{request_body, response_body, Failure};

pub use pog_proto::p2p::{
    request_body::Data as RequestBodyData, response_body::Data as ResponseBodyData, RequestBody, RequestHeader,
    ResponseBody, ResponseHeader,
};

use self::protocol::{PogRequest, PogResponse};

#[derive(Clone)]
pub struct Peer {
    pub id: PeerId,
    pub ip: libp2p::Multiaddr,
    pub voting_power: Option<u64>,
    pub last_ping: Option<u64>,
}

pub type Event = SwarmEvent<RequestResponseEvent<PogRequest, PogResponse>, ConnectionHandlerUpgrErr<std::io::Error>>;
pub type Peers = Arc<DashMap<PeerId, Peer>>;
pub type NodeKeypair = AuthenticKeypair<libp2p::noise::X25519Spec>;
