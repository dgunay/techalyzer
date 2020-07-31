use crate::datasources::datasource::{DataSource, Error};
use crate::Prices;
use alphavantage::blocking::Client; // TODO: use async client
use chrono::NaiveDate;

pub struct AlphaVantage {
    client: Client,
}

impl AlphaVantage {
    pub fn new(client: Client) -> AlphaVantage {
        AlphaVantage { client }
    }
}

impl DataSource for AlphaVantage {
    fn get(self, symbol: &str, start: NaiveDate, end: NaiveDate) -> Result<Prices, Error> {
        match self.client.get_time_series_daily(symbol) {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(Error::AlphaVantageError(e.to_string())),
        }
    }
}
