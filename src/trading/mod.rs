//! Collection of different trading models using various methods.

pub mod buyandhold;
pub mod dtmodel;
pub mod manual;
pub mod tradingmodel;

use strum_macros::{Display, EnumString};

/// Can be an ML model or a handwritten algorithm.
#[derive(Debug, EnumString, Display)]
pub enum SupportedTradingModel {
    ManualTradingAlgo,
    BuyAndHold,
    MachineLearningModel,
}
