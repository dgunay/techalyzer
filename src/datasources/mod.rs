use std::path::PathBuf;
use std::str::FromStr;

use strum_macros::EnumIter;

pub mod alphavantage;
pub mod datasource;
pub mod techalyzerjson;

// TODO: god this is a mess, figure out a better way.

/// Data sources supported by Techalyzer, be they APIs or otherwise.
#[derive(Debug, EnumIter)]
pub enum SupportedDataSources {
    /// Get a file locally
    TechalyzerJson(PathBuf),

    /// Download data from the Alpha Vantage API
    AlphaVantage,
}

impl FromStr for SupportedDataSources {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alphavantage" => Ok(SupportedDataSources::AlphaVantage),
            "AlphaVantage" => Ok(SupportedDataSources::AlphaVantage),
            possible_file => {
                let buf = PathBuf::from_str(possible_file)
                    .expect("Should never fail (err type is Infallible)");
                Ok(SupportedDataSources::TechalyzerJson(buf))
            }
        }
    }
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_display_of_supported_datasources() {}
}
