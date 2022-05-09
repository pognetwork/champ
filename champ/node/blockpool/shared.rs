use anyhow::Result;
use tokio::sync::oneshot;

type Responder<T> = oneshot::Sender<Result<T>>;

#[derive(Debug)]
pub enum Command {
    // input
    ProcessVoteProposal {
        block: pog_proto::api::SignedBlock,
        resp: Responder<()>,
    },
    ProcessFinalVote {
        block: pog_proto::api::SignedBlock,
        resp: Responder<()>,
    },

    // output
    GetQueueSize {
        resp: Responder<u64>,
    },
}
