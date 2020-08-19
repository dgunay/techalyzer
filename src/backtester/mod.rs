//! The BackTester runs backtests for a given set of Trades on a Prices time
//! series. It returns a [PortfolioPerformance](performance/struct.PortfolioPerformance.html)
//! containing portfolio value over time as well as other statistics.

pub mod performance;

use crate::trading::tradingmodel::Trades;

use crate::date::Date;
use crate::marketdata::prices::Prices;
use crate::trading::Position;
use derive_more::Display;
use performance::{PerformanceError, PortfolioPerformance};
use std::collections::BTreeMap;

/// Errors that can occur while running a backtest.
#[derive(Debug, Display)]
pub enum BackTesterError {
    /// No position found for this trading day
    #[display(fmt = "No Position/trade could be found on date {}", _0)]
    NoPositionFound(Date),
}

/// Backtests a strategy given as a map of Date => Trade
pub struct BackTester<'a> {
    /// What trade to execute on each day
    trades: Trades,

    /// price time series data
    prices: &'a Prices,

    /// How much cash the portfolio starts with/has currently
    cash: f64,

    current_position: Position,
    current_shares: i32,
}

impl<'a> BackTester<'a> {
    /// Constructs a BackTester. There must be a Position in `trades` for every
    /// day in Prices.
    pub fn new(trades: Trades, prices: &'a Prices, cash: f64) -> Result<Self, BackTesterError> {
        // For every day in the time series, there must be some Position.
        for day in prices.map.keys() {
            if trades.get(day).is_none() {
                return Err(BackTesterError::NoPositionFound(*day));
            }
        }

        Ok(Self {
            cash,
            prices,
            trades,
            current_position: Position::Out,
            current_shares: 0,
        })
    }

    // TODO: can we do backtesting immutably?
    /// Runs the backtest and returns portfolio value at each day of the period.
    pub fn backtest(&mut self) -> Result<PortfolioPerformance, PerformanceError> {
        let mut portvals = BTreeMap::new();

        // For every day in the series
        for (day, price) in self.prices.map.iter() {
            // Execute this position
            let mut trade = self.trades.get(day).cloned().unwrap();
            let cash_difference = BackTester::do_trade(self.current_shares, price, &trade);
            self.cash += cash_difference;

            if trade == Position::Hold {
                trade = self.current_position.clone();
            } else {
                self.current_position = trade.clone();
            }

            // Calculate portfolio value (equity plus cash)
            let equity_value = match trade {
                Position::Long(shares) => {
                    self.current_shares = shares as i32;
                    shares as f64 * *price
                }
                Position::Short(shares) => {
                    self.current_shares = -(shares as i32);
                    -(shares as f64) * *price
                }
                Position::Out => {
                    self.current_shares = 0;
                    0.0
                }
                Position::Hold => unreachable!(), // FIXME: dead code
            };

            // Store
            portvals.insert(*day, equity_value + self.cash);
        }

        PortfolioPerformance::new(portvals)
    }

    /// Returns cash difference from making a trade. Buying stocks costs money,
    /// shorting stocks or selling them results in a positive credit.
    fn do_trade(current_shares: i32, price: &f64, trade: &Position) -> f64 {
        // We must implicitly exit whatever trade we are currently in (e.g.
        // going long to short involves selling the shares first). We can do
        // this by subtracting our current shares from the shares in the trade.
        let shares: i32 = match trade {
            // Long trades are positive shares
            Position::Long(s) => *s as i32 - current_shares,
            // Short trades are negative shares
            Position::Short(s) => -(*s as i32) - current_shares,
            // Return to zero shares
            Position::Out => -current_shares,
            // do nothing
            Position::Hold => 0,
        };

        // For long (positive) shares, we lose cash. For short (negative)
        // shares, we gain cash.
        -(shares as f64 * price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{nearly_equal, TimeSeries};

    // TODO: less copypasted code for fixtures

    #[test]
    fn buy_and_hold_backtest() {
        let day1 = Date::from_ymd(2012, 1, 1);
        let day2 = Date::from_ymd(2012, 1, 2);
        let day3 = Date::from_ymd(2012, 1, 3);

        // Buy and hold 1000 shares for the duration of a few days
        let strat: TimeSeries<Position> = vec![
            (day1, Position::Long(1)),
            (day2, Position::Hold),
            (day3, Position::Hold),
        ]
        .iter()
        .cloned()
        .collect();

        let prices: TimeSeries<f64> = vec![(day1, 100.0), (day2, 105.0), (day3, 110.0)]
            .iter()
            .cloned()
            .collect();

        let p = Prices {
            map: prices,
            symbol: "TLZR".to_string(),
        };
        let mut bt = BackTester::new(Trades { trades: strat }, &p, 100.0).unwrap();

        let result = bt.backtest().unwrap();
        assert!(nearly_equal(result.daily_portvals[&day1], 100.0));
        assert!(nearly_equal(result.daily_portvals[&day2], 105.0));
        assert!(nearly_equal(result.daily_portvals[&day3], 110.0));
    }

    #[test]
    fn partial_sell_backtest() {
        let day1 = Date::from_ymd(2012, 1, 1);
        let day2 = Date::from_ymd(2012, 1, 2);
        let day3 = Date::from_ymd(2012, 1, 3);

        // Buy 2 shares, sell 1, hold the other.
        let strat: TimeSeries<Position> = vec![
            (day1, Position::Long(2)),
            (day2, Position::Long(1)),
            (day3, Position::Hold),
        ]
        .iter()
        .cloned()
        .collect();

        let prices: TimeSeries<f64> = vec![(day1, 100.0), (day2, 105.0), (day3, 110.0)]
            .iter()
            .cloned()
            .collect();

        let p = Prices {
            map: prices,
            symbol: "TLZR".to_string(),
        };
        let mut bt = BackTester::new(Trades { trades: strat }, &p, 200.0).unwrap();

        let result = bt.backtest().unwrap();
        // let debug: Vec<f64> = result.daily_portvals.values().into_iter().cloned().collect();
        assert!(nearly_equal(result.daily_portvals[&day1], 200.0));
        assert!(nearly_equal(result.daily_portvals[&day2], 210.0));
        assert!(nearly_equal(result.daily_portvals[&day3], 215.0));
    }

    #[test]
    fn short_and_hold_backtest() {
        let day1 = Date::from_ymd(2012, 1, 1);
        let day2 = Date::from_ymd(2012, 1, 2);
        let day3 = Date::from_ymd(2012, 1, 3);

        // Buy and hold 1000 shares for the duration of a few days
        let strat: TimeSeries<Position> = vec![
            (day1, Position::Short(1)),
            (day2, Position::Hold),
            (day3, Position::Hold),
        ]
        .iter()
        .cloned()
        .collect();

        let prices: TimeSeries<f64> = vec![(day1, 100.0), (day2, 105.0), (day3, 110.0)]
            .iter()
            .cloned()
            .collect();

        let p = Prices {
            map: prices,
            symbol: "TLZR".to_string(),
        };
        let mut bt = BackTester::new(Trades { trades: strat }, &p, 100.0).unwrap();

        let result = bt.backtest().unwrap();
        assert!(nearly_equal(result.daily_portvals[&day1], 100.0));
        assert!(nearly_equal(result.daily_portvals[&day2], 95.0));
        assert!(nearly_equal(result.daily_portvals[&day3], 90.0));
    }

    #[test]
    fn buy_then_short() {
        let day1 = Date::from_ymd(2012, 1, 1);
        let day2 = Date::from_ymd(2012, 1, 2);
        let day3 = Date::from_ymd(2012, 1, 3);
        let day4 = Date::from_ymd(2012, 1, 4);

        // Buy and hold 1000 shares for the duration of a few days
        let strat: TimeSeries<Position> = vec![
            (day1, Position::Long(1)),
            (day2, Position::Hold),
            (day3, Position::Short(1)),
            (day4, Position::Hold),
        ]
        .iter()
        .cloned()
        .collect();

        let prices: TimeSeries<f64> =
            vec![(day1, 100.0), (day2, 105.0), (day3, 110.0), (day4, 105.0)]
                .iter()
                .cloned()
                .collect();

        let p = Prices {
            map: prices,
            symbol: "TLZR".to_string(),
        };
        let mut bt = BackTester::new(Trades { trades: strat }, &p, 100.0).unwrap();

        let result = bt.backtest().unwrap();
        assert!(nearly_equal(result.daily_portvals[&day1], 100.0));
        assert!(nearly_equal(result.daily_portvals[&day2], 105.0));
        assert!(nearly_equal(result.daily_portvals[&day3], 110.0));
        assert!(nearly_equal(result.daily_portvals[&day4], 115.0));
    }

    #[test]
    fn buy_then_out() {
        let day1 = Date::from_ymd(2012, 1, 1);
        let day2 = Date::from_ymd(2012, 1, 2);
        let day3 = Date::from_ymd(2012, 1, 3);
        let day4 = Date::from_ymd(2012, 1, 4);

        // Buy and hold 1000 shares for the duration of a few days
        let strat: TimeSeries<Position> = vec![
            (day1, Position::Long(1)),
            (day2, Position::Hold),
            (day3, Position::Out),
            (day4, Position::Hold),
        ]
        .iter()
        .cloned()
        .collect();

        let prices: TimeSeries<f64> =
            vec![(day1, 100.0), (day2, 105.0), (day3, 110.0), (day4, 105.0)]
                .iter()
                .cloned()
                .collect();

        let p = Prices {
            map: prices,
            symbol: "TLZR".to_string(),
        };
        let mut bt = BackTester::new(Trades { trades: strat }, &p, 100.0).unwrap();

        let result = bt.backtest().unwrap();
        assert!(nearly_equal(result.daily_portvals[&day1], 100.0));
        assert!(nearly_equal(result.daily_portvals[&day2], 105.0));
        assert!(nearly_equal(result.daily_portvals[&day3], 110.0));
        assert!(nearly_equal(result.daily_portvals[&day4], 110.0));
    }
}
