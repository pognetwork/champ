use lazy_static::lazy_static;
use prometheus::register_int_gauge;

use super::server::timestamp;
use super::types::Peers;

lazy_static! {
    static ref PEERS_CONNECTED: prometheus::IntGauge =
        register_int_gauge!("peers_connected", "connected peers").unwrap();
    static ref PEERS_TOTAL: prometheus::IntGauge = register_int_gauge!("peers_total", "connected peers").unwrap();
}

pub fn update(peers: Peers) {
    let now = timestamp();
    let connected_peers = peers
        .iter()
        .filter(|peer| match peer.last_ping {
            Some(ping) => {
                print!("{ping}, {now}");
                ping > (now - 20)
            }
            None => false,
        })
        .count();

    PEERS_CONNECTED.set(connected_peers as i64);
    PEERS_TOTAL.set(peers.iter().count() as i64);
}
