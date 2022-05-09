use anyhow::{anyhow, Result};
use libp2p::PeerId;
use pog_proto::p2p::request_body;

use crate::p2p::types::RequestBodyData;
use crate::{
    consensus::voting_power,
    p2p::server::{P2PServer, RequestResponse},
};

pub async fn process_vote_proposal(
    server: &mut P2PServer,
    data: request_body::VoteProposal,
    _peer_id: PeerId,
) -> Result<()> {
    // if prime delegate: cast own vote and send to all other prime delegates and 10 non prime delegates
    let raw_block = match data.block {
        Some(block) => block,
        None => return Err(anyhow!("block was none")),
    };

    if server.state.blockpool_client.process_vote(raw_block.clone(), data.vote, false).await.is_err() {
        return Err(anyhow!("error during processing vote"));
    }

    //TODO: Only add vote if this is a prime delegate
    let data = request_body::VoteProposal {
        block: Some(raw_block),
        vote: voting_power::get_active_power(&server.state, server.node_wallet.account_address_bytes).await?,
    };

    //TODO: If quorum has been reached, send FinalVote
    let _ = server.standard_send(RequestBodyData::VoteProposal(data));

    Ok(())
}

pub fn process_final_vote(_server: &mut P2PServer, _data: request_body::FinalVote, _peer_id: PeerId) -> Result<()> {
    unimplemented!("For now, we're just implementing vote proposals and will find a way to make this more generic for final votes in the future");
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

    // Ok(())
}
