use super::tradingmodel::{Trades, TradingModel};
use crate::marketdata::prices::Prices;
use std::str::FromStr;

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

impl TradingModel for ManualTradingModel {
    fn get_trades(&self, prices: &Prices) -> Trades {
        todo!()
    }
}
