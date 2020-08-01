use std::path::PathBuf;
use std::str::FromStr;

pub mod alphavantage;
pub mod datasource;
pub mod techalyzerjson;

/// Data sources supported by Techalyzer
#[derive(Debug)]
pub enum SupportedDataSources {
    /// Get a file locally
    File(PathBuf),

    /// Download data from the Alpha Vantage API
    AlphaVantage,
}

#[derive(Debug)]
pub enum Error {
    NoSuchDataSource(String),
}

impl ToString for Error {
    // TODO: show all possible sources to the user to aid discoverability
    fn to_string(&self) -> String {
        match self {
            Error::NoSuchDataSource(src) => format!("No data source '{}'", src),
        }
    }
}

impl FromStr for SupportedDataSources {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alphavantage" => Ok(SupportedDataSources::AlphaVantage),
            possible_file => {
                // FIXME: is it ok to unwrap if the err type is Infallible?
                let buf = PathBuf::from_str(possible_file).unwrap();
                Ok(SupportedDataSources::File(buf))
            }
        }
    }
}
