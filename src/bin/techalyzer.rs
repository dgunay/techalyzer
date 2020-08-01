use chrono::NaiveDate;

use structopt::StructOpt;
use ta::indicators::*;
use ta_experiments::datasources::SupportedDataSources;
use ta_experiments::error::TechalyzerError;
use ta_experiments::get_market_data;
use ta_experiments::output::SupportedIndicators;
use ta_experiments::output::TechalyzerEntry;
use ta_experiments::output::TechalyzerPrintOutput;
use ta_experiments::secret::Secret;
use ta_experiments::signals::{
    bollingerbandssignals::BollingerBandsSignals,
    macdsignals::MovingAverageConvergenceDivergenceSignals,
    relativestrengthindexsignals::RelativeStrengthIndexSignals, signals::Signals,
};
use ta_experiments::source::Source;

// FIXME: we probably don't need the overhead of structopt, look into switching
// to pico-args (https://github.com/RazrFalcon/pico-args)

///
#[derive(StructOpt, Debug)]
#[structopt(name = "Techalyzer")]
struct Opts {
    /// Secret associated with your chosen data source, usually an API key or something.
    #[structopt(long)]
    secret: Option<String>,

    /// Where to get stock data from
    #[structopt(long)]
    data_source: SupportedDataSources,

    /// The symbol of the security to analyze
    symbol: String,

    /// Start date of the analysis. Leave out to go to the earliest possible date.
    start_date: Option<NaiveDate>,

    /// End date of the analysis. Leave out to go to the latest possible date
    /// (usually today).
    end_date: Option<NaiveDate>,

    #[structopt(subcommand)]
    cmd: SubCommands,
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
    /// technical indicators.
    Train {},

    /// Suggests a trading course of action given recent developments in a
    /// security's price action.
    Suggest {},
}

fn main() -> Result<(), TechalyzerError> {
    let opts = Opts::from_args();
    run_program(opts)
}

/// Wrapper function for Techalyzer to make it easier to test.
fn run_program(opts: Opts) -> Result<(), TechalyzerError> {
    // Date range for the data
    let start = opts.start_date;
    let end = opts.end_date;

    // API keys if necessary
    let secret = Secret { data: opts.secret };

    let source = match opts.data_source {
        SupportedDataSources::File(path) => Source::TechalyzerJson(path),
        SupportedDataSources::AlphaVantage => Source::AlphaVantage,
    };

    // Get market data
    // TODO: parameterize the data source
    let data = match get_market_data(source, opts.symbol, start, end, secret) {
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
            let prices: Vec<&f64> = data.map.values().collect();

            // Calculate the technical indicator outputs and signals
            // TODO: allow parameters for each indicator
            // FIXME: is there any way we can avoid heap allocating/dynamic dispatch?
            let sigs: Box<dyn Signals> = match indicator {
                SupportedIndicators::BollingerBands => Box::new(BollingerBandsSignals::new(
                    prices,
                    BollingerBands::new(20, 2.0).expect("invalid Bollinger Bands"),
                )),
                SupportedIndicators::RelativeStrengthIndex => {
                    Box::new(RelativeStrengthIndexSignals::new(
                        prices,
                        RelativeStrengthIndex::new(14).expect("invalid RSI params"),
                    ))
                }
                SupportedIndicators::MACD => {
                    Box::new(MovingAverageConvergenceDivergenceSignals::new(
                        prices,
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
            for (date, price) in data.map.iter() {
                m.insert(
                    date.clone(),
                    TechalyzerEntry {
                        // output: sigs.outputs().outputs
                        price: price.clone(),
                        signal: sigs.signals()[i],
                    },
                );
                i += 1;
            }

            // TODO: factor out this ugliness or change the datastructures
            // involved to be less gross
            let output = TechalyzerPrintOutput {
                symbol: data.symbol,
                indicator: indicator,
                map: m,
            };

            print_techalyzer_json(&output);
        }
        SubCommands::Suggest {} => todo!(),
        SubCommands::Train {} => todo!(),
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
            data_source: SupportedDataSources::File("test/json/jpm_rsi.json".into()),
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
