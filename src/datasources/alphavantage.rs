use crate::datasources::datasource::DataSource;
use alphavantage::blocking::Client; // TODO: use async client
use alphavantage::time_series::TimeSeries;
use chrono::NaiveDate;

pub struct AlphaVantage {
    client: Client,
}

impl AlphaVantage {
    pub fn new(client: Client) -> AlphaVantage {
        AlphaVantage { client: client }
    }
}

impl DataSource for AlphaVantage {
    fn get(self, symbol: &str, start: NaiveDate, end: NaiveDate) -> TimeSeries {
        let time_series = self.client.get_time_series_daily(symbol).unwrap();
        return time_series;
    }
}
