mod config;
mod consensus;
mod http;
mod rpc;
mod state;
mod validation;

use anyhow::Result;
use clap::Arg;
use http::server::HttpServer;
use roughtime::server::RoughTime;
use rpc::server::RpcServer;
use tokio::try_join;

use crate::state::ChampState;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = clap::App::new("champ-node")
        .version("0.0.1")
        .author("The POG Project <contact@pog.network>")
        .about("POGs reference implementation in rust")
        .arg(Arg::new("web").about("enables web interface"))
        .arg(Arg::new("roughtime").about("enables roughtime server"))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .about("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config = config::Config::new(Some(matches.clone()))?;
    let db = storage::new(&storage::DatabaseConfig {
        kind: storage::Databases::Mock,
        uri: "",
    })
    .await?;
    let state = ChampState::new(db, config);

    let rpc_server = RpcServer::new(state.clone());
    let http_server = HttpServer::new();
    let rough_time_server = RoughTime::new();

    let addr = "[::1]:50051".parse()?;
    let addr2 = "[::1]:50050".parse()?;
    let addr3 = "[::1]:50049".parse()?;

    let _ = try_join!(
        rpc_server.start(addr),
        http_server.start(addr2, matches.value_of("web").is_some()),
        rough_time_server.start(addr3, matches.value_of("roughtime").is_some())
    );

    Ok(())
}
