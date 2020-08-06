#[warn(missing_docs)]
pub mod backtester;
pub mod datasources;
pub mod error;
pub mod marketdata;
pub mod output;
pub mod secret;
pub mod signals;
pub mod trading;
pub mod util;

use crate::datasources::alphavantage;
use crate::datasources::datasource::{DataSource, Error};
use crate::datasources::techalyzerjson::TechalyzerJson;
use crate::datasources::SupportedDataSources;
use crate::marketdata::prices::Prices;
use chrono::NaiveDate;
use secret::Secret;
use std::ops::RangeInclusive;

use ::alphavantage::blocking::Client;

/// Gets stock price time series data from a given Source, within the given
/// date range. A Secret is used to access the data source, if necessary. For
/// and open-ended date range, use None as one of the bounds.
pub fn get_market_data(
    source: SupportedDataSources,
    symbol: String,
    date_range: RangeInclusive<NaiveDate>,
    secret: Secret,
) -> Result<Prices, Error> {
    let market_data: Prices = match source {
        SupportedDataSources::AlphaVantage => {
            let key = secret.data.unwrap_or("".to_string());
            let cl = Client::new(key.as_str());
            let av = alphavantage::AlphaVantage::new(cl);
            av.get_date_range(symbol.as_str(), date_range)?
        }
        SupportedDataSources::TechalyzerJson(path) => {
            if !path.exists() {
                return Err(Error::FileNotFound(
                    path.into_os_string().into_string().expect("invalid string"),
                ));
            }

            match TechalyzerJson::new(path.as_path()) {
                Ok(t) => t.get_date_range(symbol.as_str(), date_range)?,
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
