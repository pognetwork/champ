use anyhow::Result;
use libp2p::PeerId;
use pog_proto::p2p::request_body;

use crate::p2p::server::P2PServer;

pub fn process_vote_proposal(
    _server: &mut P2PServer,
    _data: request_body::VoteProposal,
    peer_id: PeerId,
) -> Result<()> {
    print!("got a forwarded message from {peer_id:?}");

    Ok(())
}

pub fn process_final_vote(_server: &mut P2PServer, _data: request_body::FinalVote, peer_id: PeerId) -> Result<()> {
    print!("got a forwarded message from {peer_id:?}");

    Ok(())
}

// #[tokio::main]
// async fn process_final_vote(&mut self, data: FinalVote) -> Result<()> {
//     // all nodes who calculate a 60% quorum from the vote proposal need to send a final vote
//     let raw_block = match data.block {
//         Some(block) => block,
//         None => return Err(anyhow!("block was none")),
//     };

//     if self.state.blockpool_client.process_vote(raw_block.clone(), data.vote, true).await.is_err() {
//         return Err(anyhow!("error during processing vote"));
//     }

//     let data = request_body::FinalVote {
//         block: Some(raw_block),
//         vote: voting_power::get_active_power(&self.state, self.primary_wallet.account_address_bytes).await?,
//     };

//     self.standard_send(RequestBodyData::FinalVote(data))
// }

// #[tokio::main]
// async fn process_vote_proposal(&mut self, data: VoteProposal) -> Result<()> {
//     // if prime delegate: cast own vote and send to all other prime delegates and 10 non prime delegates
//     let raw_block = match data.block {
//         Some(block) => block,
//         None => return Err(anyhow!("block was none")),
//     };

//     if self.state.blockpool_client.process_vote(raw_block.clone(), data.vote, false).await.is_err() {
//         return Err(anyhow!("error during processing vote"));
//     }

//     //TODO: Only add vote if this is a prime delegate
//     let data = request_body::VoteProposal {
//         block: Some(raw_block),
//         vote: voting_power::get_active_power(&self.state, self.primary_wallet.account_address_bytes).await?,
//     };

//     //TODO: If quorum has been reached, send FinalVote
//     self.standard_send(RequestBodyData::VoteProposal(data))
// }
