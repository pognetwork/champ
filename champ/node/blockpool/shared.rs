use anyhow::Result;
use tokio::sync::oneshot;

type Responder<T> = oneshot::Sender<Result<T>>;

#[derive(Debug)]
pub enum Command {
    ProcessBlock {
        resp: Responder<()>,
    },
    ProcessVote {
        block: pog_proto::api::SignedBlock,
        vote: u64,
        final_vote: bool,
        resp: Responder<()>,
    },
    GetQueueSize {
        resp: Responder<u64>,
    },
}
