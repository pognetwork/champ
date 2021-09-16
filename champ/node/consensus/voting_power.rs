use anyhow::{anyhow, Result};
use pog_proto::api::Block;

use crate::state::ChampStateMutex;
use std::f64::consts::E;

// so we can normalize the curves
const NORMALIZED_MAX_POW: f64 = 10.0;
const TX_CURVE_MAX: i32 = 15;
const PLATEAU_SIZE: f64 = 350.0;
const HEIGHT_WEIGHT: f64 = 0.5;
const BALANCE_WEIGHT: f64 = 0.5;

const WEEK_IN_SECONDS: f64 = 86400.0;

#[allow(dead_code)]
/// Returns ACTUAL voting power
pub async fn calculate_voting_power(state: &ChampStateMutex, account_id: String) -> Result<u32> {
    let db = &state.lock().await.db;

    let block = db.get_latest_block_by_account(&account_id).await?;
    let data = block.data.as_ref().ok_or(anyhow!("block data not found"))?;

    let first_block = db.get_block_by_height(&account_id, &0).await?;

    let bresult = balance_graph(data.balance as f64);
    let hresult = tx_graph(data.height as f64, block, first_block);

    // TODO: Account Age

    // TODO: Green Adresses?

    // TODO: Delegates 

    // Weights to change how much impact each factor should have
    let result = hresult * HEIGHT_WEIGHT + bresult * BALANCE_WEIGHT;

    Ok(result as u32)
}

pub fn balance_graph(balance: f64) -> f64 {
    // base function that is an Asymptote with Balance
    // where x is a balance type (tbd)
    // This tends towards 10, as the balance increases
    NORMALIZED_MAX_POW / (1.0+E.powf((-balance/5.0) + NORMALIZED_MAX_POW))
}

pub fn tx_graph(block_height: f64, new_block: &Block, old_block: &Block) -> f64 {
    // TODO: dont go back 10 years acc age, only go back 1 month

    // to get the time between the first and most recent block 
    // we need the minimum to not give too high power from the start
    let time = if new_block.timestamp - old_block.timestamp < WEEK_IN_SECONDS as u64 {
        WEEK_IN_SECONDS
    } else {
        (new_block.timestamp - old_block.timestamp) as f64
    };

    // x is the nr of tx based on the account life in weeks
    // https://www.geogebra.org/calculator/ymkv5ew6
    let tx_per_week = (time / block_height) / WEEK_IN_SECONDS;
    // this is between 0 and 1 where plateau starts at 0.5
    let graph_result = 1.0 / (tx_per_week/(PLATEAU_SIZE/2.0) - 1.0).powi(2 * TX_CURVE_MAX) + 1.0;
    // to normalize tx graph and balanc graph
    graph_result * 10.0
}