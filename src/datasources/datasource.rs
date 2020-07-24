use chrono::NaiveDate;
use alphavantage::time_series::TimeSeries;

pub trait DataSource {
  fn get(self, symbol: &str, start: NaiveDate, end: NaiveDate) -> TimeSeries;
}