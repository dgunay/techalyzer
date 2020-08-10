use super::tradingmodel::{Trades, TradingModel};
use crate::{
    marketdata::prices::Prices,
    signals::{
        bollingerbandssignals::BollingerBandsSignals,
        macdsignals::MovingAverageConvergenceDivergenceSignals,
        relativestrengthindexsignals::RelativeStrengthIndexSignals,
    },
};
use std::str::FromStr;
use ta::indicators::{BollingerBands, MovingAverageConvergenceDivergence, RelativeStrengthIndex};

pub struct ManualTradingModel {
    shares: u64,
}

impl ManualTradingModel {
    pub fn new(shares: u64) -> Self {
        Self { shares }
    }

    pub fn set_shares(&mut self, shares: u64) {
        self.shares = shares;
    }
}

impl Default for ManualTradingModel {
    fn default() -> Self {
        Self::new(1000)
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

impl ManualTradingModel {
    fn current_market_state() -> MarketState {
        todo!("given certain long-term technical indicators, are we trending or sideways?")
    }
}

impl TradingModel for ManualTradingModel {
    fn get_trades(&self, prices: &Prices) -> Trades {
        // Make a bin of technical indicators to use - 2 trending, 2 oscillating.

        let rsi = RelativeStrengthIndexSignals::new(prices, RelativeStrengthIndex::default());
        let bb = BollingerBandsSignals::new(prices, BollingerBands::default());
        let macd = MovingAverageConvergenceDivergenceSignals::new(
            prices,
            MovingAverageConvergenceDivergence::default(),
        );

        // let indics = vec![
        //     RelativeStrengthIndexSignals::new(price, mut rsi)
        // ]

        for (day, price) in prices.iter() {
            // Trending or sideways market?
        }

        todo!()
    }
}
