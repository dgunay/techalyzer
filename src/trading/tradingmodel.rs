use crate::backtester::Position;
use crate::Date;
use crate::{error::TechalyzerError, marketdata::prices::Prices, util::TimeSeries};

use serde::Serialize;

#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Trades {
    pub trades: TimeSeries<Position>,
}

impl Trades {
    pub fn get(&self, k: &Date) -> Option<&Position> {
        self.trades.get(k)
    }
}

/// Given historical price data, comes up with a series of trades to attempt
/// to turn as much of a profit as possible.
pub trait TradingModel {
    /// Error type that can happen for our implementation of TradingModel.
    type Error: Into<TechalyzerError>;

    /// Given the Prices time series data, returns a Trades object.
    fn get_trades(self, prices: &Prices) -> Result<Trades, Self::Error>;
}
