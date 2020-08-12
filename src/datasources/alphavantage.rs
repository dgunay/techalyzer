use crate::datasources::datasource::{DataSource, Error};
use crate::Prices;
use alphavantage::time_series::Entry;
use crate::Date;

use alphavantage::blocking::Client;

pub struct AlphaVantage {
    client: Client,
}

impl AlphaVantage {
    pub fn new(client: Client) -> AlphaVantage {
        AlphaVantage { client }
    }
}

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

/// Helper function to convert the date of an Entry into a Date
pub fn entry_to_date(entry: Option<&Entry>) -> Date {
    entry.expect("No first Entry").date.naive_local().date().into()
}

#[cfg(test)]
pub mod tests {}
