use crate::{
    backtester::performance::PerformanceError,
    trading::{
        buyandhold::BuyAndHoldError, dtmodel::DecisionTreeError, manual::ManualTradingModelError,
    },
};
use derive_more::{Display, From};

// use std::str::FromStr;

// TODO: unify error handling around this module

/// Errors produced by Techalyzer main program
#[derive(Debug, Display, From)]
pub enum TechalyzerError {
    #[display(fmt = "{}", _0)]
    Generic(String),
}

// impl FromStr for TechalyzerError {
//     type Err;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         todo!()
//     }
// }

// TODO: is there any way we can just do this, or do we need to wait for stabilization?
// impl From<T: ToString> for TechalyzerError {

// }

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
impl_techalyzer_error_from_stringable_type!(ManualTradingModelError);
impl_techalyzer_error_from_stringable_type!(DecisionTreeError);
impl_techalyzer_error_from_stringable_type!(std::io::Error);
impl_techalyzer_error_from_stringable_type!(bincode::Error);

// // TODO: maybe a macro would help reduce some of this repetition.
// impl From<serde_json::Error> for TechalyzerError {
//     fn from(e: serde_json::Error) -> Self {
//         e.to_string().into()
//     }
// }

// impl From<ta::errors::Error> for TechalyzerError {
//     fn from(e: ta::errors::Error) -> Self {
//         e.to_string().into()
//     }
// }

// impl From<PerformanceError> for TechalyzerError {
//     fn from(e: PerformanceError) -> Self {
//         e.to_string().into()
//     }
// }

// impl From<BuyAndHoldError> for TechalyzerError {
//     fn from(e: BuyAndHoldError) -> Self {
//         e.to_string().into()
//     }
// }

// impl From<ManualTradingModelError> for TechalyzerError {
//     fn from(e: ManualTradingModelError) -> Self {
//         e.to_string().into()
//     }
// }

// impl From<DecisionTreeError> for TechalyzerError {
//     fn from(e: DecisionTreeError) -> Self {
//         e.to_string().into()
//     }
// }

// impl From<std::io::Error> for TechalyzerError {
//     fn from(e: std::io::Error) -> Self {
//         e.to_string().into()
//     }
// }
