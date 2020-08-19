//! Measures portfolio performance as total/daily returns over periods of time.

use super::Position;
use crate::util::{first_key, TimeSeries};
use crate::{trading::tradingmodel::Trades, Date};
use derive_more::Display;
use serde::Serialize;
use stats::stddev;
use std::{collections::BTreeMap, ops::RangeBounds};

/// Represents portfolio performance.
#[derive(Debug, Serialize)]
pub struct PortfolioPerformance {
    /// The running total portfolio value in a time series.
    pub daily_portvals: TimeSeries<f64>,

    // pub sharpe_ratio: f64, // TODO: add this
    /// The daily portfolio returns in a time series.
    pub daily_returns: TimeSeries<f64>,

    /// Standard deviation of daily returns.
    pub volatility: f64,
}

/// Errors that may occur during portfolio performance calculation.
#[derive(Debug, Serialize, Display)]
pub enum PerformanceError {
    /// Occurs if there is not at least one data point to measure.
    #[display(fmt = "Not enough data points to calculate performance")]
    NotEnoughDataPoints,

    /// Occurs if the given date is not in the performance period given when
    /// constructing the PortfolioPerformance.
    #[display(
        fmt = "Day {} not found in performance period while calculating trade accuracy",
        _0
    )]
    DayNotInPerformancePeriod(Date),
}

impl PortfolioPerformance {
    /// Constructs a PortfolioPerformance. There must be at least one datapoint
    /// in `daily_portvals`.
    pub fn new(daily_portvals: TimeSeries<f64>) -> Result<Self, PerformanceError> {
        // Calculate daily returns
        // TODO: this can be probably done more elegantly either with fold_first once
        // stabilized, or through a better pattern I'm not yet aware of. for
        // example:
        // daily_portvals.iter().fold_first(|yesterday, today| {
        //     let ret = (today.1 / yesterday.1) - 1.0;
        //     daily_returns.insert(today.0.clone(), ret);
        //     today
        // });
        let mut daily_returns = BTreeMap::new();
        let first_key = first_key(&daily_portvals).ok_or(PerformanceError::NotEnoughDataPoints)?;
        daily_returns.insert(*first_key, 0.0);
        let mut yesterday = daily_portvals[first_key];
        for (day, value) in daily_portvals.iter() {
            if day == first_key {
                continue;
            }
            let ret = (value / yesterday) - 1.0;
            daily_returns.insert(*day, ret);
            yesterday = *value;
        }

        let volatility = stddev(daily_returns.values().cloned());

        Ok(Self {
            daily_portvals,
            daily_returns,
            // sharpe_ratio: -1.0, // TODO:
            volatility,
        })
    }

    /// Returns a percent accuracy for trades (what percentage of them are
    /// profitable). For the purposes of this measurement, a "trade" is counted
    /// as opening a long or short position, and then exiting the trade either
    /// by going out or by entering an opposite position (long to short or short
    /// to long)
    pub fn trades_accuracy(&self, trades: &Trades) -> Result<f64, PerformanceError> {
        let mut last_entry = Position::Out;
        let mut entry_portval = 0.0;
        let mut total_trades = 0;
        let mut profitable_trades = 0;
        for (day, portval) in &self.daily_portvals {
            let current_trade = trades
                .get(&day)
                .ok_or(PerformanceError::DayNotInPerformancePeriod(*day))?;

            // if it's an exit, calculate profit or loss
            if current_trade.is_exit_from(last_entry) {
                let profit = (portval / entry_portval) - 1.0;
                if profit > 0.0 {
                    profitable_trades += 1;
                }

                // Sometimes incomplete trades happen, just don't count them.
                if profit != 0.0 {
                    total_trades += 1;
                }

                // Reset the entry marker
                last_entry = *current_trade;
            }

            // If this is an entry, mark it.
            if current_trade.is_entry() {
                // if it's an entry, record the entry portval
                entry_portval = *portval;
                last_entry = *current_trade;
            }
        }

        Ok(profitable_trades as f64 / total_trades as f64)
    }

    /// Returns the total return in the date range.
    pub fn range_return(&self, range: impl RangeBounds<Date>) -> Result<f64, PerformanceError> {
        let mut iter = self.daily_portvals.range(range);
        let first = iter.next().ok_or(PerformanceError::NotEnoughDataPoints)?.1;
        let last = iter.last().ok_or(PerformanceError::NotEnoughDataPoints)?.1;

        Ok((last / first) - 1.0)
    }

    /// Returns the total return for the whole time series of portfolio values.
    pub fn total_return(&self) -> Result<f64, PerformanceError> {
        self.range_return(..)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{backtester::Position as Pos, util::nearly_equal};

    #[test]
    fn total_return_divide_by_zero() {
        let fixture = vec![
            (Date::from_ymd(2020, 3, 1), 0.0, Pos::Long(1)),
            (Date::from_ymd(2020, 3, 2), 10.0, Pos::Hold),
        ];
        let (daily_portvals, _) = construct_fixture(&fixture);
        let pp = PortfolioPerformance::new(daily_portvals).unwrap();
        assert_eq!(pp.total_return().unwrap(), f64::INFINITY);
    }

    fn construct_fixture(data: &Vec<(Date, f64, Pos)>) -> (TimeSeries<f64>, Trades) {
        let mut portvals = TimeSeries::new();
        let mut trades = TimeSeries::new();
        for (day, val, position) in data.iter().cloned() {
            portvals.insert(day, val);
            trades.insert(day, position);
        }

        (portvals, Trades { trades })
    }

    #[test]
    fn daily_returns() {
        let day1 = Date::from_ymd(2020, 3, 1);
        let day2 = Date::from_ymd(2020, 3, 2);
        let data = vec![(day1, 10.0, Pos::Long(1)), (day2, 11.0, Pos::Out)];
        let (pv, _) = construct_fixture(&data);

        let pp = PortfolioPerformance::new(pv).unwrap();
        let daily_rets = pp.daily_returns;
        assert_eq!(daily_rets[&day1], 0.0);
        assert!(nearly_equal(daily_rets[&day2], 0.1));
    }

    #[test]
    fn trades_accuracy_100_percent() {
        // successful long and short
        let data = vec![
            (Date::from_ymd(2020, 3, 1), 1.0, Pos::Long(1)),
            (Date::from_ymd(2020, 3, 2), 1.2, Pos::Hold),
            (Date::from_ymd(2020, 3, 3), 1.4, Pos::Out),
            (Date::from_ymd(2020, 3, 4), 1.4, Pos::Short(1)),
            (Date::from_ymd(2020, 3, 5), 1.5, Pos::Hold),
            (Date::from_ymd(2020, 3, 6), 1.6, Pos::Out),
        ];
        let (pv, trades) = construct_fixture(&data);
        let pp = PortfolioPerformance::new(pv).unwrap();

        let accuracy = pp.trades_accuracy(&trades).unwrap();
        assert_eq!(accuracy, 1.0);
    }

    #[test]
    fn trades_accuracy_66_percent() {
        let data = vec![
            // successful long and short
            (Date::from_ymd(2020, 3, 1), 1.0, Pos::Long(1)),
            (Date::from_ymd(2020, 3, 2), 1.2, Pos::Hold),
            (Date::from_ymd(2020, 3, 3), 1.4, Pos::Out),
            (Date::from_ymd(2020, 3, 4), 1.4, Pos::Short(1)),
            (Date::from_ymd(2020, 3, 5), 1.5, Pos::Hold),
            (Date::from_ymd(2020, 3, 6), 1.6, Pos::Out),
            // Unsuccessful long
            (Date::from_ymd(2020, 3, 7), 1.6, Pos::Long(1)),
            (Date::from_ymd(2020, 3, 8), 1.5, Pos::Hold),
            // This last short shouldn't be counted as a trade because it is
            // incomplete (effectively just an exit of the long trade)
            (Date::from_ymd(2020, 3, 9), 1.4, Pos::Short(1)),
        ];
        let (pv, trades) = construct_fixture(&data);
        let pp = PortfolioPerformance::new(pv).unwrap();

        let accuracy = pp.trades_accuracy(&trades).unwrap();
        // assert_eq!(accuracy, 0.6666666666666666);
        assert_eq!(accuracy, 2.0 / 3.0);
    }
}
