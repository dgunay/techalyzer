use crate::backtester::Position;
use crate::{error::TechalyzerError, marketdata::prices::Prices};
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize, Clone)]
pub struct Trades {
    pub trades: BTreeMap<NaiveDate, Position>,
}

impl Trades {
    pub fn get(&self, k: &NaiveDate) -> Option<&Position> {
        self.trades.get(k)
    }
}

/// Given historical price data, comes up with a series of trades to attempt
/// to turn as much of a profit as possible.
pub trait TradingModel {
    /// Error type that can happen for our implementation of TradingModel.
    type Error: Into<TechalyzerError>;

    fn get_trades(self, prices: &Prices) -> Result<Trades, Self::Error>;
}
