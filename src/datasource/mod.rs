//! Code for getting stock price information from a variety of sources.

use crate::{date::Date, marketdata::prices::Prices};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{ops::RangeBounds, str::FromStr};
use strum_macros::EnumIter;
use thiserror::Error;

pub mod alphavantage;
pub mod csv;
pub mod techalyzerjson;

/// Data sources supported by Techalyzer, be they APIs or otherwise.
#[derive(Debug, EnumIter, Deserialize, Serialize, PartialEq)]
pub enum SupportedDataSource {
    /// Get a file locally
    TechalyzerJson(PathBuf),

    /// Download data from the Alpha Vantage API
    AlphaVantage,

    /// Use a CSV file.
    CsvFile(PathBuf),
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

                // FIXME: disgusting and inelegant, try and clean this up and
                // make it not panic.
                let extension = buf
                    .extension()
                    .expect(
                        format!("File '{}' has no file extension", buf.to_str().unwrap()).as_str(),
                    )
                    .to_os_string();
                if extension == "json" {
                    Ok(SupportedDataSource::TechalyzerJson(buf))
                } else if extension == "csv" {
                    Ok(SupportedDataSource::CsvFile(buf))
                } else {
                    Err(Error::Other {
                        msg: format!(
                            "{} does not end with either .json or .csv",
                            buf.to_str().expect("Failed to read pathbuf as str")
                        ),
                        context: format!("ends with {}", extension.to_str().unwrap()),
                    })
                }
            }
        }
    }
}

// TODO: this error enum is causing a lot of architectural problems - should we
// switch to an error crate or reorganize things?
/// Errors arising from attempts to get data from different data sources
#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    AlphaVantageError(String),

    #[error("File '{0}' not found")]
    FileNotFound(String),

    #[error("Symbol mismatch (expected {expected:?}, found {actual:?})")]
    SymbolMismatch { expected: String, actual: String },

    #[error("'{0}' is not a supported data source")]
    NoSuchDataSource(String),

    #[error("Error with CSV file: {0}")]
    CsvError(String),

    #[error("Other error: {msg:?} (context: {context:?})")]
    Other { msg: String, context: String },
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
