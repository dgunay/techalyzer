use super::tradingmodel::{Trades, TradingModel};
use crate::{backtester::Position, signals::signals::SignalsIter};
use crate::{
    marketdata::prices::Prices,
    signals::{
        bollingerbandssignals::{BBSignalsIter, BollingerBandsSignals},
        macdsignals::{MACDSignalsIter, MovingAverageConvergenceDivergenceSignals},
        relativestrengthindexsignals::{RSISignalsIter, RelativeStrengthIndexSignals},
        signals::Signals,
    },
};
use chrono::NaiveDate;
use std::{collections::BTreeMap, str::FromStr};
use ta::indicators::SimpleMovingAverage;

pub enum Error {
    NoSignalAvailable,
}

pub struct ManualTradingModel {
    shares: u64,

    /// How far the signal needs to be from 0 in order to make a trade. For
    /// example, if the dead zone is 0.2, only an average signal of less than
    /// -0.2 or greater than 0.2 will cause the model to go short or long.
    dead_zone: f64,
}

impl ManualTradingModel {
    pub fn new(shares: u64, dead_zone: f64) -> Self {
        Self { shares, dead_zone }
    }

    pub fn set_shares(&mut self, shares: u64) {
        self.shares = shares;
    }
}

impl Default for ManualTradingModel {
    fn default() -> Self {
        Self::new(1000, 0.0)
    }
}

pub enum ManualTradingModelError {
    NotConvertibleFromStr(String),
}

impl FromStr for ManualTradingModel {
    type Err = ManualTradingModelError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "manual" | "manualtradingmodel" => Ok(ManualTradingModel::default()),
            _ => Err(ManualTradingModelError::NotConvertibleFromStr(
                s.to_string(),
            )),
        }
    }
}

enum MarketState {
    Trending,
    Oscillating,
}

pub fn average_slope(_prices: &Prices, _sma: SimpleMovingAverage) -> f64 {
    todo!()
}

impl ManualTradingModel {
    fn current_market_state(&self, prices: &Prices, _today: &NaiveDate) -> MarketState {
        // Take the average slope of some N-day moving average, perhaps 75
        // TODO: parameterize trend checker window instead of hardcoding 75
        let sma = SimpleMovingAverage::new(75).expect("Couldn't construct SMA");
        match average_slope(&prices, sma) {
            slope if slope >= 0.4 || slope <= -0.4 => MarketState::Trending,
            _ => MarketState::Oscillating,
        }
    }
}

impl TradingModel for ManualTradingModel {
    fn get_trades(&self, prices: &Prices) -> Trades {
        // Make a bin of technical indicators to use - 2 trending, 2 oscillating.

        let mut rsi = RSISignalsIter::default();
        let mut bb = BBSignalsIter::default();
        let mut macd = MACDSignalsIter::default();

        let mut trades = BTreeMap::new();
        for (day, price) in prices.iter() {
            // TODO: make the market trend cause the algo to favor trend or
            // oscillating indicators
            // let market_state = match self.current_market_state(&prices, &day) {
            //     MarketState::Trending => todo!("Favor trend indicators"),
            //     MarketState::Oscillating => todo!("Favor oscillating indicators"),
            // };
            let sum: f64 = vec![rsi.next(*price).0, bb.next(*price).0, macd.next(*price).0]
                .iter()
                .map(|s| s.val)
                .sum();

            // Consult the indicators consensus.
            let trade = match sum / 3.0 {
                avg if avg >= self.dead_zone => Position::Long(self.shares),
                avg if avg <= -self.dead_zone => Position::Short(self.shares),
                _ => Position::Out, // TODO: should I hold instead?
            };

            // Make a trade.
            trades.insert(day.clone(), trade);
        }

        Trades { trades }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_manual_trader() {
        todo!("write test for manual trader")
    }
}
