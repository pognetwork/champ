use pog_proto::api::Block;
use std::f64::consts::E;

// so we can normalize the curves
const NORMALIZED_MAX_POW: f64 = 10.0;
const TX_CURVE_MAX: i32 = 15;
const PLATEAU_SIZE: f64 = 350.0;

const WEEK_IN_SECONDS: f64 = 60.0 * 60.0 * 24.0 * 7.0;

pub fn balance_graph(balance: u64) -> f64 {
    // base function that is an Asymptote with Balance
    // where x is a balance type (tbd)
    // This tends towards 10, as the balance increases
    NORMALIZED_MAX_POW / (1.0 + E.powf((balance as f64 / -5.0) + NORMALIZED_MAX_POW))
}

pub fn block_graph(block_height: u64, new_block: &Block, old_block: Option<&Block>) -> f64 {
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
    let blocks_per_week = (time / block_height as f64) / WEEK_IN_SECONDS;
    // this is between 0 and 1 where plateau starts at 0.5
    let graph_result = 1.0 / (blocks_per_week / (PLATEAU_SIZE / 2.0) - 1.0).powi(2 * TX_CURVE_MAX) + 1.0;
    // to normalize tx graph and balance graph
    graph_result * 10.0
}

pub fn age_graph(account_age: u64) -> f64 {
    // x is the account age in weeks
    // starts with negative power but increases at around 1 month
    // slowly increases steadily
    // x + 1 to avoid log0
    // 0.1x + 3 to allow the graph to go through 31 (month ish)
    // - 4 to shift the start
    let account_age_weeks = (account_age as f64 / WEEK_IN_SECONDS).floor();
    (account_age_weeks + 1.0).log10() + (0.1 * account_age_weeks + 3.0).sqrt() - 4.0
}
