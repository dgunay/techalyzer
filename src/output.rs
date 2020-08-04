use crate::signals::signals::Output;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// use strum;
use std::collections::BTreeMap;
use strum_macros::EnumString;

#[derive(Debug, Serialize, Deserialize, EnumString)]
// #[serde(untagged)]
pub enum SupportedIndicators {
    BollingerBands,
    RelativeStrengthIndex,
    MACD,
}

/// An entry at some date with price, signal, and technical indicator data
#[derive(Serialize, Deserialize)]
pub struct TechalyzerEntry {
    pub signal: f64,
    pub price: f64,
    pub output: Output,
}

// TODO: a map structure of date => [price, signals, outputs] would be very useful
// for charting and otherwise using the data.
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
