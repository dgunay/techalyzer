use chrono::NaiveDate;
use derive_more::Display;
use structopt::StructOpt;
use ta_experiments::get_market_data;
use ta_experiments::secret::Secret;
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
        signals: bool,
    },
    Train {},
    Suggest {},
}

#[derive(Debug)]
enum SupportedIndicators {
    BollingerBands,
    RelativeStrengthIndex,
    MACD
}

impl std::str::FromStr for SupportedIndicators {
    type Err = TechalyzerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bollinger-bands" => Ok(SupportedIndicators::BollingerBands),
            "rsi" => Ok(SupportedIndicators::RelativeStrengthIndex),
            "macd" => Ok(SupportedIndicators::MACD),
            _ => Err(TechalyzerError::Generic(format!("{} is not a supported technical indicator", s)))
        }
    }
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
            signals
        } =>  {
            todo!()   
        }
        SubCommands::Suggest {} => todo!(),
        SubCommands::Train {} => todo!(),
    }
}

#[cfg(test)]
mod tests {
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
