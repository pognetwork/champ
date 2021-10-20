pub mod blockpool;
mod cli;
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

use crate::{blockpool::Blockpool, state::ChampState};

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
        .subcommand(
            clap::App::new("admin")
                .about("access to the admin interface")
                .subcommand(
                    clap::App::new("create-user")
                        .about("creates a user for the web api")
                        .after_help("Format: -u [username] -p [password]")
                        .arg(
                            Arg::new("username")
                                .short('u')
                                .about("new username")
                                .takes_value(true)
                                .value_name("USERNAME")
                                .forbid_empty_values(true),
                        )
                        .arg(
                            Arg::new("password")
                                .short('p')
                                .about("new password")
                                .takes_value(true)
                                .value_name("PASSWORD")
                                .forbid_empty_values(true),
                        ),
                )
                .subcommand(clap::App::new("generate-key").about("generates a node private key used for JWTs")),
        )
        .get_matches();

    let config = config::Config::new(Some(matches.clone()))?;

    println!("{:?}", config);

    let db = storage::new(&storage::DatabaseConfig {
        kind: storage::Databases::Sled,
        uri: None,
        path: Some("".to_string()),
    })
    .await?;
    let mut blockpool = Blockpool::new();
    let state = ChampState::new(db, config, blockpool.get_client());
    blockpool.add_state(state.clone());

    if let Some(matches) = matches.subcommand_matches("admin") {
        cli::admin::run(matches, &state).await?;
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
        rough_time_server.start(addr3, matches.value_of("roughtime").is_some()),
        blockpool.start(),
    );

    Ok(())
}
