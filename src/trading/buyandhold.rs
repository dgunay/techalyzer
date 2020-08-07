use super::tradingmodel::{Trades, TradingModel};
use crate::backtester::Position::*;
use crate::marketdata::prices::Prices;
use crate::util::first_key;
use std::{collections::BTreeMap, convert::Infallible, str::FromStr};

pub struct BuyAndHold {
    shares: u64,
}

impl Default for BuyAndHold {
    fn default() -> Self {
        Self { shares: 1000 }
    }
}

impl TradingModel for BuyAndHold {
    fn get_trades(&self, prices: &Prices) -> Trades {
        let mut trades = BTreeMap::new();
        let first_day = first_key(&prices.map).expect("No first day in prices");
        trades.insert(first_day.clone(), Long(self.shares));
        let mut iter = prices.map.iter();
        let _ = iter.next();
        for (d, _) in iter {
            trades.insert(d.clone(), Hold);
        }

        Trades { trades }
    }
}

impl FromStr for BuyAndHold {
    type Err = Infallible;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        // TODO: how should we make the shares parameterized?
        Ok(Self { shares: 1000 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_buy_and_hold() {
        let day1 = NaiveDate::from_ymd(2020, 1, 1);
        let day2 = NaiveDate::from_ymd(2020, 1, 2);
        let day3 = NaiveDate::from_ymd(2020, 1, 3);
        let model = BuyAndHold { shares: 1000 };
        let map: BTreeMap<NaiveDate, f64> = vec![(day1, 30.0), (day2, 32.0), (day3, 34.0)]
            .iter()
            .cloned()
            .collect();
        let prices = Prices {
            symbol: "JPM".to_string(),
            map: map,
        };

        let trades = model.get_trades(&prices);
        assert_eq!(trades.get(&day1).unwrap(), &Long(1000));
        assert_eq!(trades.get(&day2).unwrap(), &Hold);
        assert_eq!(trades.get(&day3).unwrap(), &Hold);
    }
}
