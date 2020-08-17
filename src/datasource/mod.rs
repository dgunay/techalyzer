//! Code for getting stock price information from a variety of sources.

use crate::{date::Date, marketdata::prices::Prices};
use derive_more::Display;
use std::path::PathBuf;
use std::{ops::RangeBounds, str::FromStr};
use strum_macros::EnumIter;

pub mod alphavantage;
pub mod techalyzerjson;

/// Data sources supported by Techalyzer, be they APIs or otherwise.
#[derive(Debug, EnumIter)]
pub enum SupportedDataSource {
    /// Get a file locally
    TechalyzerJson(PathBuf),

    /// Download data from the Alpha Vantage API
    AlphaVantage,
}

impl FromStr for SupportedDataSource {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alphavantage" => Ok(SupportedDataSource::AlphaVantage),
            "AlphaVantage" => Ok(SupportedDataSource::AlphaVantage),
            possible_file => {
                // TODO: when stabilized, use into_ok()
                let buf = PathBuf::from_str(possible_file)
                    .expect("Should never fail (err type is Infallible)");
                Ok(SupportedDataSource::TechalyzerJson(buf))
            }
        }
    }
}

/// Errors arising from attempts to get data from different data sources
#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "{}", _0)]
    AlphaVantageError(String),

    #[display(fmt = "File '{}' not found", _0)]
    FileNotFound(String),

    #[display(fmt = "Symbol mismatch (expected {}, found {})", expected, actual)]
    SymbolMismatch { expected: String, actual: String },

    #[display(fmt = "'{}' is not a supported data source", _0)]
    NoSuchDataSource(String),

    #[display(fmt = "Other error: {} (context: {})", _0, _1)]
    Other(String, String),
}

/// Interface for retrieving stock prices from a data source.
pub trait DataSource {
    /// Gets the full range of price data
    fn get(&self, symbol: &str) -> Result<Prices, Error>;

    /// Gets price data in a date range.
    fn get_date_range(&self, symbol: &str, range: impl RangeBounds<Date>) -> Result<Prices, Error> {
        let prices = self.get(symbol)?;
        Ok(prices.date_range(range))
    }
}
