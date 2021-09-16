use roughenough::config;
use roughenough::config::ServerConfig;
use roughenough::server::{MioEvents, Server};

pub async fn start_server(port: u16) {
    let config = config::MemoryConfig::new(port);
    polling_loop(Box::from(config))
}

pub fn polling_loop(config: Box<dyn ServerConfig>) {
    let mut server = Server::new(config);
    let mut events = MioEvents::with_capacity(1024);

    loop {
        server.process_events(&mut events);
    }
}
