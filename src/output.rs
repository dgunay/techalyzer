//! Structs for serializing/deserializing the output of Techalyzer to JSON.

use crate::{
    backtester::performance::PortfolioPerformance,
    indicators::SupportedIndicators,
    marketdata::prices::Prices,
    signals::{Output, Signal},
    trading::tradingmodel::Trades,
    util::TimeSeries,
};
use serde::{Deserialize, Serialize};

/// An entry at some date with price, signal, and technical indicator data.
#[derive(Serialize, Deserialize)]
pub struct TechalyzerEntry {
    pub signal: Signal,
    pub price: f64,
    pub output: Output,
}

/// Organizes the output of Print the way we want before printing to JSON.
#[derive(Serialize, Deserialize)]
pub struct TechalyzerPrintOutput {
    pub map: TimeSeries<TechalyzerEntry>,
    pub symbol: String,
    pub indicator: SupportedIndicators,
}

/// Organizes the output of BackTest before printing to JSON.
#[derive(Serialize)]
pub struct TechalyzerBacktestOutput {
    pub prices: Prices,
    pub trades: Trades,
    pub performance: PortfolioPerformance,
    pub benchmark: PortfolioPerformance,
    pub total_return: f64,
    pub model_name: String,
    pub symbol: String,
}

#[cfg(test)]
mod tests {
    use super::SupportedIndicators;

    #[test]
    fn test_supported_indicators_serializes_to_string() {
        let bb = vec![SupportedIndicators::BollingerBands];
        let res = serde_json::to_string(&bb).unwrap();
        assert_eq!(res, "[\"BollingerBands\"]");
    }
}
