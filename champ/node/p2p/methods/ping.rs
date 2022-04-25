use anyhow::Result;
use crypto::rand::prelude::IteratorRandom;
use libp2p::request_response::ResponseChannel;
use pog_proto::p2p::{request_body, response_body};

use crate::p2p::{protocol::PogResponse, server::P2PServer};
use pog_proto::p2p::response_body::Data as ResponseBodyData;

const PING_PEER_COUNT: usize = 10;

pub fn process_ping(
    server: &mut P2PServer,
    _data: request_body::Ping,
    channel: ResponseChannel<PogResponse>,
) -> Result<()> {
    print!("got a ping, now sending pong :)");

    // choose a number of random peers
    let mut r = crypto::rand::thread_rng();
    let peers = server
        .peers
        .values()
        .choose_multiple(&mut r, PING_PEER_COUNT)
        .iter()
        .map(|p| p.ip.to_vec())
        .collect::<Vec<Vec<u8>>>();
    let pong = response_body::Pong {
        peers,
    };
    server.send_response(channel, ResponseBodyData::Pong(pong))
}
