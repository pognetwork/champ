use anyhow::{anyhow, Result};

use crate::state::ChampStateMutex;
use std::f64::consts::E;

// so we can normalize the curves
const NORMALIZED_MAX_POW: f64 = 10.0;
const TX_CURVE_MAX: i32 = 15;
const PLATEAU_SIZE: f64 = 350.0;
const HEIGHT_WEIGHT: f64 = 0.5;
const BALANCE_WEIGHT: f64 = 0.5;

#[allow(dead_code)]
/// Returns ACTUAL voting power and actual voting power
pub async fn calculate_voting_power(state: ChampStateMutex, account_id: String) -> Result<u64> {
    let db = &state.lock().await.db;

    let block = db.get_latest_block_by_account(&account_id).await?;
    let data = block.data.as_ref().ok_or(anyhow!("block data not found"))?;

    let first_block = db.get_block_by_height(&account_id, &0).await?;

    // 86400 is a week in seconds
    let time_interval = if block.timestamp - first_block.timestamp < 86400 {
        86400.0
    } else {
        (block.timestamp - first_block.timestamp) as f64
    };

    // base function that is an Asymptote with Balance
    // where x is a balance type (tbd)
    // This tends towards 10, as the balance increases
    let balance = data.balance as f64;
    let bresult = NORMALIZED_MAX_POW / (1.0+E.powf((-balance/5.0) + NORMALIZED_MAX_POW));

    // Interactions
    // where x is the nr of tx based on the account life in weeks
    // https://www.geogebra.org/calculator/ymkv5ew6
    // TODO: dont go back 10 years acc age, only go back 1 month
    let tx_per_week = (time_interval / data.height as f64) / 86400.0;
    // this is between 0 and 1 where plateau starts at 0.5
    let graph_result = 1.0 / (tx_per_week/(PLATEAU_SIZE/2.0) - 1.0).powi(2 * TX_CURVE_MAX) + 1.0;
    let hresult = graph_result * 10.0;

    let result = hresult * HEIGHT_WEIGHT + bresult * BALANCE_WEIGHT;
    // Account Age
    // Age / NrOfTx to give ratio to avoid spamming

    // Green Adresses?

    // Delegates 
    
    Ok(result as u64)
}