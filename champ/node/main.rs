mod cli;
mod consensus;
mod http;
mod rpc;
mod state;
mod validation;

use anyhow::{anyhow, Result};
use clap::Arg;
use futures::try_join;
use http::server::HttpServer;
use roughtime::server::RoughTime;
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

    let state = Arc::new(Mutex::new(ChampState {
        db,
    }));

    // cargo run --bin champ-node -- --help
    // champ-node --help

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
        .subcommand(
            clap::App::new("create-user")
                .about("creates a user for the web api")
                .version("0.0.1")
                .arg(Arg::new("username").short('u').about("new username"))
                .arg(Arg::new("password").short('p').about("new username")),
        )
        .get_matches();

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
    }

    if let Some(ref matches) = matches.subcommand_matches("create-user") {
        let user = matches.value_of("username").ok_or(anyhow!("username cannot be empty"))?;
        let password = matches.value_of("password").ok_or(anyhow!("username cannot be empty"))?;
        cli::create_user::run(&state, user, password).await;
        return Ok(());
    }

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
