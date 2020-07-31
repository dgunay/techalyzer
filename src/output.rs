use crate::signals::signals::Outputs;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use crate::error::TechalyzerError;
// use strum;
use strum_macros::EnumString;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, EnumString)]
// #[serde(untagged)]
pub enum SupportedIndicators {
    BollingerBands,
    RelativeStrengthIndex,
    MACD,
}

// #[derive(Serialize, Deserialize)]
// pub struct SupportedIndicators;
// impl SupportedIndicators {
//     pub const BollingerBands: &'static str = "bollinger-bands";
//     pub const RelativeStrengthIndex: &'static str = "rsi";
//     pub const MACD: &'static str = "macd";
// }

// impl std::str::FromStr for SupportedIndicators {
//     type Err = TechalyzerError;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             SupportedIndicators::BollingerBands => Ok(SupportedIndicators::BollingerBands),
//             SupportedIndicators::RelativeStrengthIndex => Ok(SupportedIndicators::RelativeStrengthIndex),
//             SupportedIndicators::MACD => Ok(SupportedIndicators::MACD),
//             _ => Err(TechalyzerError::Generic(format!(
//                 "{} is not a supported technical indicator",
//                 s
//             ))),
//         }
//     }
// }

/// An entry at some date with price, signal, and technical indicator data
#[derive(Serialize, Deserialize)]
pub struct TechalyzerEntry {
    // date: NaiveDate,
    pub signal: f64,
    // pub output: Outputs, // TODO: Outputs needs to be restructed to fit this model better
    pub price: f64
}

// TODO: a map structure of date => [price, signals, outputs] would be very useful
// for charting and otherwise using the data.
/// Organizes our data the way we want before printing.
#[derive(Serialize, Deserialize)]
pub struct TechalyzerPrintOutput {
    pub map: BTreeMap<NaiveDate, TechalyzerEntry>,
    pub symbol: String,
    pub indicator: SupportedIndicators,
    // pub dates: Vec<NaiveDate>,
    // pub signals: Vec<f64>,
    // pub outputs: Option<Outputs>,
    // pub prices: Vec<f64>,
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