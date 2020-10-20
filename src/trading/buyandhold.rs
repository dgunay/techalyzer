//! Buys and holds shares for the entirety of the trading period.

use super::tradingmodel::{Trades, TradingModel};
use crate::marketdata::prices::Prices;
use crate::trading::Position::*;
use std::{collections::BTreeMap, fmt::Display};
use thiserror::Error;

pub struct BuyAndHold {
    /// How many share to buy and hold.
    shares: u64,
}

impl Default for BuyAndHold {
    fn default() -> Self {
        Self { shares: 1000 }
    }
}

impl Display for BuyAndHold {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BuyAndHold")
    }
}

/// Things that can go wrong with the BuyAndHold.
#[derive(Debug, Error)]
pub enum BuyAndHoldError {
    #[error("No first day found")]
    NoFirstDay,
}

impl TradingModel for BuyAndHold {
    type Error = BuyAndHoldError;

    fn get_trades(self, prices: &Prices) -> Result<Trades, Self::Error> {
        let mut trades = BTreeMap::new();
        let mut iter = prices.map.iter();
        let (first_day, _) = iter.next().ok_or(BuyAndHoldError::NoFirstDay)?;
        trades.insert(*first_day, Long(self.shares));
        for (d, _) in iter {
            trades.insert(*d, Hold);
        }

        Ok(Trades { trades })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{util::TimeSeries, Date};

    #[test]
    fn test_buy_and_hold() {
        let day1 = Date::from_ymd(2020, 1, 1);
        let day2 = Date::from_ymd(2020, 1, 2);
        let day3 = Date::from_ymd(2020, 1, 3);
        let model = BuyAndHold { shares: 1000 };
        let map: TimeSeries<f64> = vec![(day1, 30.0), (day2, 32.0), (day3, 34.0)]
            .iter()
            .cloned()
            .collect();
        let prices = Prices {
            symbol: "JPM".to_string(),
            map: map,
        };

        let trades = model.get_trades(&prices).unwrap();
        assert_eq!(trades.get(&day1).unwrap(), &Long(1000));
        assert_eq!(trades.get(&day2).unwrap(), &Hold);
        assert_eq!(trades.get(&day3).unwrap(), &Hold);
    }
}
