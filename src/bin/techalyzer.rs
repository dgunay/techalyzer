use chrono::NaiveDate;

use rustlearn::trees::decision_tree::Hyperparameters;
use std::{
    ops::{RangeBounds, RangeInclusive},
    path::PathBuf,
    str::FromStr,
};
use structopt::StructOpt;
use strum_macros::{Display, EnumString};
use ta::indicators::*;
use techalyzer::datasources::SupportedDataSources;
use techalyzer::error::TechalyzerError;
use techalyzer::get_market_data;
use techalyzer::output::SupportedIndicators;
use techalyzer::output::TechalyzerEntry;
use techalyzer::output::{TechalyzerBacktestOutput, TechalyzerPrintOutput};
use techalyzer::secret::Secret;
use techalyzer::signals::{
    bollingerbandssignals::{BBSignalsIter, BollingerBandsSignals},
    macdsignals::{MACDSignalsIter, MovingAverageConvergenceDivergenceSignals},
    relativestrengthindexsignals::{RSISignalsIter, RelativeStrengthIndexSignals},
    signals::{Signals, SignalsIter},
};
use techalyzer::{
    backtester::BackTester,
    marketdata::prices::Prices,
    trading::{
        buyandhold::BuyAndHold, dtmodel::DecisionTreeTrader, manual::ManualTradingModel,
        tradingmodel::TradingModel,
    },
    util::today_naive,
};

// FIXME: we probably don't need the overhead of structopt, look into switching
// to pico-args (https://github.com/RazrFalcon/pico-args)

///
#[derive(StructOpt, Debug)]
#[structopt(name = "Techalyzer")]
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

// TODO: move this somewhere else
/// Can be an ML model or a handwritten algorithm.
#[derive(Debug, EnumString, Display)]
pub enum SupportedTradingModel {
    ManualTradingAlgo,
    BuyAndHold,
    MachineLearningModel { model_file: Option<PathBuf> },
}

fn train_model<'a>(
    prices: &Prices,
    train_dates: impl RangeBounds<NaiveDate>,
    signal_generators: Vec<&'a mut dyn SignalsIter>,
) -> DecisionTreeTrader<'a> {
    let dt = Hyperparameters::new(2).build();
    // let rsi = &mut RSISignalsIter::default();
    // let bb = &mut BBSignalsIter::default();
    // let macd = &mut MACDSignalsIter::default();
    // let signal_generators: Vec<&mut dyn SignalsIter> = vec![rsi, bb, macd];

    // TODO: either load a model or train a new one right here.
    let mut model = DecisionTreeTrader::new(dt, signal_generators);

    // Slice the prices into training data
    let training_p = prices.date_range(train_dates);
    model.train(&training_p, todo!()).unwrap();
    model
}

fn main() -> Result<(), TechalyzerError> {
    let opts = Opts::from_args();
    run_program(opts)
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
    let impossibly_early_date = NaiveDate::from_ymd(1000, 1, 1);
    let date_range: RangeInclusive<NaiveDate> = match (start, end) {
        (None, None) => impossibly_early_date..=today_naive(),
        (None, Some(end)) => impossibly_early_date..=end,
        (Some(start), None) => start..=today_naive(),
        (Some(start), Some(end)) => start..=end,
    };

    // Get market data
    let data = match get_market_data(opts.data_source, opts.symbol, date_range, secret) {
        Ok(d) => d,
        Err(e) => {
            return Err(TechalyzerError::Generic(format!("{}", e)));
        }
    };

    match opts.cmd {
        SubCommands::Print {
            indicator,
            print_signals: _,
        } => {
            // TODO: evaluate/benchmark signal generation using ndarray vs Vec<f64>
            let prices = Prices {
                map: data.map,
                symbol: data.symbol,
            };
            // let prices: Vec<&f64> = data.map.values().collect();

            // Calculate the technical indicator outputs and signals
            // TODO: allow parameters for each indicator
            // FIXME: is there any way we can avoid heap allocating/dynamic dispatch?
            let sigs: Box<dyn Signals> = match indicator {
                SupportedIndicators::BollingerBands => Box::new(BollingerBandsSignals::new(
                    &prices,
                    BollingerBands::new(20, 2.0).expect("invalid Bollinger Bands"),
                )),
                SupportedIndicators::RelativeStrengthIndex => {
                    Box::new(RelativeStrengthIndexSignals::new(
                        &prices,
                        RelativeStrengthIndex::new(14).expect("invalid RSI params"),
                    ))
                }
                SupportedIndicators::MACD => {
                    Box::new(MovingAverageConvergenceDivergenceSignals::new(
                        &prices,
                        MovingAverageConvergenceDivergence::new(12, 26, 9)
                            .expect("Invalid MACD params"),
                    ))
                }
            };

            // TODO: sadly output shapes are not all the same, BollingerBandsOutput
            // is a tuple of f64s whereas the other indicators usually just have
            // a single f64 per data point. Can this be reconciled in a pretty way
            // before printing it?

            let mut m = std::collections::BTreeMap::new();
            let mut i = 0;
            // for (date, price) in data.map.iter() {
            for (date, price) in prices.iter() {
                m.insert(
                    date.clone(),
                    TechalyzerEntry {
                        price: price.clone(),
                        signal: sigs.signals()[i],
                        output: sigs.outputs()[i].clone(),
                    },
                );
                i += 1;
            }

            // TODO: factor out this ugliness or change the datastructures
            // involved to be less gross
            let output = TechalyzerPrintOutput {
                symbol: prices.symbol,
                indicator: indicator,
                map: m,
            };

            print_techalyzer_json(&output);
        }
        SubCommands::Suggest { model: _ } => todo!("Suggest not yet implemented"),
        SubCommands::Train {} => todo!("Train not yet implemented"),
        SubCommands::BackTest {
            trading_model,
            cash,
        } => {
            let trades = match trading_model {
                // TODO: don't unwrap
                SupportedTradingModel::BuyAndHold => {
                    BuyAndHold::default().get_trades(&data).unwrap()
                }
                SupportedTradingModel::ManualTradingAlgo => {
                    ManualTradingModel::default().get_trades(&data).unwrap()
                }
                SupportedTradingModel::MachineLearningModel { model_file } => {
                    let model = match model_file {
                        Some(p) => todo!("deserialize"),
                        None => train_model(&data, todo!(), todo!()),
                    };

                    // TODO: don't unwrap
                    model.get_trades(&data).unwrap()
                }
            };

            // TODO: have a way for the model to tell us its signal data

            // Give the backtester the trades
            let symbol = data.symbol.clone();
            let mut backtester = match BackTester::new(trades.clone(), &data, cash) {
                Ok(bt) => bt,
                Err(e) => return Err(TechalyzerError::Generic(e.to_string())),
            };

            // Run the backtest
            let performance = match backtester.backtest() {
                Ok(perf) => perf,
                Err(e) => return Err(TechalyzerError::Generic(e.to_string())),
            };

            let total_return = performance
                .total_return()
                .expect("Couldn't get total return");
            let output = TechalyzerBacktestOutput {
                performance,
                total_return,
                trades: trades.clone(),
                model_name: trading_model.to_string(),
                symbol,
                prices: data,
            };

            // Serialize the backtest
            match serde_json::to_writer(std::io::stdout(), &output) {
                Ok(_) => (),
                Err(e) => return Err(TechalyzerError::Generic(e.to_string())),
            };

            // TODO: other stats like daily return here
            // print!(
            //     "Total return: {}",
            //     output.total_return
            // );
        }
    }

    Ok(())
}

/// Outputs a string to an output buffer, Stdout by default.
fn print_techalyzer_json(output: &TechalyzerPrintOutput) {
    // TODO: genericize the output stream to allow for writing to a file
    print!(
        "{}",
        serde_json::to_string(output).expect("Failed to output as JSON")
    );
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
