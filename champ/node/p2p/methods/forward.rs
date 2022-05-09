use anyhow::Result;
use libp2p::PeerId;
use pog_proto::p2p::request_body;

use crate::p2p::server::P2PServer;

pub fn process_forward(_server: &mut P2PServer, _data: request_body::Forward, peer_id: PeerId) -> Result<()> {
    print!("got a forwarded message from {peer_id:?}");

    Ok(())
}
