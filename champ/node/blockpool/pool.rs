#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use pog_proto::api::{BlockID, SignedBlock};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};
use tracing::info;

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    consensus::{self, voting_power},
    state::ChampStateArc,
    validation::block,
};

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
    sent_votes: HashSet<BlockID>,
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
    pub async fn process_block(&self, block: pog_proto::api::RawBlock, vote: u64) -> Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let block: SignedBlock = block.try_into()?;

        self.tx
            .send(Command::ProcessBlock {
                block,
                vote,
                resp: resp_tx,
            })
            .await
            .with_context(|| "error sending process request")?;
        resp_rx.await?
    }

    pub async fn process_vote(&self, block: pog_proto::api::RawBlock, vote: u64) -> Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let block: SignedBlock = block.try_into()?;

        self.tx
            .send(Command::ProcessVote {
                block,
                vote,
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
            sent_votes: HashSet::new(),
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
                    vote,
                    resp,
                } => {
                    let result = block::validate(&block, &state).await;
                    //TODO: Fix this with different quorum!
                    let quorum = self.calculate_quorum(&block);

                    // Quorum setting in Consensus module - currently 60%
                    if quorum > consensus::voting_power::VOTE_PERCENTAGE_NEEDED {
                        match result {
                            Ok(_) => {
                                self.block_queue.push_back(QueueItem {
                                    block: block.clone(),
                                });
                                self.sent_votes.remove(&block.get_id());
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
                    vote,
                    resp,
                } => {
                    //TODO: Fix this
                    // here a vote is received
                    // if prime delegate, add vote and send back
                    // if quorum is reached, send final vote
                    let block_id = block.get_id();
                    if self.sent_votes.contains(&block_id) {
                        let _ = resp.send(Err(anyhow!("block already processed")));
                        continue;
                    }
                    self.sent_votes.insert(block_id);

                    let wallet_manager = state.wallet_manager.read().await;
                    let wallet =
                        wallet_manager.primary_wallet().await.ok_or_else(|| anyhow!("no primary wallet found"));
                    if wallet.is_err() {
                        continue;
                    }
                    let this_account_address = wallet.unwrap().account_address_bytes;

                    let result = voting_power::get_active_power(
                        self.state.as_ref().expect("how did this happen?"),
                        this_account_address,
                    )
                    .await;
                    if result.is_err() {
                        continue;
                    }
                    let this_voting_power = result.unwrap();

                    // TODO: only add own voting if this is delegate
                    // TODO: check somewhere that the same sender cant vote twice
                    self.save_vote(vote, &block_id);
                    self.save_vote(this_voting_power, &block_id);

                    // Add own vote to quorum if prime delegate
                    let _quorum = self.calculate_quorum(&block);

                    let result = block::validate(&block, &state).await;
                    match result {
                        Ok(_) => {
                            let _ = resp.send(Ok(()));
                        }
                        Err(_) => {
                            let _ = resp.send(Err(anyhow!("not implemented")));
                        }
                    }
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

    fn calculate_quorum(&self, block: &SignedBlock) -> f64 {
        // count the final votes received based on a blockID and once 60% of the online voting has been reached, add the block to the chain
        let block_id = block.get_id();

        let all_votes = self.block_votes.get(&block_id);

        let total_votes = all_votes.unwrap().iter().sum::<u64>() as f64;

        let total_network_power = self.state.as_ref().unwrap().blockpool_client.get_total_network_power();
        total_votes / total_network_power
    }

    fn save_vote(&mut self, vote: u64, block_id: &[u8; 32]) {
        let all_votes = match self.block_votes.get(block_id) {
            Some(v) => {
                v.to_owned().push(vote);
                v.to_owned()
            }
            None => vec![vote],
        };

        let result = &self.block_votes.insert(*block_id, all_votes);
        if result.is_none() {
            panic!("something went wrong")
        }
    }
}

type Responder<T> = oneshot::Sender<Result<T>>;

#[derive(Debug)]
pub enum Command {
    ProcessBlock {
        block: pog_proto::api::SignedBlock,
        vote: u64,
        resp: Responder<()>,
    },
    ProcessVote {
        block: pog_proto::api::SignedBlock,
        vote: u64,
        resp: Responder<()>,
    },
    GetQueueSize {
        resp: Responder<u64>,
    },
}
