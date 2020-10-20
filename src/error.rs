//! Provides a top-level error enum for Techalyzer. Anything that implements
//! ToString or Display can be converted into a TechalyzerError.
//!
//! Works using a macro to write a FromStr implementation for a list of given
//! types. Probably won't be necessary anymore when
//! [associated type bounds](https://rust-lang.github.io/rfcs/2289-associated-type-bounds.html)
//! are stabilized (can just write `impl FromStr<T: ToString> for TechalyzerError`).

use crate::{
    backtester::{performance::PerformanceError, BackTesterError},
    indicators::SupportedIndicators,
    marketdata::prices::PricesError,
    trading::{buyandhold::BuyAndHoldError, dtmodel::DecisionTreeError, manual::CanNeverHappen},
};
use derive_more::From;
use strum::VariantNames;
use thiserror::Error;

/// Errors produced by Techalyzer main program
#[derive(Debug, From, Error)]
pub enum TechalyzerError {
    #[error("{0}")]
    Generic(String),

    #[error(
        "Must include at least one signal generator (supported: {})",
        list_of_indicators()
    )]
    NoIndicatorSpecified,

    #[error("Please supply a model file.")]
    NoModelFileSpecified,
}

fn list_of_indicators() -> String {
    let v = Vec::from(SupportedIndicators::VARIANTS);
    v.join(", ")
}

// TODO: is this necessary if we use anyhow + thiserror?

/// Makes a From<T: ToString> implementation for TechalyzerError
macro_rules! impl_techalyzer_error_from_stringable_type {
    ($type:ty) => {
        impl From<$type> for TechalyzerError {
            fn from(e: $type) -> Self {
                e.to_string().into()
            }
        }
    };
}

impl_techalyzer_error_from_stringable_type!(serde_json::Error);
impl_techalyzer_error_from_stringable_type!(ta::errors::Error);
impl_techalyzer_error_from_stringable_type!(PerformanceError);
impl_techalyzer_error_from_stringable_type!(BuyAndHoldError);
impl_techalyzer_error_from_stringable_type!(CanNeverHappen);
impl_techalyzer_error_from_stringable_type!(DecisionTreeError);
impl_techalyzer_error_from_stringable_type!(std::io::Error);
impl_techalyzer_error_from_stringable_type!(bincode::Error);
impl_techalyzer_error_from_stringable_type!(bincode::ErrorKind);
impl_techalyzer_error_from_stringable_type!(BackTesterError);
impl_techalyzer_error_from_stringable_type!(PricesError);
impl_techalyzer_error_from_stringable_type!(crate::trading::ml::mlmodel::Error);
