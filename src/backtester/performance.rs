//! Measures portfolio performance as total/daily returns over periods of time.

use crate::util::first_key;
use crate::Date;
use derive_more::Display;
use serde::Serialize;
use stats::stddev;
use std::{collections::BTreeMap, ops::RangeBounds};

/// Represents portfolio performance.
#[derive(Debug, Serialize)]
pub struct PortfolioPerformance {
    /// The running total portfolio value in a time series.
    pub daily_portvals: BTreeMap<Date, f64>,

    // pub sharpe_ratio: f64, // TODO: add this
    /// The daily portfolio returns in a time series.
    pub daily_returns: BTreeMap<Date, f64>,

    /// Standard deviation of daily returns.
    pub volatility: f64,
}

/// Errors that may occur during portfolio performance calculation.
#[derive(Debug, Serialize, Display)]
pub enum PerformanceError {
    /// Occurs if there is not at least one data point to measure.
    #[display(fmt = "Not enough data points to calculate performance")]
    NotEnoughDataPoints,
}

impl PortfolioPerformance {
    /// Constructs a PortfolioPerformance. There must be at least one datapoint
    /// in `daily_portvals`.
    pub fn new(daily_portvals: BTreeMap<Date, f64>) -> Result<Self, PerformanceError> {
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
        daily_returns.insert(first_key.clone(), 0.0);
        let mut yesterday = daily_portvals[first_key];
        for (day, value) in daily_portvals.iter() {
            if day == first_key {
                continue;
            }
            let ret = (value / yesterday) - 1.0;
            daily_returns.insert(day.clone(), ret);
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
    use crate::util::nearly_equal;

    #[test]
    fn total_return_divide_by_zero() {
        let mut daily_portvals = BTreeMap::new();
        daily_portvals.insert(Date::from_ymd(2020, 3, 1), 0.0);
        daily_portvals.insert(Date::from_ymd(2020, 3, 2), 10.0);
        let pp = PortfolioPerformance::new(daily_portvals).unwrap();
        assert_eq!(pp.total_return().unwrap(), f64::INFINITY);
    }

    #[test]
    fn daily_returns() {
        let mut daily_portvals = BTreeMap::new();
        let day1 = Date::from_ymd(2020, 3, 1);
        let day2 = Date::from_ymd(2020, 3, 2);
        daily_portvals.insert(day1, 10.0);
        daily_portvals.insert(day2, 11.0);
        let pp = PortfolioPerformance::new(daily_portvals).unwrap();
        let daily_rets = pp.daily_returns;
        assert_eq!(daily_rets[&day1], 0.0);
        assert!(nearly_equal(daily_rets[&day2], 0.1));
    }
}
