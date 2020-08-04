//! TODO: document

use crate::Prices;
use chrono::NaiveDate;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum Position {
    Long,
    Short,
    Out,
}

/// A trade with the position (long/short/out) and number of shares commit to
/// the trade.
#[derive(Clone)]
struct Trade {
    position: Position,
    shares: u64,
}

/// Backtests a strategy given as a map of NaiveDate => Trade
struct BackTester {
    /// What trade to execute on each day
    strategy: BTreeMap<NaiveDate, Trade>,

    /// price time series data
    prices: Prices,

    /// How much cash the portfolio starts with
    cash: f64,
}

impl BackTester {
    /// Runs the backtest and returns portfolio value at each day of the period.
    pub fn backtest(&self) -> BTreeMap<NaiveDate, f64> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_backtest() {
        let strat: BTreeMap<NaiveDate, Trade> = vec![(
            NaiveDate::from_ymd(2012, 1, 1),
            Trade {
                position: Position::Long,
                shares: 1000,
            },
        )]
        .iter()
        .cloned()
        .collect();

        let bt = BackTester {
            // TODO: write a simplistic test for buy and hold over like, 3 days.
            strategy: strat,
            cash: 1000000.0,
            prices: todo!(),
        };
    }
}
