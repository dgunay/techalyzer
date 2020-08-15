use std::{ops::RangeInclusive, path::PathBuf, str::FromStr};
use structopt::StructOpt;
use techalyzer::datasources::SupportedDataSources;
use techalyzer::error::TechalyzerError;
use techalyzer::get_market_data;
use techalyzer::output::SupportedIndicators;
use techalyzer::secret::Secret;
use techalyzer::subcommands::*;
use techalyzer::{
    date::{today, Date},
    trading::SupportedTradingModel,
};

#[derive(StructOpt, Debug)]
#[structopt(name = "Techalyzer", author = "Devin Gunay")]
struct Opts {
    /// Secret associated with your chosen data source, usually an API key
    #[structopt(long)]
    secret: Option<String>,

    // TODO: it'd be better for error display if a data source were
    // selected as mutually exclusive flags (e.g. --file-data and --api-data)
    /// Where to get stock data from
    #[structopt(long, short)]
    data_source: SupportedDataSources,

    /// The symbol of the security to analyze
    symbol: String,

    /// Start date of the analysis. Defaults to the earliest possible date.
    #[structopt(long, short, parse(try_from_str = parse_date))]
    start_date: Option<Date>,

    /// End date of the analysis. Defaults to the latest possible date
    /// (usually today).
    #[structopt(long, short, parse(try_from_str = parse_date))]
    end_date: Option<Date>,

    #[structopt(subcommand)]
    cmd: SubCommands,
}

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

#[derive(StructOpt, Debug)]
enum SubCommands {
    /// Using time series price data, prints a technical indicator and the
    /// normalized signals generated by Techalyzer to STDOUT as JSON data.
    Print {
        #[structopt(short, long)]
        indicator: SupportedIndicators,

        /// Print buy/sell signals along with the indicator
        #[structopt(short, long)]
        print_signals: bool,
    },

    /// Trains a machine learning model on stock data to make trades based on
    /// technical indicators, then serializes it for later use.
    Train {
        #[structopt(flatten)]
        params: TrainingParams,

        /// File path to output a model file to.
        #[structopt(long, short)]
        out_path: Option<PathBuf>,
    },

    /// Suggests a trading course of action given recent developments in a
    /// security's price action.
    Suggest { model: SupportedTradingModel },

    /// Backtests a strategy through a given dataset
    BackTest {
        /// Which trading model to use.
        trading_model: SupportedTradingModel,

        /// Saved model file to use (generate one with `techalyzer train`)
        #[structopt(long, short, required_if("trading-model", "MachineLearningModel"))]
        model_file: Option<PathBuf>,

        /// How much cash the model begins with.
        cash: f64, // TODO: is there a good money type/bignum to avoid possible problems?
    },
}

#[derive(Debug, StructOpt)]
struct TrainingParams {
    /// Start date of the training dataset.
    train_start_date: Date,

    /// End date of the training. Defaults to the end of the dataset, less
    /// `horizon` days.
    train_end_date: Option<Date>,

    /// How many days in the future to check future returns in order to decide
    /// how to label the data. Defaults to 10 days.
    #[structopt(default_value = "10", long, short)]
    horizon: u32,

    /// What percentage (+/-) returns to consider a buying or shorting
    /// opportunity when looking at future returns. Defaults to 0.03 (3%
    /// returns).
    #[structopt(default_value = "0.03", long, short)]
    decision_threshold: f64,

    /// Which technical indicators to use to generate features for the learner.
    #[structopt(long, short)]
    signal_generators: Vec<SupportedIndicators>,
}

impl Default for TrainingParams {
    fn default() -> Self {
        Self {
            signal_generators: vec![
                SupportedIndicators::RelativeStrengthIndex,
                SupportedIndicators::BollingerBands,
                SupportedIndicators::MACD,
            ],
            train_start_date: very_early_date(),
            train_end_date: Some(Date::default()),
            horizon: 10,
            decision_threshold: 0.03,
        }
    }
}

fn main() -> Result<(), TechalyzerError> {
    let opts = Opts::from_args();
    match run_program(opts) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

fn very_early_date() -> Date {
    Date::from_ymd(1000, 1, 1)
}

/// Wrappable main function to make it easier to test.
fn run_program(opts: Opts) -> Result<(), TechalyzerError> {
    // Date range for the data
    let start = opts.start_date;
    let end = opts.end_date;

    // API keys if necessary
    let secret = Secret { data: opts.secret };

    let start_date = start.unwrap_or(very_early_date());
    let end_date = end.unwrap_or(today());

    // FIXME: this is a hack because I can't figure out how to have both
    // bounded inclusive ranges and full/unbounded ranges in the same variable.
    let impossibly_early_date = very_early_date();
    let date_range: RangeInclusive<Date> = match (start, end) {
        (None, None) => impossibly_early_date..=today(),
        (None, Some(end)) => impossibly_early_date..=end,
        (Some(start), None) => start..=today(),
        (Some(start), Some(end)) => start..=end,
    };

    // Get market data
    let prices = match get_market_data(opts.data_source, opts.symbol, start_date..=end_date, secret)
    {
        Ok(d) => d,
        Err(e) => {
            return Err(TechalyzerError::Generic(format!("{}", e)));
        }
    };

    // Run a subcommand
    match opts.cmd {
        SubCommands::Print {
            indicator,
            print_signals: _,
        } => {
            // TODO: evaluate/benchmark signal generation using ndarray vs Vec<f64>
            print(prices, indicator)?;
        }
        SubCommands::Suggest { model: _ } => todo!("Suggest not yet implemented"),
        SubCommands::Train {
            params: p,
            out_path,
        } => {
            if p.signal_generators.is_empty() {
                return Err(TechalyzerError::NoIndicatorSpecified);
            }

            dbg!(&prices);

            // Manual end date or -horizon days before the end of the dataset.
            let end_date = end.unwrap_or(
                *prices
                    .iter()
                    .rev()
                    .nth(p.horizon as usize)
                    .ok_or(format!(
                        "No day found {} days before last day in dataset",
                        p.horizon
                    ))?
                    .0,
            );

            // Copy our training dates out of the Price data set.
            let range: Vec<Date> = prices
                .date_range(start_date..=end_date)
                .map
                .keys()
                .cloned()
                .collect();

            // TODO: include date info
            // FIXME: need a way to output to null for testing
            let out_path =
                out_path.unwrap_or_else(|| PathBuf::from(format!("{}.bin", &prices.symbol)));
            train(prices, range, p.signal_generators, p.horizon, out_path)?
        }
        SubCommands::BackTest {
            trading_model,
            cash,
            model_file,
        } => {
            backtest(prices, trading_model, model_file, cash)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{run_program, Opts, SubCommands};
    use super::{SupportedDataSources, SupportedIndicators};
    use techalyzer::date::Date;
    use crate::TrainingParams;
    use tempfile::NamedTempFile;

    #[test]
    fn end_to_end_print_rsi() {
        // Basic smoke test that the program can go end to end
        let res = run_program(Opts {
            data_source: SupportedDataSources::TechalyzerJson("test/json/jpm_rsi.json".into()),
            secret: None,
            symbol: "JPM".to_string(),
            start_date: None,
            end_date: None,
            cmd: SubCommands::Print {
                indicator: SupportedIndicators::RelativeStrengthIndex,
                print_signals: true,
            },
        });

        res.unwrap();
    }

    // TODO: test behavior of each path (mainly whether required arguments work
    // properly or not)

    #[test]
    fn training_dates() {
        let file = NamedTempFile::new().unwrap();

        // FIXME: running Train with this date range causes errors
        let res = run_program(Opts {
            secret: None,
            data_source: SupportedDataSources::TechalyzerJson("test/json/jpm_rsi.json".into()),
            symbol: "JPM".to_string(),
            start_date: None,
            end_date: Some(Date::from_ymd(2020, 06, 02)),
            cmd: SubCommands::Train {
                params: TrainingParams::default(),
                out_path: Some(file.path().to_path_buf()),
            },
        }).unwrap();
        // todo!()
    }
}
