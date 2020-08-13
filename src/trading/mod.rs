pub mod buyandhold;
pub mod dtmodel;
pub mod manual;
pub mod tradingmodel;

use std::path::PathBuf;
use strum_macros::{Display, EnumString};

// TODO: move this somewhere else
/// Can be an ML model or a handwritten algorithm.
#[derive(Debug, EnumString, Display)]
pub enum SupportedTradingModel {
    ManualTradingAlgo,
    BuyAndHold,
    MachineLearningModel(PathBuf),
}

// impl Display for SupportedTradingModel {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             SupportedTradingModel::ManualTradingAlgo => write!(f, self.)
//             SupportedTradingModel::BuyAndHold => {}
//             SupportedTradingModel::MachineLearningModel(_) => {}
//         }
//     }
// }
