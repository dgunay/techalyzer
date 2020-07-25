use alphavantage::time_series::TimeSeries;
use chrono::NaiveDate;

/// Errors arising from attempts to get data from different data sources
pub enum Error {
    AlphaVantageError(String),
}
// TODO: find a way to pretty print Error

pub trait DataSource {
    fn get(self, symbol: &str, start: NaiveDate, end: NaiveDate) -> Result<TimeSeries, Error>;
}
