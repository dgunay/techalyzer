use alphavantage::time_series::TimeSeries;
use chrono::NaiveDate;

pub trait DataSource {
    fn get(self, symbol: &str, start: NaiveDate, end: NaiveDate) -> TimeSeries;
}
