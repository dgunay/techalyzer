//! Configuration formats and parameters for various aspects of Techalyzer.

use crate::{
    datasource::SupportedDataSource,
    date::{today, Date},
    output::SupportedIndicators,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use structopt::StructOpt;

// TODO: link to these as the central source of truth for the frontend args
// and backend.

// TODO: is it possible to have missing keys use a default value?

/// Serde compatible struct for saving and loading premade Techalyzer
/// configurations.
#[derive(Serialize, Deserialize)]
struct ParamFile {
    ml_params: MachineLearningParams,
    backtester_params: BacktesterParams,
    general_params: GeneralParams,
}

/// General parameters common to every subcommand of Techalyzer.
#[derive(Serialize, Deserialize, StructOpt, Debug)]
pub struct GeneralParams {
    /// Secret associated with your chosen data source, usually an API key
    #[structopt(long)]
    pub secret: Option<String>,

    // TODO: it'd be better for error display if a data source were
    // selected as mutually exclusive flags (e.g. --file-data and --api-data)
    /// Where to get stock data from
    #[structopt(long, short)]
    pub data_source: SupportedDataSource,

    /// The symbol of the security to analyze
    pub symbol: String,

    /// Start date of the analysis. Defaults to the earliest possible date.
    #[structopt(long, short, parse(try_from_str = parse_date))]
    pub start_date: Option<Date>,

    /// End date of the analysis. Defaults to the latest possible date
    /// (usually today).
    #[structopt(long, short, parse(try_from_str = parse_date))]
    pub end_date: Option<Date>,
}

/// Parameters when running the Train command.
#[derive(Debug, StructOpt)]
pub struct TrainingParams {
    /// Start date of the training dataset. Defaults to the beginning of the
    /// dataset.
    pub train_start_date: Option<Date>,

    /// End date of the training. Defaults to the end of the dataset, less
    /// `horizon` days.
    pub train_end_date: Option<Date>,

    /// How many days in the future to check future returns in order to decide
    /// how to label the data.
    #[structopt(default_value = "10", long, short)]
    pub horizon: u32,

    /// What percentage (+/-) returns to consider a buying or shorting
    /// opportunity when looking at future returns. Defaults to 3%
    /// returns.
    #[structopt(default_value = "0.03", long, short)]
    pub decision_threshold: f64,

    /// Which technical indicators to use to generate features for the learner.
    #[structopt(long, short)]
    pub signal_generators: Vec<SupportedIndicators>,
}

impl Default for TrainingParams {
    fn default() -> Self {
        Self {
            signal_generators: vec![
                SupportedIndicators::RelativeStrengthIndex,
                SupportedIndicators::BollingerBands,
                SupportedIndicators::MACD,
            ],
            train_start_date: None,
            train_end_date: Some(Date::default()),
            horizon: 10,
            decision_threshold: 0.03,
        }
    }
}

// TODO: determine whether separate structs for different kinds of ml algorithms
// are necessary
/// Parameters for training a machine learning model.
#[derive(Serialize, Deserialize)]
pub struct MachineLearningParams {}

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
