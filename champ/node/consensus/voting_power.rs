use anyhow::{anyhow, Result};

use crate::state::ChampStateMutex;

/// Returns ACTUAL voting power and actual voting power
pub async fn calculate_voting_power(state: ChampStateMutex, account_id: String) -> Result<u64> {
    let db = &state.lock().await.db;

    let block = db.get_latest_block_by_account(&account_id).await?;
    let data = block.data.as_ref().ok_or(anyhow!("block data not found"))?;

    let balance = data.balance;
    

    // base function that is an Asymptote with Balance
    // 10 / (1 + e^[-(x/5) + 10]) where importance < 0 and balance < 0
    // where x is a balance ratio
    // This tends towards 10, as the balance increases

    // Interactions
    //-|1/(x+1)| + 1 where importance < 0 and nr of trx < 0
    // where x is the nr of trx
    // this tends towards 1 as the nr of trx increases

    // Account Age
    // Age / NrOfTrx to give ratio to avoid spamming

    // Green Adresses?

    // Delegates 
    
    Ok(0)
}