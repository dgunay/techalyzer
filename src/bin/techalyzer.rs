use chrono::NaiveDate;
use derive_more::Display;
use serde::Serialize;
use structopt::StructOpt;
use ta::indicators::*;
use ta_experiments::get_market_data;
use ta_experiments::secret::Secret;
use ta_experiments::signals::{
    bollingerbandssignals::BollingerBandsSignals,
    macdsignals::MovingAverageConvergenceDivergenceSignals,
    relativestrengthindexsignals::RelativeStrengthIndexSignals, signals::Outputs, signals::Signals,
};
use ta_experiments::source::Source;

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

#[derive(Debug)]
enum SupportedIndicators {
    BollingerBands,
    RelativeStrengthIndex,
    MACD,
}

impl std::str::FromStr for SupportedIndicators {
    type Err = TechalyzerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bollinger-bands" => Ok(SupportedIndicators::BollingerBands),
            "rsi" => Ok(SupportedIndicators::RelativeStrengthIndex),
            "macd" => Ok(SupportedIndicators::MACD),
            _ => Err(TechalyzerError::Generic(format!(
                "{} is not a supported technical indicator",
                s
            ))),
        }
    }
}

/// Dynamically dispatch to a ta::indicators::* from our enum
// impl From<SupportedIndicators> for Box<dyn ta::Next<f64, Output = f64>> {
//     fn from(s: SupportedIndicators) -> Self {
//         match s {
//             SupportedIndicators::BollingerBands => Box::new(ta::indicators::BollingerBands::new(14, 2.0).unwrap()),
//             SupportedIndicators::RelativeStrengthIndex => {}
//             SupportedIndicators::MACD => {}
//         }
//     }
// }

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

#[derive(Debug, Display)]
enum TechalyzerError {
    #[display(fmt = "{}", _0)]
    Generic(String),
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
            print_signals,
        } => {
            // TODO: evaluate/benchmark signal generation using ndarray vs Vec<f64>
            let prices: Vec<f64> = data.prices.prices.to_vec();

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

            // TODO: factor out this ugliness or change the datastructures
            // involved to be less gross
            let output = TechalyzerPrintOutput {
                symbol: data.prices.symbol,
                signals: sigs.signals(),
                outputs: if print_signals {
                    Some(sigs.outputs())
                } else {
                    None
                },
                dates: data.prices.dates.map(|d| d.naive_local().date()).to_vec(),
            };

            print_techalyzer_json(&output);
        }
        SubCommands::Suggest {} => todo!(),
        SubCommands::Train {} => todo!(),
    }

    Ok(())
}

// TODO: a map structure of date => [price, signals, outputs] would be very useful
// for charting and otherwise using the data.
/// Organizes our data the way we want before printing.
#[derive(Serialize)]
struct TechalyzerPrintOutput<'a> {
    symbol: String,
    dates: Vec<NaiveDate>,
    signals: &'a Vec<f64>,
    outputs: Option<Outputs>,
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
    #[allow(unused_imports)] // TODO: remove this when tests are written
    use super::{run_program, Opts, SubCommands};

    #[test]
    fn test_main() {
        todo!("write some kind of integration tests for the whole program");
        // ()
        // let res = run_program(Opts {
        //     secret: None,
        //     symbol: "JPM".to_string(),
        //     start_date: None,
        //     end_date: None,
        // });
    }
}
