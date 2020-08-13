use crate::marketdata::prices::Prices;
use crate::Date;
use derive_more::Display;

use std::ops::RangeBounds;

/// Errors arising from attempts to get data from different data sources
#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "{}", _0)]
    AlphaVantageError(String),

    #[display(fmt = "File '{}' not found", _0)]
    FileNotFound(String),

    #[display(fmt = "Symbol mismatch (expected {}, found {})", expected, actual)]
    SymbolMismatch { expected: String, actual: String },

    #[display(fmt = "Other error: {} (context: {})", _0, _1)]
    Other(String, String),
}

pub trait DataSource {
    /// Gets the full range of price data
    fn get(&self, symbol: &str) -> Result<Prices, Error>;

    /// Gets price data in a date range.
    fn get_date_range(&self, symbol: &str, range: impl RangeBounds<Date>) -> Result<Prices, Error> {
        let prices = self.get(symbol)?;
        Ok(prices.date_range(range))
    }
}

#[cfg(test)]
mod tests {}
