pub mod datasources;
pub mod marketdata;
pub mod analysis;
pub mod secret;
pub mod source;
pub mod util;

use crate::datasources::alphavantage;
use crate::datasources::datasource::{DataSource, Error};
use crate::marketdata::stubmarketdata::StubMarketData;
use crate::source::Source;
use chrono::NaiveDate;
use secret::Secret;

/// TODO: document
pub fn get_market_data(
    source: Source,
    symbol: String,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    secret: Secret,
) -> Result<StubMarketData, Error> {
    let end = match end_date {
        Some(d) => d,
        None => chrono::Utc::now().naive_local().date(), // end at today's date
    };

    let start = match start_date {
        Some(d) => d,
        // FIXME: probably should just be None and let each source use its earliest
        // day
        None => NaiveDate::from_ymd(1901, 1, 1),
    };

    let market_data: StubMarketData = match source {
        Source::AlphaVantage => {
            let key = secret.data.unwrap_or("".to_string());
            let cl = ::alphavantage::blocking::Client::new(key.as_str());
            let av = alphavantage::AlphaVantage::new(cl);
            av.get(symbol.as_str(), start, end)?.into()
        }
    };

    return Ok(market_data);
}
