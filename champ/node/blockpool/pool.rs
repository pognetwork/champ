#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use pog_proto::api::SignedBlock;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};
use tracing::info;

use std::collections::VecDeque;

use crate::{state::ChampStateArc, validation::block};

#[derive(Debug)]
struct QueueItem {
    block: pog_proto::api::SignedBlock,
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
    pub async fn process_block(&self, block: pog_proto::api::RawBlock) -> Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let block: SignedBlock = block.try_into()?;

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

        let state = self.state.clone().unwrap();

        info!("blockpool started listening to incoming commands");
        while let Some(cmd) = self.rx.recv().await {
            use Command::*;
            match cmd {
                ProcessBlock {
                    block,
                    resp,
                } => {
                    let result = block::validate(&block, &state).await;
                    match result {
                        Ok(_) => {
                            self.block_queue.push_back(QueueItem {
                                block,
                            });
                            let _ = resp.send(Ok(()));
                        } //TODO: Vote yes
                        Err(block::BlockValidationError::Invalid(_)) => {
                            let _ = resp.send(Ok(()));
                        } //TODO: maybe retry or handle errors and then Start a vote
                        Err(block::BlockValidationError::Error(err)) => {
                            let _ = resp.send(Err(anyhow!("error {err}")));
                        }
                    }
                }
                ProcessVote {
                    resp,
                } => {
                    let _ = resp.send(Err(anyhow!("not implemented")));
                }
                GetQueueSize {
                    resp,
                } => {
                    let _ = resp.send(Ok(self.block_queue.len() as u64));
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
        block: pog_proto::api::SignedBlock,
        resp: Responder<()>,
    },
    ProcessVote {
        resp: Responder<()>,
    },
    GetQueueSize {
        resp: Responder<u64>,
    },
}
