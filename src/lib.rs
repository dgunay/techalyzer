// TODO: library code goes here

mod datasources;
pub mod marketdata;
pub mod secret;
pub mod source;

use crate::datasources::alphavantage;
use crate::datasources::datasource::DataSource;
use crate::marketdata::stubmarketdata::StubMarketData;
use crate::source::Source;
use chrono::NaiveDate;
use secret::Secret;

pub fn get_market_data(
    source: Source,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    secret: Secret,
) -> StubMarketData {
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

    let market_data = match source {
        Source::AlphaVantage => {
            let key = secret.data.unwrap();
            let cl = ::alphavantage::blocking::Client::new(key.as_str());
            let av = alphavantage::AlphaVantage::new(cl);
            let res = av.get("JPM", start, end);
            StubMarketData {}
        }
    }

    return market_data;
}
