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
pub async fn get_actual_power(state: &ChampStateArc, account_id: api::AccountID) -> Result<u32> {
    debug!("Calculating actual voting power");

    let db = &state.db.lock().await;

    let block = db.get_latest_block_by_account(account_id).await?;
    let data = block.data.as_ref().ok_or_else(|| anyhow!("block data not found"))?;

    // Block from between lookback range and max lookback range
    let old_block_result = db
        .get_latest_block_by_account_before(
            account_id,
            block.timestamp - LOOKBACK_RANGE,
            block.timestamp - MAX_LOOKBACK_RANGE,
        )
        .await?;

    // First Block from an account
    let first_block = db.get_block_by_height(account_id, &0).await?.ok_or_else(|| anyhow!("no block found"))?;

    let new_block_balance = data.balance;
    let old_block_balance = old_block_result
        .clone()
        .ok_or_else(|| anyhow!("block not found"))?
        .data
        .ok_or_else(|| anyhow!("block data not found"))?
        .balance;

    let bresult = balance_graph(data.balance);
    let cresult = cashflow_graph(new_block_balance, old_block_balance);
    let bbresult = block_graph(data.height, &block, old_block_result.as_ref());
    let aresult = age_graph(block.timestamp - first_block.timestamp);
    let iresult = inactive_tax_graph(new_block_balance, old_block_balance);

    trace!("Graph results: balance={0}, cashflow={1}, block={2}, age={3}", bresult, cresult, bbresult, aresult);
    // TODO: Green Adresses?

    // Weights to change how much impact each factor should have
    let graph_result = bbresult * BLOCK_WEIGHT
        + bresult * BALANCE_WEIGHT
        + aresult * AGE_WEIGHT
        + cresult * CASHFLOW_WEIGHT
        + iresult * INACTIVE_TAX_WEIGHT;

    let result = if graph_result < 0.0 {
        0
    } else {
        graph_result as u32
    };

    trace!("total actual voting power result: {}", result);

    Ok(result)
}

/// Returns the active power of an account that is being used on the network.
/// Active power is the account power with the delegated power.
#[tracing::instrument]
pub async fn get_active_power(state: &ChampStateArc, account_id: api::AccountID) -> Result<u32> {
    debug!("Calculating actual voting power");
    let actual_power = get_actual_power(state, account_id).await?;
    let delegate_power = get_delegated_power(state, account_id).await?;
    let total_network_power = get_max_voting_power();
    let total_power = actual_power + delegate_power;
    if total_power > total_network_power {
        return Ok(total_network_power);
    }
    trace!("total active voting power result: {}", total_power);
    Ok(total_power)
}

/// Gets the sum of the power of each delegate of an account
async fn get_delegated_power(state: &ChampStateArc, account_id: api::AccountID) -> Result<u32> {
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

/// Gets the max voting power in the system and sets a limit of a percentage
fn get_max_voting_power() -> u32 {
    //TODO: Get all voting power of all prime delegates combined
    let total_prime_delegate_power = 100_000_000_f64;
    (total_prime_delegate_power * MAX_NETWORK_POWER) as u32
}

#[cfg(test)]
mod tests {
    use crate::consensus::{
        graphs::{balance_graph, cashflow_graph},
        voting_power::BALANCE_WEIGHT,
        voting_power::CASHFLOW_WEIGHT,
    };
    use pog_proto::api::signed_block::BlockData;
    use pog_proto::api::SignedBlock;
    #[test]
    fn check_voting_power() {
        // Switch on to output debug table
        const TEST_TABLE_ON: bool = false;

        let blocks = vec![
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 1000,
                    height: 1,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 900,
                    height: 2,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 300,
                    height: 3,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 0,
                    height: 4,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 500,
                    height: 5,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 400,
                    height: 6,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 400,
                    height: 7,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 600,
                    height: 8,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 1000,
                    height: 9,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
            SignedBlock {
                signature: b"signature".to_vec(),
                public_key: b"key".to_vec(),
                timestamp: 1,
                data: Some(BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 900,
                    height: 10,
                    previous: b"previous".to_vec(),
                    transactions: vec![],
                }),
            },
        ];
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
            let new_data = match block.data {
                Some(d) => d,
                None => BlockData {
                    version: 0,
                    signature_type: 0,
                    balance: 0,
                    height: 0,
                    previous: b"some".to_vec(),
                    transactions: vec![],
                },
            };
            let balance_importance = balance_graph(new_data.balance) * BALANCE_WEIGHT;
            let cashflow_importance = cashflow_graph(new_data.balance, old_data.balance) * CASHFLOW_WEIGHT;
            let total_importance = balance_importance + cashflow_importance;
            println!(
                "{0} \t|----| {1} \t|----| {2} \t|----| {3} \t|----| {4}",
                old_data.balance, new_data.balance, balance_importance, cashflow_importance, total_importance
            );
            old_data = new_data;
        }
        assert!(!TEST_TABLE_ON);
    }
}
