use anyhow::{anyhow, Result};
use pog_proto::api::Block;

use crate::state::ChampStateMutex;
use std::f64::consts::E;

// so we can normalize the curves
const NORMALIZED_MAX_POW: f64 = 10.0;
const TX_CURVE_MAX: i32 = 15;
const PLATEAU_SIZE: f64 = 350.0;
const HEIGHT_WEIGHT: f64 = 1.0;
const BALANCE_WEIGHT: f64 = 1.0;
const AGE_WEIGHT: f64 = 1.0;

// Month in Seconds
const LOOKBACK_RANGE: u64 = 60 * 60 * 24 * 30;
// 2 Months in Seconds
const MAX_LOOKBACK_RANGE: u64 = 60 * 60 * 24 * 30 * 2;

const WEEK_IN_SECONDS: f64 = 60.0 * 60.0 * 24.0 * 7.0;

#[allow(dead_code)]
/// Returns ACTUAL voting power
pub async fn calculate_voting_power(state: &ChampStateMutex, account_id: String) -> Result<u32> {
    let db = &state.lock().await.db;

    let block = db.get_latest_block_by_account(&account_id).await?;
    let data = block.data.as_ref().ok_or(anyhow!("block data not found"))?;

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

    let bresult = balance_graph(data.balance);
    let hresult = tx_graph(data.height, block, old_block_result);
    let aresult = age_graph(block.timestamp - first_block.timestamp);

    // TODO: Green Adresses?

    // TODO: Delegates

    // Weights to change how much impact each factor should have
    let result = hresult * HEIGHT_WEIGHT + bresult * BALANCE_WEIGHT + aresult * AGE_WEIGHT;

    Ok(result as u32)
}

fn balance_graph(balance: u64) -> f64 {
    // base function that is an Asymptote with Balance
    // where x is a balance type (tbd)
    // This tends towards 10, as the balance increases
    NORMALIZED_MAX_POW / (1.0 + E.powf((balance as f64 / -5.0) + NORMALIZED_MAX_POW))
}

fn tx_graph(block_height: u64, new_block: &Block, old_block: Option<&Block>) -> f64 {
    let old_block_time = match old_block {
        Some(b) => b.timestamp,
        None => new_block.timestamp,
    };
    // to get the time between the first and most recent block
    // we need the minimum to not give too high power from the start
    let time = if new_block.timestamp - old_block_time < WEEK_IN_SECONDS as u64 {
        WEEK_IN_SECONDS
    } else {
        (new_block.timestamp - old_block_time) as f64
    };

    // x is the nr of tx based on the account life in weeks
    // https://www.geogebra.org/calculator/ymkv5ew6
    let tx_per_week = (time / block_height as f64) / WEEK_IN_SECONDS;
    // this is between 0 and 1 where plateau starts at 0.5
    let graph_result = 1.0 / (tx_per_week / (PLATEAU_SIZE / 2.0) - 1.0).powi(2 * TX_CURVE_MAX) + 1.0;
    // to normalize tx graph and balanc graph
    graph_result * 10.0
}

fn age_graph(account_age: u64) -> f64 {
    // x is the account age in weeks
    // starts with negative power but increases at around 1 month
    // slowly increases steadily
    // x + 1 to avoid log0
    // 0.1x + 3 to allow the graph to go through 31 (month ish)
    // - 4 to shift the start
    let account_age_weeks = (account_age as f64 / WEEK_IN_SECONDS).floor();
    (account_age_weeks + 1.0).log10() + (0.1 * account_age_weeks + 3.0).sqrt() - 4.0
}
