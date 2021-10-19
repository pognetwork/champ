use anyhow::{Context, Result};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

use std::collections::VecDeque;

use crate::state::ChampStateArc;

#[derive(Debug)]
struct QueueItem {
    block: pog_proto::api::Block,
}

#[derive(Debug)]
pub struct Blockpool {
    pub tx: Sender<Command>,
    rx: Receiver<Command>,
    block_queue: VecDeque<QueueItem>,
    state: Option<ChampStateArc>,
}

impl Default for Blockpool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct BlockpoolClient {
    tx: Sender<Command>,
}

impl BlockpoolClient {
    pub async fn process_block(&self, block: pog_proto::api::Block) -> Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.tx
            .send(Command::ProcessBlock {
                block,
                resp: resp_tx,
            })
            .await
            .with_context(|| "error sending process request")?;
        resp_rx.await?
    }

    pub async fn process_vote(&self) -> Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.tx
            .send(Command::ProcessVote {
                resp: resp_tx,
            })
            .await
            .with_context(|| "error sending process request")?;
        resp_rx.await?
    }

    pub async fn get_queue_size(&self) -> Result<u64> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.tx
            .send(Command::GetQueueSize {
                resp: resp_tx,
            })
            .await
            .with_context(|| "error sending process request")?;
        resp_rx.await?
    }
}

impl Blockpool {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);
        Self {
            tx,
            rx,
            block_queue: VecDeque::with_capacity(10_000),
            state: None,
        }
    }

    pub fn add_state(&mut self, state: ChampStateArc) {
        self.state = Some(state);
    }

    pub fn get_client(&self) -> BlockpoolClient {
        BlockpoolClient {
            tx: self.tx.clone(),
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<std::io::Error>> {
        if self.state.is_none() {
            panic!("add_state has to be called first")
        }

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
