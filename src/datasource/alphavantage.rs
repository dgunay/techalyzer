//! Stock price information from [Alpha Vantage](https://www.alphavantage.co/).

use crate::datasource::{DataSource, Error};
use crate::marketdata::prices::Prices;
use alphavantage::blocking::Client;

/// Contains an Alpha Vantage client.
pub struct AlphaVantage {
    client: Client,
}

impl AlphaVantage {
    pub fn new(client: Client) -> AlphaVantage {
        AlphaVantage { client }
    }
}

/// Gets the data from the ALpha Vantage API.
impl DataSource for AlphaVantage {
    fn get(&self, symbol: &str) -> Result<Prices, Error> {
        // TODO: if start is in the last 100 market days, don't request the full
        // time series datas

        // FIXME: this doesn't use adjusted close, consider submitting a PR for
        // that.
        match self.client.get_time_series_daily_full(symbol) {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(Error::AlphaVantageError(e.to_string())),
        }
    }
}

#[cfg(test)]
pub mod tests {}
