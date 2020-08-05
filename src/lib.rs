pub mod backtester;
pub mod datasources;
pub mod error;
pub mod marketdata;
pub mod output;
pub mod secret;
pub mod signals;
pub mod util;

use crate::datasources::alphavantage;
use crate::datasources::datasource::{DataSource, Error};
use crate::datasources::techalyzerjson::TechalyzerJson;
use crate::datasources::SupportedDataSources;
use crate::marketdata::prices::Prices;
use chrono::NaiveDate;
use secret::Secret;

/// Gets stock price time series data from a given Source, from start to end
/// date. A Secret is used to access the data source, if necessary.
pub fn get_market_data(
    source: SupportedDataSources,
    symbol: String,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    secret: Secret,
) -> Result<Prices, Error> {
    let market_data: Prices = match source {
        SupportedDataSources::AlphaVantage => {
            let key = secret.data.unwrap_or("".to_string());
            let cl = ::alphavantage::blocking::Client::new(key.as_str());
            let av = alphavantage::AlphaVantage::new(cl);
            av.get(symbol.as_str(), start_date, end_date)?
        }
        SupportedDataSources::TechalyzerJson(path) => {
            if !path.exists() {
                return Err(Error::FileNotFound(
                    path.into_os_string().into_string().expect("invalid string"),
                ));
            }

            match TechalyzerJson::new(path.as_path()) {
                Ok(t) => t.get(symbol.as_str(), start_date, end_date)?,
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
