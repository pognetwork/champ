#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use pog_proto::api::{BlockID, SignedBlock};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};
use tracing::info;

use std::collections::{HashMap, VecDeque};

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
    block_votes: HashMap<BlockID, Vec<u64>>,
}

impl Default for Blockpool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
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

    pub async fn process_vote(&self, block: pog_proto::api::RawBlock) -> Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let block: SignedBlock = block.try_into()?;

        self.tx
            .send(Command::ProcessVote {
                block,
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

    /// Gets the total voting power in the network
    pub fn get_total_network_power(&self) -> f64 {
        //TODO: Get all voting power of all prime delegates combined
        let total_prime_delegate_power = 100_000_000_f64;
        total_prime_delegate_power
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
            block_votes: HashMap::new(),
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
                    let quorum = self.calculate_quorum(&block);

                    if quorum > 0.6 {
                        match result {
                            Ok(_) => {
                                self.block_queue.push_back(QueueItem {
                                    block,
                                });
                                let _ = resp.send(Ok(()));
                            } //TODO: Send a final vote back
                            Err(block::BlockValidationError::Invalid(_)) => {
                                let _ = resp.send(Ok(()));
                            } //TODO: maybe retry or handle errors and then Start a vote
                            Err(block::BlockValidationError::Error(err)) => {
                                let _ = resp.send(Err(anyhow!("error {err}")));
                            }
                        }
                    }
                }
                ProcessVote {
                    block,
                    resp,
                } => {
                    // If THIS_ID is a Prime Delegate get the voting power of this account
                    //TODO: let own_voting = voting_power::get_active_power(self.state, THIS_ID);
                    let _result = block::validate(&block, &state).await;
                    //match result {
                    //    Ok(_) => todo!("re-send the block to the network"),
                    //    Err(_) => todo!("wait or discard block"),
                    //}

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

    fn calculate_quorum(&mut self, block: &SignedBlock) -> f64 {
        // count the final votes received based on a blockID and once 60% of the online voting has been reached, add the block to the chain
        let block_id = block.get_id();
        let all_votes = &self.block_votes[&block_id];
        let total_votes = all_votes.iter().sum::<u64>() as f64;
        let total_network_power = self.state.as_ref().unwrap().blockpool_client.get_total_network_power();
        total_votes / total_network_power
        // if the block came from a final vote:
        // add the block to the chain and send own final vote"
        // if the block came frma vote proposal:
        // we check if we are prime delegate and if yes we cast our vote and send our vote out
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
        block: pog_proto::api::SignedBlock,
        resp: Responder<()>,
    },
    GetQueueSize {
        resp: Responder<u64>,
    },
}
