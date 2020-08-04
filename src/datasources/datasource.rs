use crate::Prices;
use chrono::NaiveDate;
use derive_more::Display;

/// Errors arising from attempts to get data from different data sources
#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "{}", _0)]
    AlphaVantageError(String),

    #[display(fmt = "File {} not found", _0)]
    FileNotFound(String),

    #[display(fmt = "Symbol mismatch (expected {}, found {})", expected, actual)]
    SymbolMismatch { expected: String, actual: String },

    #[display(fmt = "Other error: {} (context: {})", _0, _1)]
    Other(String, String),
}

pub trait DataSource {
    fn get(
        self,
        symbol: &str,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
    ) -> Result<Prices, Error>;
}
