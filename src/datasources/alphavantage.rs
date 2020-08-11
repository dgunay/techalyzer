use crate::datasources::datasource::{DataSource, Error};
use crate::Prices;
use alphavantage::time_series::Entry;
use chrono::NaiveDate;

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

/// Helper function to convert the date of an Entry into a NaiveDate
pub fn entry_to_naivedate(entry: Option<&Entry>) -> NaiveDate {
    entry.expect("No first Entry").date.naive_local().date()
}

#[cfg(test)]
pub mod tests {}
