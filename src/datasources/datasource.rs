use crate::StubMarketData;
use chrono::NaiveDate;
use derive_more::Display;

/// Errors arising from attempts to get data from different data sources
#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "{}", _0)]
    AlphaVantageError(String),
}

pub trait DataSource {
    fn get(self, symbol: &str, start: NaiveDate, end: NaiveDate) -> Result<StubMarketData, Error>;
}
