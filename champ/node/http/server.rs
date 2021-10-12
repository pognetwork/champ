use derive_new::new;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use tower::ServiceBuilder;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

#[derive(Debug, new)]
pub struct HttpServer {}

impl HttpServer {
    pub async fn start(&self, addr: SocketAddr, enable: bool) -> Result<(), Box<dyn std::error::Error>> {
        if !enable {
            return Ok(());
        }

        let http_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(HttpHandler::handle)) });

        println!("starting http server at {}", addr);

        let service = ServiceBuilder::new().timeout(Duration::from_secs(30)).service(http_service);

        Server::bind(&addr).serve(service).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct HttpHandler {}
impl HttpHandler {
    pub async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok(Response::new("Hello, World".into()))
    }
}
