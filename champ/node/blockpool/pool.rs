use anyhow::Result;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

use std::collections::VecDeque;

#[derive(Debug)]
struct QueueItem {
    block: pog_proto::api::Block,
}

#[derive(Debug)]
pub struct Blockpool {
    pub tx: Sender<Command>,
    rx: Receiver<Command>,
    block_queue: VecDeque<QueueItem>,
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
            block_queue: VecDeque::with_capacity(10_000),
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
                ProcessBlock {
                    block: _,
                    resp,
                } => {
                    let _ = resp.send(Ok(()));
                }
                ProcessVote {
                    resp,
                } => {
                    let _ = resp.send(Ok(()));
                }
                GetQueueSize {
                    resp,
                } => {
                    let _ = resp.send(Ok(0));
                }
            }
        }
        Ok(())
    }
}

type Responder<T> = oneshot::Sender<Result<T>>;

#[derive(Debug)]
pub enum Command {
    ProcessBlock {
        block: pog_proto::api::Block,
        resp: Responder<()>,
    },
    ProcessVote {
        resp: Responder<()>,
    },
    GetQueueSize {
        resp: Responder<u64>,
    },
}
