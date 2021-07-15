mod http;
mod rpc;

use clap::clap_app;
use futures::join;
use http::server::HttpServer;
use rpc::server::RpcServer;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct ChampState {
    username: String,
}

pub type ChampStateMutex = Arc<Mutex<ChampState>>;

#[tokio::main]

async fn main() {
    let state = Arc::new(Mutex::new(ChampState {
        username: String::from("tyee"),
    }));

    let matches = clap_app!("champ-node" =>
        (version: "0.0.1")
        (author: "The POG Project <contact@pog.network>")
        (about: "POG's reference implementation in rust")
        (@arg CONFIG: -c --config +takes_value "Sets a custom config file")
    )
    .get_matches();

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
    }

    let rpc_server = RpcServer::new(state.clone());
    let http_server = HttpServer::new();
    let addr = "[::1]:50051".parse().unwrap();
    let addr2 = "[::1]:50050".parse().unwrap();

    let _ = join!(rpc_server.start(addr), http_server.start(addr2));
}
