use super::shared::Command;
use anyhow::{Context, Result};
use pog_proto::api::SignedBlock;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
pub struct BlockpoolClient {
    tx: mpsc::Sender<Command>,
}

impl BlockpoolClient {
    pub fn new(tx: mpsc::Sender<Command>) -> Self {
        Self {
            tx,
        }
    }

    pub async fn send_command<F, T>(&self, command: F) -> Result<T>
    where
        F: FnOnce(oneshot::Sender<Result<T, anyhow::Error>>) -> Command,
    {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.tx.send(command(resp_tx)).await.with_context(|| "error sending process request")?;
        resp_rx.await?
    }

    pub async fn process_final_vote(&self, block: pog_proto::api::RawBlock, _vote: u64) -> Result<()> {
        let block: SignedBlock = block.try_into()?;

        self.send_command(|resp| Command::ProcessFinalVote {
            block,
            resp,
        })
        .await
    }

    pub async fn process_vote_proposal(&self, block: pog_proto::api::RawBlock, _vote: u64) -> Result<()> {
        let block: SignedBlock = block.try_into()?;

        self.send_command(|resp| Command::ProcessVoteProposal {
            block,
            resp,
        })
        .await
    }

    pub async fn get_queue_size(&self) -> Result<u64> {
        self.send_command(|resp| Command::GetQueueSize {
            resp,
        })
        .await
    }

    /// Gets the total voting power in the network
    pub fn get_total_network_power(&self) -> f64 {
        //TODO: Get all voting power of all prime delegates combined
        100_000_000_f64
    }
}
