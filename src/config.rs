//! Configuration formats and parameters for various aspects of Techalyzer.

use crate::{
    datasource::SupportedDataSource,
    date::{today, Date},
    indicators::{ListOfIndicators, SupportedIndicators},
    trading::dtmodel::{DecisionThreshold, Horizon},
    util::ToJson,
};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, ops::Deref, str::FromStr};
use structopt::StructOpt;

// TODO: link to these as the central source of truth for the frontend args
// and backend.

// TODO: is it possible to have missing keys use a default value?

/// General parameters common to every subcommand of Techalyzer.
#[derive(Serialize, Deserialize, StructOpt, Debug, PartialEq)]
pub struct GeneralParams {
    /// Secret associated with your chosen data source, usually an API key
    #[structopt(long)]
    #[serde(default)]
    pub secret: Option<String>,

    // TODO: it'd be better for error display if a data source were
    // selected as mutually exclusive flags (e.g. --file-data and --api-data)
    /// Where to get stock data from
    #[structopt(long, short)]
    pub data_source: SupportedDataSource,

    /// The symbol of the security to analyze
    #[structopt()]
    pub symbol: Symbol,

    /// Start date of the analysis. Defaults to the earliest possible date.
    #[structopt(long, short, parse(try_from_str = parse_date))]
    #[serde(default)]
    pub start_date: Option<Date>,

    /// End date of the analysis. Defaults to the latest possible date
    /// (usually today).
    #[structopt(long, short, parse(try_from_str = parse_date))]
    #[serde(default)]
    pub end_date: Option<Date>,
}

/// A stock ticker symbol.
#[derive(Debug, Default, Serialize, Deserialize, FromStr, PartialEq)]
#[serde(transparent)]
pub struct Symbol(String);
impl Symbol {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl Deref for Symbol {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToJson for GeneralParams {}

/// Parameters when running the Train command.
#[derive(Debug, StructOpt, Deserialize, Serialize, PartialEq)]
pub struct TrainingParams {
    /// Start date of the training dataset. Defaults to the beginning of the
    /// dataset.
    #[serde(default)]
    pub train_start_date: Option<Date>,

    /// End date of the training. Defaults to the end of the dataset, less
    /// `horizon` days.
    #[serde(default)]
    pub train_end_date: Option<Date>,

    /// How many days in the future to check future returns in order to decide
    /// how to label the data.
    // TODO: implement a default
    #[structopt(default_value = "10", long, short)]
    #[serde(default)]
    pub horizon: Horizon,

    /// What percentage (+/-) returns to consider a buying or shorting
    /// opportunity when looking at future returns. Defaults to 3%
    /// returns.
    #[structopt(default_value = "0.03", long, short)]
    #[serde(default)]
    pub decision_threshold: DecisionThreshold,

    /// Which technical indicators to use to generate features for the learner.
    #[structopt(long, short, default_value)]
    #[serde(default)]
    pub signal_generators: ListOfIndicators,
}

impl ToJson for TrainingParams {}

// FIXME: remove this when done experimenting
impl FromStr for TrainingParams {
    type Err = Infallible;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Self::default())
    }
}

impl Default for TrainingParams {
    fn default() -> Self {
        Self {
            signal_generators: ListOfIndicators(vec![
                SupportedIndicators::RelativeStrengthIndex,
                SupportedIndicators::BollingerBands,
                SupportedIndicators::MACD,
            ]),
            train_start_date: None,
            train_end_date: None,
            // train_end_date: Some(Date::default()),
            horizon: 10.into(),
            decision_threshold: 0.03.into(),
        }
    }
}

// TODO: determine whether separate structs for different kinds of ml algorithms
// are necessary

/// Parameters for running the Backtester.
#[derive(Serialize, Deserialize)]
pub struct BacktesterParams {}

/// Gives us a little more flexibility when parsing dates from the command line
/// for things like "today"
fn parse_date(datestr: &str) -> Result<Date, chrono::ParseError> {
    match datestr {
        "today" => Ok(today()),
        "yesterday" => Ok(today() - chrono::Duration::days(1)),
        // TODO: maybe implement things like "a year ago", "a month ago", etc
        s => Date::from_str(s),
    }
}

#[cfg(test)]
mod tests {
    use super::{GeneralParams, Symbol, TrainingParams};
    use crate::{datasource::SupportedDataSource::TechalyzerJson, util::ToJson};

    #[test]
    fn test_trainingparams_json() {
        // results of dumping default parameters to json
        let result = TrainingParams::default();
        let _as_json_str = result.to_json().unwrap();
        // should look something like this, the exact specifics are not that
        // important
        /* {
          "train_start_date": null,
          "train_end_date": "2020-08-20",
          "horizon": 10,
          "decision_threshold": 0.03,
          "signal_generators": [
            "RelativeStrengthIndex",
            "BollingerBands",
            "MACD"
          ]
        } */

        // What happens if we leave out keys? Do they revert to their defaults?
        let left_out_keys = r#"
        {
            
        }
        "#;

        let params: TrainingParams = serde_json::from_str(left_out_keys).unwrap();
        assert_eq!(params, TrainingParams::default());
    }

    // FIXME: this test is largely obsolete because I decided not to go with
    // deserializable GeneralParams as a way of supplying arguments instead of
    // CLI due to limitations in structopt.
    #[test]
    fn test_generalparams_json() {
        // results of dumping default parameters to json
        let gp = GeneralParams {
            data_source: TechalyzerJson("test/json/jpm_rsi.json".into()),
            symbol: Symbol("jpm".to_string()),
            secret: None,
            start_date: None,
            end_date: None,
        };
        let _as_json_str = gp.to_json().unwrap();

        // should look something like this, the exact specifics are not that
        // important
        /*
        {
          "secret": null,
          "data_source": {
            "TechalyzerJson": "test/json/jpm_rsi.json"
          },
          "symbol": "jpm",
          "start_date": null,
          "end_date": null
        }
        */

        // What happens if we leave out non-mandatory keys? Do they revert to their defaults?
        let left_out_keys = r#"
        {
            "data_source": { 
                "TechalyzerJson": "test/json/jpm_rsi.json"
            },
            "symbol": "jpm"
        }
        "#;

        let params: GeneralParams = serde_json::from_str(left_out_keys).unwrap();
        assert_eq!(params, gp);
    }
}
