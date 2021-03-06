use crate::auth::interceptors::interceptor_auth;
use crate::metrics::ServiceStatus;
use crate::rpc::lattice::{LatticeServer, LatticeService};
use crate::rpc::node_admin::{NodeAdminServer, NodeAdminService};
use crate::rpc::node_user::{NodeUserServer, NodeUserService};
use crate::rpc::node_wallet_manager::{NodeWalletManagerServer, NodeWalletManagerService};
use crate::state::ChampStateArc;
use std::{net::SocketAddr, time::Duration};

use anyhow::anyhow;
use tonic::transport::Server;
use tonic::Request;
use tracing::info;

use lazy_static::lazy_static;
use prometheus::register_int_gauge;

lazy_static! {
    static ref GRPC_HEALTH: prometheus::IntGauge = register_int_gauge!("grpc_health", "grpc server status").unwrap();
}

#[derive(Debug)]
pub struct RpcServer {
    state: ChampStateArc,
}

impl RpcServer {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }

    pub async fn start(&self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        GRPC_HEALTH.set(ServiceStatus::Starting as i64);

        let (public_key, users) = {
            let cfg = &self.state.config.read().await;
            (cfg.admin.jwt_public_key.to_owned(), cfg.node_users.clone())
        };

        let block_service_server = LatticeServer::new(LatticeService::new(self.state.clone()));
        let node_user = NodeUserServer::new(NodeUserService::new(self.state.clone()));

        info!("starting rpc server at {}", addr);
        let grpc_web = tonic_web::config()
            // .allow_origins(vec!["http://admin.localhost:2020"])
            .allow_all_origins()
            .expose_headers(vec!["x-request-id", "x-grpc-web"]);

        // The stack of middleware that our service will be wrapped in
        let timeout = tower::ServiceBuilder::new().timeout(Duration::from_secs(30)).into_inner();
        let server =
            Server::builder().accept_http1(true).layer(timeout).add_service(grpc_web.enable(block_service_server));

        GRPC_HEALTH.set(ServiceStatus::Healthy as i64);

        if self.state.config.read().await.admin.enabled {
            let public_key = public_key.ok_or_else(|| anyhow!("cannot start admin service: no jwt key"))?;
            let public_key2 = public_key.clone();
            let users2 = users.clone();

            let node_admin_server = NodeAdminServer::with_interceptor(
                NodeAdminService::new(self.state.clone()),
                move |request: Request<()>| interceptor_auth(request, &public_key, &users),
            );

            let node_wallet_manager_server = NodeWalletManagerServer::with_interceptor(
                NodeWalletManagerService::new(self.state.clone()),
                move |request| interceptor_auth(request, &public_key2, &users2),
            );

            if let Err(e) = server
                .add_service(grpc_web.enable(node_admin_server))
                .add_service(grpc_web.enable(node_wallet_manager_server))
                .add_service(grpc_web.enable(node_user))
                .serve(addr)
                .await
            {
                tracing::error!("error while starting grpc server: {e:?}");
                GRPC_HEALTH.set(ServiceStatus::Broken as i64);
                return Err(anyhow!("failed to start grpc server").into());
            }
        } else {
            info!("admin service is disabled");
            if let Err(e) = server.serve(addr).await {
                tracing::error!("error while starting grpc server: {e:?}");
                GRPC_HEALTH.set(ServiceStatus::Broken as i64);
                return Err(anyhow!("failed to start grpc server").into());
            }
        }

        Ok(())
    }
}
