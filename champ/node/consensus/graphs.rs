use pog_proto::api::SignedBlock;

// so we can normalize the curves
const TX_CURVE_MAX: i32 = 15;
const PLATEAU_SIZE: f64 = 350.0;

const NORMALIZE_BALANCE: f64 = 1.0;
const NORMALIZE_CASHFLOW: f64 = 1.0;
const NORMALIZE_INACTIVE_TAX: f64 = 5.0;
const NORMALIZE_BLOCK: f64 = 10.0;

const WEEK_IN_SECONDS: f64 = 60.0 * 60.0 * 24.0 * 7.0;

pub fn balance_graph(balance: u64) -> f64 {
    balance as f64 / NORMALIZE_BALANCE
}

pub fn cashflow_graph(new_block_balance: u64, old_block_balance: u64) -> f64 {
    let cashflow = new_block_balance as i128 - old_block_balance as i128;

    -cashflow as f64 / NORMALIZE_CASHFLOW
}

pub fn inactive_tax_graph(new_block_balance: u64, old_block_balance: u64) -> f64 {
    let cashflow = new_block_balance as i128 - old_block_balance as i128;

    tracing::trace!("real cashflow={}", cashflow);
    // Inactive Tax
    if cashflow == 0 && new_block_balance > 0 {
        return -(new_block_balance as f64 / NORMALIZE_INACTIVE_TAX);
    }
    0.0
}

pub fn block_graph(block_height: u64, new_block: &SignedBlock, old_block: Option<&SignedBlock>) -> f64 {
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
    graph_result * NORMALIZE_BLOCK
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

#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;
    use pog_proto::api::{signed_block::BlockData, SignedBlock};

    use crate::consensus::graphs::{age_graph, balance_graph, block_graph, cashflow_graph, inactive_tax_graph};

    #[test]
    fn test_balance_graph() {
        assert_eq!(1000.0, balance_graph(1000));
        assert_eq!(5.0, balance_graph(5));
    }
    #[test]
    fn test_cashflow_graph() {
        assert_eq!(500.0, cashflow_graph(500, 1000));
        assert_eq!(-500.0, cashflow_graph(1000, 500));
        assert_eq!(0.0, cashflow_graph(1000, 1000));
    }
    #[test]
    fn test_inactive_tax_graph() {
        assert_eq!(0.0, inactive_tax_graph(500, 1000));
        assert_eq!(-200.0, inactive_tax_graph(1000, 1000));
    }
    #[test]
    fn test_block_graph() {
        let new_block = SignedBlock {
            signature: b"signature".to_vec(),
            public_key: b"public_key".to_vec(),
            timestamp: 100_000,
            data: Some(BlockData {
                version: 0,
                signature_type: 1,
                balance: 1000,
                height: 20,
                previous: b"previous".to_vec(),
                transactions: vec![],
            }),
        };
        let old_block = Some(SignedBlock {
            signature: b"signature".to_vec(),
            public_key: b"public_key".to_vec(),
            timestamp: 50_000,
            data: Some(BlockData {
                version: 0,
                signature_type: 1,
                balance: 500,
                height: 19,
                previous: b"other_previous".to_vec(),
                transactions: vec![],
            }),
        });
        assert_eq!(
            (20.17295623738592 * 100_000_f64) as u64,
            (block_graph(10, &new_block, old_block.as_ref()) * 100_000_f64) as u64
        );
    }
    #[test]
    fn test_age_graph() {
        assert_eq!((-2.267949192431123 * 100_000_f64) as u64, (age_graph(100_000) * 100_000_f64) as u64);
        assert_eq!((2.635988521203979 * 100_000_f64) as u64, (age_graph(100_000_000) * 100_000_f64) as u64);
    }
    #[test]
    fn test_snapshots() {
        assert_yaml_snapshot!(vec![
            balance_graph(1000).to_string(),
            balance_graph(0).to_string(),
            balance_graph(2500).to_string()
        ]);
        assert_yaml_snapshot!(vec![
            cashflow_graph(500, 1000).to_string(),
            cashflow_graph(1000, 1000).to_string(),
            cashflow_graph(0, 0).to_string()
        ]);
        assert_yaml_snapshot!(vec![
            inactive_tax_graph(1000, 1000).to_string(),
            inactive_tax_graph(0, 0).to_string(),
            inactive_tax_graph(1000, 1500).to_string()
        ]);
        assert_yaml_snapshot!(vec![
            age_graph(605000).to_string(),
            age_graph(100_000_000).to_string(),
            age_graph(0).to_string()
        ]);
    }
}
