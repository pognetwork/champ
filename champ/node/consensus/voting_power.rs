use anyhow::{anyhow, Result};

use crate::consensus::graphs;
use crate::state::ChampStateMutex;

// To balance each graph
const HEIGHT_WEIGHT: f64 = 1.0;
const BALANCE_WEIGHT: f64 = 1.0;
const AGE_WEIGHT: f64 = 1.0;

// so we can normalize the network affect
const MAX_NETWORK_POWER: f64 = 0.3;

// Month in Seconds
const LOOKBACK_RANGE: u64 = 60 * 60 * 24 * 30;
// 2 Months in Seconds
const MAX_LOOKBACK_RANGE: u64 = 60 * 60 * 24 * 30 * 2;

/// Returns actual voting power of an account.
/// Actual voting power is without the delegated power.
pub async fn get_actual_power(state: &ChampStateMutex, account_id: String) -> Result<u32> {
    let db = &state.lock().await.db;

    let block = db.get_latest_block_by_account(&account_id).await?;
    let data = block.data.as_ref().ok_or_else(|| anyhow!("block data not found"))?;

    // Block from between lookback range and max lookback range
    let old_block_result = db
        .get_latest_block_by_account_before(
            &account_id,
            block.timestamp - LOOKBACK_RANGE,
            block.timestamp - MAX_LOOKBACK_RANGE,
        )
        .await?;

    // First Block from an account
    let first_block = db.get_block_by_height(&account_id, &0).await?;

    let bresult = graphs::balance_graph(data.balance);
    let hresult = graphs::tx_graph(data.height, block, old_block_result);
    let aresult = graphs::age_graph(block.timestamp - first_block.timestamp);

    // TODO: Green Adresses?

    // TODO: Delegates

    // Weights to change how much impact each factor should have
    let result = hresult * HEIGHT_WEIGHT + bresult * BALANCE_WEIGHT + aresult * AGE_WEIGHT;

    Ok(result as u32)
}

#[allow(dead_code)]
/// Gets the sum of the power of each delegate of an account
async fn get_delegated_power(state: &ChampStateMutex, account_id: String) -> Result<u32> {
    // TODO: Cache this
    let mut power = 0;
    let db = &state.lock().await.db;

    let mut delegates = db.get_delegates_by_account(&account_id).await?;
    // TODO: Test Performance and do this concurrently?
    while let Some(d) = delegates.pop() {
        let p = get_actual_power(state, d.to_owned()).await?;
        power += p;
    }

    Ok(power)
}

#[allow(dead_code)]
/// Returns the active power of an account that is being used on the network.
/// Active power is the account power with the delegated power.
pub async fn get_active_power(state: &ChampStateMutex, account_id: String) -> Result<u32> {
    let actual_power = get_actual_power(state, account_id.clone()).await?;
    let delegate_power = get_delegated_power(state, account_id.to_owned()).await?;
    let total_network_power = get_max_voting_power();
    let total_power = actual_power + delegate_power;
    if total_power > total_network_power {
        return Ok(total_network_power);
    }
    Ok(total_power)
}
/// Gets the max voting power in the system and sets a limit of a percentage
fn get_max_voting_power() -> u32 {
    //TODO: Get all voting power in the system
    (100_000_000_f64 * MAX_NETWORK_POWER) as u32
}
