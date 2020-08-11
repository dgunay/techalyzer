use crate::signals::signals::{Output, Signal};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// use strum;
use std::collections::BTreeMap;
use strum_macros::EnumString;

#[derive(Debug, Serialize, Deserialize, EnumString)]
// #[serde(untagged)]
pub enum SupportedIndicators {
    #[strum(serialize = "BollingerBands", serialize = "bb")]
    BollingerBands,

    #[strum(serialize = "RelativeStrengthIndex", serialize = "rsi")]
    RelativeStrengthIndex,

    #[strum(serialize = "MACD", serialize = "macd")]
    MACD,
}

/// An entry at some date with price, signal, and technical indicator data
#[derive(Serialize, Deserialize)]
pub struct TechalyzerEntry {
    pub signal: Signal,
    pub price: f64,
    pub output: Output,
}

/// Organizes our data the way we want before printing.
#[derive(Serialize, Deserialize)]
pub struct TechalyzerPrintOutput {
    pub map: BTreeMap<NaiveDate, TechalyzerEntry>,
    pub symbol: String,
    pub indicator: SupportedIndicators,
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
