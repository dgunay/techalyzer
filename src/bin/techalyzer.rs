use chrono::NaiveDate;

use structopt::StructOpt;
use ta::indicators::*;
use ta_experiments::error::TechalyzerError;
use ta_experiments::get_market_data;
use ta_experiments::output::SupportedIndicators;
use ta_experiments::output::TechalyzerPrintOutput;
use ta_experiments::secret::Secret;
use ta_experiments::signals::{
    bollingerbandssignals::BollingerBandsSignals,
    macdsignals::MovingAverageConvergenceDivergenceSignals,
    relativestrengthindexsignals::RelativeStrengthIndexSignals, signals::Signals,
};
use ta_experiments::source::Source;
use ta_experiments::output::TechalyzerEntry;

// FIXME: we probably don't need the overhead of structopt, look into switching
// to pico-args (https://github.com/RazrFalcon/pico-args)
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opts {
    /// Secret associated with your chosen data source, usually an API key or something.
    #[structopt(long)]
    secret: Option<String>,

    /// The symbol of the security to analyze
    symbol: String,

    /// Start date of the analysis. Leave out to go to the earliest possible date.
    start_date: Option<NaiveDate>,

    /// End date of the analysis. Leave out to go to the latest possible date (usually today).
    end_date: Option<NaiveDate>,

    #[structopt(subcommand)]
    cmd: SubCommands,
}

/// The subcommands that Techalyzer can do
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
enum SubCommands {
    /// Prints a technical indicator to STDOUT as JSON data.
    Print {
        #[structopt(short, long)]
        indicator: SupportedIndicators,

        /// Print buy/sell signals along with the indicator
        #[structopt(short, long)]
        print_signals: bool,
    },
    Train {},
    Suggest {},
}

fn main() {
    let opts = Opts::from_args();
    match run_program(opts) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }
}

/// Wrapper function for Techalyzer to make it easier to test.
fn run_program(opts: Opts) -> Result<(), TechalyzerError> {
    // Date range for the data
    let start = opts.start_date;
    let end = opts.end_date;

    // API keys if necessary
    let secret = Secret { data: opts.secret };

    // Get market data
    // TODO: parameterize the data source
    let data = match get_market_data(Source::AlphaVantage, opts.symbol, start, end, secret) {
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
                m.insert(date.clone(), TechalyzerEntry {
                    // output: sigs.outputs().outputs
                    price: price.clone(),
                    signal: sigs.signals()[i],
                });
                i += 1;
            }

            // TODO: factor out this ugliness or change the datastructures
            // involved to be less gross
            let output = TechalyzerPrintOutput {
                symbol: data.symbol,
                indicator: indicator,
                map: m
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
    use super::SupportedIndicators;
    #[allow(unused_imports)] // TODO: remove this when tests are written
    use super::{run_program, Opts, SubCommands};

    #[test]
    fn test_print() {
        // Basic smoke test that print functionality works
        let _res = run_program(Opts {
            secret: None,
            symbol: "JPM".to_string(),
            start_date: None,
            end_date: None,
            cmd: SubCommands::Print {
                indicator: SupportedIndicators::RelativeStrengthIndex,
                print_signals: true,
            },
        });
    }
}
