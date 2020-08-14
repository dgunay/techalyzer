use crate::{
    backtester::performance::PerformanceError,
    output::SupportedIndicators,
    trading::{
        buyandhold::BuyAndHoldError, dtmodel::DecisionTreeError, manual::ManualTradingModelError,
    },
};
use derive_more::{Display, From};
use strum::VariantNames;

// use std::str::FromStr;

// TODO: unify error handling around this module

/// Errors produced by Techalyzer main program
#[derive(Debug, Display, From)]
pub enum TechalyzerError {
    #[display(fmt = "{}", _0)]
    Generic(String),

    #[display(
        fmt = "Must include at least one signal generator (supported: {})",
        "list_of_indicators()"
    )]
    NoIndicatorSpecified,

    #[display(fmt = "Please supply a model file.")]
    NoModelFileSpecified,
}

fn list_of_indicators() -> String {
    let v = Vec::from(SupportedIndicators::VARIANTS);
    v.join(", ")
}

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
impl_techalyzer_error_from_stringable_type!(dg_ta::errors::Error);
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

// impl From<dg_ta::errors::Error> for TechalyzerError {
//     fn from(e: dg_ta::errors::Error) -> Self {
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
