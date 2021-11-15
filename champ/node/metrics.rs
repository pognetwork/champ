use std::net::SocketAddr;

use anyhow::Result;
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

use prometheus::{Encoder, TextEncoder};

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

        let serve_future =
            Server::bind(&addr).serve(make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(serve_req)) }));

        serve_future.await?;
        Ok(())
    }
}

async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let response =
        Response::builder().status(200).header(CONTENT_TYPE, encoder.format_type()).body(Body::from(buffer)).unwrap();

    Ok(response)
}
