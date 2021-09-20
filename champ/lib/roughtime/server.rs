use std::net::SocketAddr;

use hex;
use roughenough::config::{MemoryConfig, ServerConfig, DEFAULT_BATCH_SIZE, DEFAULT_STATUS_INTERVAL};
use roughenough::key::KmsProtection;
use roughenough::server::{MioEvents, Server};

pub struct RoughTime {}
impl RoughTime {
    pub fn new() -> RoughTime {
        RoughTime {}
    }

    pub async fn polling_loop(config: Box<dyn ServerConfig>) {
        let mut server = Server::new(config);
        let mut events = MioEvents::with_capacity(1024);

        loop {
            server.process_events(&mut events);
        }
    }

    pub async fn start(self, addr: SocketAddr, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        if !enabled {
            return Ok(());
        }

        let config = MemoryConfig {
            port: addr.port(),
            interface: addr.ip().to_string(),
            seed: hex::decode("a32049da0ffde0ded92ce10a0230d35fe615ec8461c14986baa63fe3b3bac3db").unwrap(),
            batch_size: DEFAULT_BATCH_SIZE,
            status_interval: DEFAULT_STATUS_INTERVAL,
            kms_protection: KmsProtection::Plaintext,
            health_check_port: None,
            client_stats: false,
            fault_percentage: 0,
        };

        println!("starting roughtime server at {}", addr);
        Self::polling_loop(Box::from(config)).await;
        Ok(())
    }
}
