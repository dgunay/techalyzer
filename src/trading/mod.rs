pub mod buyandhold;
pub mod dtmodel;
pub mod manual;
pub mod tradingmodel;

use strum_macros::{Display, EnumString};

// TODO: move this somewhere else
/// Can be an ML model or a handwritten algorithm.
#[derive(Debug, EnumString, Display)]
pub enum SupportedTradingModel {
    ManualTradingAlgo,
    BuyAndHold,
    MachineLearningModel,
}
