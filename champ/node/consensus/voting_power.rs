use anyhow::{anyhow, Result};
use tracing::{debug, trace};

use crate::consensus::graphs::*;
use crate::state::ChampStateArc;
use pog_proto::api;

// To balance each graph
const BLOCK_WEIGHT: f64 = 1.2;
const BALANCE_WEIGHT: f64 = 0.75;
const CASHFLOW_WEIGHT: f64 = 1.0;
const AGE_WEIGHT: f64 = 1.0;
const INACTIVE_TAX_WEIGHT: f64 = 1.0;

// so we can normalize the network affect
const MAX_NETWORK_POWER: f64 = 0.3;

// Month in Seconds
const LOOKBACK_RANGE: u64 = 60 * 60 * 24 * 30;
// 2 Months in Seconds
const MAX_LOOKBACK_RANGE: u64 = 60 * 60 * 24 * 30 * 2;

/// Returns actual voting power of an account.
/// Actual voting power is without the delegated power.
#[tracing::instrument]
pub async fn get_actual_power(state: &ChampStateArc, account_id: api::AccountID) -> Result<u64> {
    debug!("Calculating actual voting power");

    let db = &state.db.lock().await;
    let block = db.get_latest_block_by_account(account_id).await?;

    // Block from between lookback range and max lookback range
    let old_block_result = db
        .get_latest_block_by_account_before(
            account_id,
            block.header.timestamp - LOOKBACK_RANGE,
            block.header.timestamp - MAX_LOOKBACK_RANGE,
        )
        .await?;

    // First Block from an account
    let first_block = db.get_block_by_height(account_id, &0).await?.ok_or_else(|| anyhow!("no block found"))?;

    let new_block_balance = block.data.balance;
    let old_block_balance = old_block_result.clone().ok_or_else(|| anyhow!("block not found"))?.data.balance;

    let bresult = balance_graph(block.data.balance);
    let cresult = cashflow_graph(new_block_balance, old_block_balance);
    let bbresult = block_graph(block.data.height, &block, old_block_result.as_ref());
    let aresult = age_graph(block.header.timestamp - first_block.header.timestamp);

    // Weights to change how much impact each factor should have
    let net_result =
        bbresult * BLOCK_WEIGHT + bresult * BALANCE_WEIGHT + aresult * AGE_WEIGHT + cresult * CASHFLOW_WEIGHT;

    let iresult = inactive_tax_graph(new_block_balance, old_block_balance, net_result);

    trace!("Graph results: balance={0}, cashflow={1}, block={2}, age={3}", bresult, cresult, bbresult, aresult);
    // TODO: Green Adresses?

    let graph_result = net_result + iresult * INACTIVE_TAX_WEIGHT;

    let result = if graph_result < 0.0 {
        0
    } else {
        graph_result as u64
    };

    trace!("total actual voting power result: {}", result);

    Ok(result)
}

/// Returns the active power of an account that is being used on the network.
/// Active power is the account power with the delegated power.
#[tracing::instrument]
pub async fn get_active_power(state: &ChampStateArc, account_id: api::AccountID) -> Result<u64> {
    debug!("Calculating actual voting power");
    let actual_power = get_actual_power(state, account_id).await?;
    let delegate_power = get_delegated_power(state, account_id).await?;
    // get max voting power in the network (all nodes combined)
    let total_network_power = state.blockpool_client.clone().get_total_network_power();
    // a single node can only have a percentage of the max network power (current 30% but this will change)
    let total_allowed_voting_power = (total_network_power * MAX_NETWORK_POWER) as u64;
    let total_power = actual_power + delegate_power;
    if total_power > total_allowed_voting_power {
        return Ok(total_allowed_voting_power);
    }
    trace!("total active voting power result: {}", total_power);
    Ok(total_power)
}

/// Gets the sum of the power of each delegate of an account
async fn get_delegated_power(state: &ChampStateArc, account_id: api::AccountID) -> Result<u64> {
    debug!("calculating delegated power");
    // TODO: Cache this
    let mut power = 0;
    let db = &state.db.lock().await;

    let mut delegates = db.get_delegates_by_account(account_id).await?;
    // TODO: Test Performance and do this concurrently?
    while let Some(d) = delegates.pop() {
        let p = get_actual_power(state, d.to_owned()).await?;
        power += p;
    }

    trace!("total delegated voting power: {}", power);
    Ok(power)
}

#[cfg(test)]
mod tests {
    use crate::consensus::{
        graphs::{balance_graph, cashflow_graph},
        voting_power::BALANCE_WEIGHT,
        voting_power::CASHFLOW_WEIGHT,
    };
    use pog_proto::api::SignedBlock;
    use pog_proto::api::{BlockData, BlockHeader};
    #[test]
    fn check_voting_power() {
        let mut blocks: Vec<SignedBlock> = Vec::new();
        for (height, balance) in [1000, 900, 300, 0, 500, 400, 400, 600, 1000, 800].iter().enumerate() {
            blocks.push(SignedBlock::new(
                BlockHeader {
                    signature: b"signature".to_vec(),
                    public_key: b"key".to_vec(),
                    timestamp: 1,
                },
                BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: *balance as u64,
                    height: height as u64,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                },
            ))
        }

        let mut old_data = BlockData {
            version: 0,
            signature_type: 0,
            balance: 0,
            height: 0,
            previous: b"some".to_vec(),
            transactions: vec![],
        };
        println!("Old Balance - New Balance  -  Balance I  -  Cashflow I - Total I");
        for block in blocks {
            let balance_importance = balance_graph(block.data.balance) * BALANCE_WEIGHT;
            let cashflow_importance = cashflow_graph(block.data.balance, old_data.balance) * CASHFLOW_WEIGHT;
            let total_importance = balance_importance + cashflow_importance;
            println!(
                "{0} \t|----| {1} \t|----| {2} \t|----| {3} \t|----| {4}",
                old_data.balance, block.data.balance, balance_importance, cashflow_importance, total_importance
            );
            old_data = block.data;
        }
        // Switch on to output debug table
        // assert!(false);
    }
}
