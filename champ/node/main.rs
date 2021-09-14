mod http;
mod rpc;
mod state;
mod consensus;

use anyhow::Result;
use clap::clap_app;
use futures::try_join;
use http::server::HttpServer;
use rpc::server::RpcServer;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::ChampState;

#[tokio::main]
async fn main() -> Result<()> {
    let db = storage::new(&storage::DatabaseConfig {
        kind: storage::Databases::Mock,
        uri: "",
    })
    .await?;

    let state = Arc::new(Mutex::new(ChampState { db }));
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
    let addr = "[::1]:50051".parse()?;
    let addr2 = "[::1]:50050".parse()?;
    let _ = try_join!(rpc_server.start(addr), http_server.start(addr2));
    Ok(())
}
