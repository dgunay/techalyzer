#[deny(missing_docs)]
pub mod backtester;
pub mod datasource;
pub mod date;
pub mod error;
pub mod marketdata;
pub mod output;
pub mod secret;
pub mod signals;
pub mod subcommands;
pub mod trading;
pub mod util;

use crate::datasource::alphavantage;
use crate::datasource::techalyzerjson::TechalyzerJson;
use crate::datasource::SupportedDataSource;
use crate::datasource::{DataSource, Error};
use crate::marketdata::prices::Prices;
use ::alphavantage::blocking::Client;
use date::Date;
use secret::Secret;
use std::ops::RangeInclusive;

/// Gets stock price time series data from a given `SupportedDataSource`, within the given
/// date range. A Secret is used to access the data source, if necessary.
pub fn get_market_data(
    source: SupportedDataSource,
    symbol: String,
    date_range: RangeInclusive<Date>,
    secret: Secret,
) -> Result<Prices, Error> {
    let market_data: Prices = match source {
        SupportedDataSource::AlphaVantage => {
            // TODO: just make Secret contain an empty string with the Default trait
            let key = secret.data.unwrap_or_else(|| "".to_string());
            let cl = Client::new(key.as_str());
            let av = alphavantage::AlphaVantage::new(cl);
            av.get_date_range(symbol.as_str(), date_range)?
        }
        SupportedDataSource::TechalyzerJson(path) => {
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

    Ok(market_data)
}
