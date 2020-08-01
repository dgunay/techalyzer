pub mod datasources;
pub mod error;
pub mod marketdata;
pub mod output;
pub mod secret;
pub mod signals;
pub mod source;
pub mod util;

use crate::datasources::alphavantage;
use crate::datasources::datasource::{DataSource, Error};
use crate::datasources::techalyzerjson::TechalyzerJson;
use crate::marketdata::prices::Prices;
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
) -> Result<Prices, Error> {
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

    let market_data: Prices = match source {
        Source::AlphaVantage => {
            let key = secret.data.unwrap_or("".to_string());
            let cl = ::alphavantage::blocking::Client::new(key.as_str());
            let av = alphavantage::AlphaVantage::new(cl);
            av.get(symbol.as_str(), start, end)?
        }
        Source::TechalyzerJson(path) => {
            if !path.exists() {
                // FIXME: don't unwrap
                return Err(Error::FileNotFound(
                    path.into_os_string().into_string().unwrap(),
                ));
            }

            match TechalyzerJson::new(path.as_path()) {
                Ok(t) => t.get(symbol.as_str(), start, end)?,
                Err(io_err) => {
                    return Err(Error::Other(
                        io_err.to_string(),
                        format!("Tried to open {:?}", path),
                    ))
                }
            }
        }
    };

    return Ok(market_data);
}
