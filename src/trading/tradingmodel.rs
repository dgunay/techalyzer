use crate::backtester::Position;
use crate::marketdata::prices::Prices;
use chrono::NaiveDate;
use std::collections::BTreeMap;

struct Trades {
    trades: BTreeMap<NaiveDate, Position>,
}

/// Given historical price data, comes up with a series of trades to attempt
/// to turn as much of a profit as possible.
trait TradingModel {
    fn get_trades(prices: &Prices) -> Trades;
}
