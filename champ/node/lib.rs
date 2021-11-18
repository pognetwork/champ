mod auth;
mod blockpool;
mod cli;
mod config;
mod consensus;
mod http;
mod metrics;
mod rpc;
mod state;
pub mod storage;
pub mod validation;

use anyhow::Result;
use clap::Arg;
use http::HttpServer;
use roughtime::server::RoughTime;
use rpc::server::RpcServer;
use tokio::try_join;
use tracing::{debug, Level};

use crate::{blockpool::Blockpool, metrics::MetricsServer, state::ChampState};

pub async fn run() -> Result<()> {
    let matches = clap::App::new("champ-node")
        .version("0.0.1")
        .author("The POG Project <contact@pog.network>")
        .about("POGs reference implementation in rust")
        .arg(Arg::new("web").long("feat-web").takes_value(false).about("enables web interface"))
        .arg(Arg::new("metrics").long("feat-metrics").takes_value(false).about("enables metrics api"))
        .arg(Arg::new("roughtime").long("feat-roughtime").takes_value(false).about("enables roughtime server"))
        .arg(
            Arg::new("loglevel")
                .short('l')
                .long("loglevel")
                .value_name("LOGLEVEL")
                .about("Sets a log level. Can be one of `trace`, `debug`, `info`, `warn`, `error` ")
                .takes_value(true),
        )
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
                        )
                        .arg(
                            Arg::new("perms")
                                .about("adds permissions")
                                .takes_value(true)
                                .multiple_values(true)
                                .value_name("PERMISSIONS")
                                .forbid_empty_values(false)
                                .max_values(20)
                                .min_values(0),
                        ),
                )
                .subcommand(clap::App::new("generate-key").about("generates a node private key used for JWTs")),
        )
        .get_matches();

    let log_level = match matches.value_of("loglevel") {
        Some("trace") => Level::TRACE,
        Some("debug") => Level::DEBUG,
        Some("info") => Level::INFO,
        Some("warn") => Level::WARN,
        Some("error") => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(log_level)
        // build but do not install the subscriber.
        .init();

    debug!("loading config");
    let config = config::Config::new(Some(matches.clone()))?;

    debug!("initializing database");
    let db = storage::new(&config.database).await?;

    debug!("initializing blockpool");
    let mut blockpool = Blockpool::new();

    let state = ChampState::new(db, config, blockpool.get_client());
    blockpool.add_state(state.clone());

    if let Some(matches) = matches.subcommand_matches("admin") {
        debug!("command matched to admin subcommand");
        cli::admin::run(matches, &state).await?;
        return Ok(());
    }

    let rpc_server = RpcServer::new(state.clone());
    let http_server = HttpServer::new();
    let rough_time_server = RoughTime::new();
    let metrics_server = MetricsServer::new();

    // should default to ipv4 addresses since docker doesn't support ipv6 by default
    let rpc_addr = "0.0.0.0:50051".parse()?;
    let http_addr = "0.0.0.0:50050".parse()?;
    let rough_time_addr = "0.0.0.0:50049".parse()?;
    let metrics_addr = "0.0.0.0:50048".parse()?;

    debug!("starting services");
    let err = try_join!(
        rpc_server.start(rpc_addr),
        metrics_server.start(metrics_addr, matches.is_present("metrics")),
        http_server.start(http_addr, matches.is_present("web")),
        rough_time_server.start(rough_time_addr, matches.is_present("roughtime")),
        blockpool.start(),
    );

    tracing::error!("exiting, error occurred while starting services: {:?}", err);
    Ok(())
}
