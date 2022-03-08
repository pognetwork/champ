mod auth;
mod blockpool;
mod cli;
mod config;
mod consensus;
mod http;
mod metrics;
mod p2p;
mod rpc;
mod state;
pub mod storage;
pub mod validation;
pub mod wallets;

use std::env;

use anyhow::{anyhow, Result};
use http::HttpServer;
use roughtime::server::RoughTime;
use state::ChampStateArc;
use tokio::{sync::RwLock, try_join};
use tracing::{debug, Level};

use crate::{
    blockpool::Blockpool,
    metrics::MetricsServer,
    p2p::server::P2PServer,
    rpc::server::RpcServer,
    state::{ChampState, ChampStateArgs},
    wallets::WalletManager,
};

/// run is champ's main entry point
/// This function processes cli arguments, sets up all global state, and starts various services
pub async fn run() -> Result<()> {
    let matches = cli::parser::new();
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
    let config = RwLock::new(config);

    debug!("initializing database");
    let database_config = &config.read().await.database.clone();
    let db = storage::new(database_config).await?;

    debug!("initializing blockpool");
    let mut blockpool = Blockpool::new();

    debug!("initializing wallet manager");
    let wallet_manager = WalletManager::new();
    let wallet_manager = RwLock::new(wallet_manager);

    debug!("initializing champ state");
    let state = ChampState::new(ChampStateArgs {
        db,
        config,
        wallet_manager,
        blockpool_client: blockpool.get_client(),
    });

    debug!("injecting state into blockpool");
    blockpool.add_state(state.clone());

    debug!("injecting state into walletmanager");
    let wallet_manager = &mut state.wallet_manager.write().await;
    wallet_manager.add_state(state.clone());
    wallet_manager.initialize().await?;

    if let Some(matches) = matches.subcommand_matches("admin") {
        debug!("command matched to admin subcommand");
        cli::admin::run(matches, &state).await?;
        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("wallet") {
        debug!("command matched to wallet subcommand");
        cli::wallet::run(matches, &state).await?;
        return Ok(());
    }

    process_env(state.clone()).await?;

    let mut p2p_server = P2PServer::new(state.clone());
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
        p2p_server.start(),
        rpc_server.start(rpc_addr),
        metrics_server.start(metrics_addr, matches.is_present("metrics")),
        http_server.start(http_addr, matches.is_present("web")),
        rough_time_server.start(rough_time_addr, matches.is_present("roughtime")),
        blockpool.start(),
    );

    tracing::error!("exiting, error occurred while starting services: {:?}", err);
    Ok(())
}

/// process_env processes champ-related environment variables
async fn process_env(state: ChampStateArc) -> Result<()> {
    let mut config = state.config.write().await;
    let mut wallet_manager = state.wallet_manager.write().await;

    if let Ok(primary_wallet_password) = env::var("CHAMP_PRIMARY_WALLET_PASSWORD") {
        if env::var("CHAMP_GENERATE_PRIMARY_WALLET").is_ok() {
            config.consensus.primary_wallet = Some(wallet_manager.create_wallet(&primary_wallet_password).await?);
            config.write()?;
        }

        match config.consensus.primary_wallet.clone() {
            Some(primary_wallet) => wallet_manager.unlock_wallet(primary_wallet, &primary_wallet_password).await?,
            None => return Err(anyhow!("CHAMP_PRIMARY_WALLET_PASSWORD defined but no primary wallet to unlock. Specify primary wallet in config.consensus.primary_wallet"))
        }
    }

    Ok(())
}
