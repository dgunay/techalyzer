// TODO: use StructOpt or something to get args

use chrono::NaiveDate;
use structopt::StructOpt;
use ta_experiments::get_market_data;
use ta_experiments::secret::Secret;
use ta_experiments::source::Source;

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
}

fn main() -> Result<(), chrono::format::ParseError> {
    let opts = Opts::from_args();

    // TODO: default to all history available or use args
    // Date range for the data
    let start = opts.start_date;
    let end = opts.end_date;

    // API keys if necessary
    let secret = Secret { data: opts.secret };

    // Get market data
    get_market_data(Source::AlphaVantage, start, end, secret);

    Ok(())
}
