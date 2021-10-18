use anyhow::Result;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

#[derive(Debug)]
pub struct Blockpool {
    tx: Sender<Command>,
    rx: Receiver<Command>,
}

#[derive(Debug)]
pub struct BlockpoolClient {
    tx: Sender<Command>,
}

impl Blockpool {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);
        Self {
            tx,
            rx,
        }
    }

    pub fn get_client(&self) -> BlockpoolClient {
        BlockpoolClient {
            tx: self.tx.clone(),
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<std::io::Error>> {
        while let Some(cmd) = self.rx.recv().await {
            use Command::*;
            match cmd {
                Get {
                    key,
                    resp,
                } => {
                    let _ = resp.send(Ok(()));
                }
                Set {
                    key,
                    val,
                    resp,
                } => {
                    let _ = resp.send(Ok(()));
                }
            }
        }
        Ok(())
    }
}

type Responder<T> = oneshot::Sender<Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<()>,
    },
    Set {
        key: String,
        val: Vec<u8>,
        resp: Responder<()>,
    },
}
