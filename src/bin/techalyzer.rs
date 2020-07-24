// TODO: use StructOpt or something to get args

use ta_experiments::get_market_data;
use ta_experiments::source::Source;
use chrono::NaiveDate;
use ta_experiments::secret::Secret;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opts {
    #[structopt(long)]
    secret: Option<String>,

    symbol: String,

    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
}

fn main() -> Result<(), chrono::format::ParseError> {
    let opts = Opts::from_args();

    // TODO: default to all history available or use args
    // Date range for the data
    let start = NaiveDate::parse_from_str("2020-01-01", "%Y-%m-%d")?;
    let today = chrono::Utc::now().naive_local().date();
    let end = today;
    
    // API keys if necessary
    let secret = Secret { data: opts.secret };

    // Get market data
    get_market_data(Source::AlphaVantage, start, end, secret);

    Ok(())
}
