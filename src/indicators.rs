use crate::{config::TrainingParams, signals::SignalsIter};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use strum_macros::{Display, EnumIter, EnumString, EnumVariantNames};

/// The list of technical indicators supported by Techalyzer.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    EnumVariantNames,
    EnumIter,
    Copy,
    Clone,
    PartialEq,
    EnumString,
    Display,
)]
pub enum SupportedIndicators {
    #[strum(serialize = "BollingerBands", serialize = "bb")]
    BollingerBands,

    #[strum(serialize = "RelativeStrengthIndex", serialize = "rsi")]
    RelativeStrengthIndex,

    #[strum(serialize = "MACD", serialize = "macd")]
    MACD,

    SmaCrossover,
}

fn default_indicators() -> Vec<SupportedIndicators> {
    TrainingParams::default().signal_generators.0
}

impl From<&SupportedIndicators> for Box<dyn SignalsIter> {
    fn from(s: &SupportedIndicators) -> Self {
        Self::from(*s)
    }
}

/// Newtype wrapper for a vector of SupportedIndicator. This type exists solely
/// because Strum errors out when the user passes an empty Vec of enums, but we
/// want that to be an acceptable input for Techalyzer.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(transparent)]
pub struct ListOfIndicators(pub Vec<SupportedIndicators>);

impl ListOfIndicators {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<ListOfIndicators> for Vec<SupportedIndicators> {
    fn from(l: ListOfIndicators) -> Self {
        l.0
    }
}

impl Default for ListOfIndicators {
    fn default() -> Self {
        Self(default_indicators())
    }
}

impl Display for ListOfIndicators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let strings: Vec<String> = self.0.iter().map(SupportedIndicators::to_string).collect();
        write!(f, "{}", strings.join(" "))
    }
}

impl FromStr for ListOfIndicators {
    type Err = strum::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split on whitespace and then parse as strings
        let mut res = ListOfIndicators(Vec::new());
        for indic in s.split_whitespace() {
            res.0.push(SupportedIndicators::from_str(indic)?);
        }

        Ok(res)
    }
}
