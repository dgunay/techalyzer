use chrono::NaiveDate;
use std::{ops::RangeInclusive, path::PathBuf, str::FromStr};
use structopt::StructOpt;
use techalyzer::datasources::SupportedDataSources;
use techalyzer::error::TechalyzerError;
use techalyzer::get_market_data;
use techalyzer::output::SupportedIndicators;
use techalyzer::secret::Secret;
use techalyzer::subcommands::*;
use techalyzer::{trading::SupportedTradingModel, util::today_naive};

#[derive(StructOpt, Debug)]
#[structopt(name = "Techalyzer", author = "Devin Gunay")]
struct Opts {
    /// Secret associated with your chosen data source, usually an API key or something.
    #[structopt(long)]
    secret: Option<String>,

    // TODO: it'd be better for error display if a data source were
    // selected as mutually exclusive flags (e.g. --file-data and --api-data)
    /// Where to get stock data from
    #[structopt(long, short)]
    data_source: SupportedDataSources,

    /// The symbol of the security to analyze
    symbol: String,

    /// Start date of the analysis. Leave out to go to the earliest possible date.
    #[structopt(long, short, parse(try_from_str = parse_date))]
    start_date: Option<NaiveDate>,

    /// End date of the analysis. Leave out to go to the latest possible date
    /// (usually today).
    #[structopt(long, short, parse(try_from_str = parse_date))]
    end_date: Option<NaiveDate>,

    #[structopt(subcommand)]
    cmd: SubCommands,
}

/// Gives us a little more flexibility when parsing dates from the command line
/// for things like "today"
fn parse_date(datestr: &str) -> Result<NaiveDate, chrono::ParseError> {
    match datestr {
        "today" => Ok(chrono::Utc::now().naive_local().date()),
        "yesterday" => Ok(chrono::Utc::now().naive_local().date() - chrono::Duration::days(1)),
        // TODO: maybe implement things like "a year ago", "a month ago", etc
        s => NaiveDate::from_str(s),
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
        // TODO: decide on common model training parameters
        #[structopt(flatten)]
        params: TrainingParams,

        #[structopt(long, short)]
        out_path: PathBuf,
    },

    /// Suggests a trading course of action given recent developments in a
    /// security's price action.
    Suggest {
        /// See [SupportedTradingModel](enum.SupportedTradingModel.html)
        model: SupportedTradingModel,
    },

    /// Backtests a strategy through a given dataset
    BackTest {
        trading_model: SupportedTradingModel,
        cash: f64, // TODO: is there a good money type/bignum to avoid possible problems?
    },
}

// TODO: remove if we don't use this
// #[derive(Debug)]
// enum TrainOrLoad {
//     Train(TrainingParams),
//     Load(PathBuf),
// }

// impl Default for TrainOrLoad {
//     fn default() -> Self {
//         TrainOrLoad::Train(TrainingParams::default())
//     }
// }

#[derive(Debug, StructOpt)]
struct TrainingParams {
    signal_generators: Vec<SupportedIndicators>,
    train_start_date: NaiveDate,
    train_end_date: NaiveDate,
    #[structopt(long, short)]
    horizon: u32,
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
            train_end_date: today_naive(),
            horizon: 10,
        }
    }
}

fn main() -> Result<(), TechalyzerError> {
    let opts = Opts::from_args();
    run_program(opts)
}

fn very_early_date() -> NaiveDate {
    NaiveDate::from_ymd(1000, 1, 1)
}

/// Wrappable main function to make it easier to test.
fn run_program(opts: Opts) -> Result<(), TechalyzerError> {
    // Date range for the data
    let start = opts.start_date;
    let end = opts.end_date;

    // API keys if necessary
    let secret = Secret { data: opts.secret };

    // FIXME: this is a hack because I can't figure out how to have both
    // bounded inclusive ranges and full/unbounded ranges in the same variable.
    let impossibly_early_date = very_early_date();
    let date_range: RangeInclusive<NaiveDate> = match (start, end) {
        (None, None) => impossibly_early_date..=today_naive(),
        (None, Some(end)) => impossibly_early_date..=end,
        (Some(start), None) => start..=today_naive(),
        (Some(start), Some(end)) => start..=end,
    };

    // Get market data
    let prices = match get_market_data(opts.data_source, opts.symbol, date_range, secret) {
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
        } => train(
            prices,
            p.train_start_date..=p.train_end_date,
            p.signal_generators,
            p.horizon,
            out_path,
        )?,
        SubCommands::BackTest {
            trading_model,
            cash,
        } => {
            backtest(prices, trading_model, cash)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{run_program, Opts, SubCommands};
    use super::{SupportedDataSources, SupportedIndicators};

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
}
