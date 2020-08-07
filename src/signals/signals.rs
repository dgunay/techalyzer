use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// buy/sell signals given by a technical indicator.
pub trait Signals {
    /// 1.0 for an absolute buy, -1.0 for an absolute short, 0.0 for do nothing.
    fn signals(&self) -> &Vec<f64>;
    fn outputs(&self) -> &Vec<Output>;
}

// TODO: consider making a way to stream serialization to json
// https://github.com/serde-rs/json/issues/345

/// Represents a single point output of a ta technical indicactor. Usually a
/// float, sometimes a float tuple depending on the indicator.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Output {
    /// Keys are the name of the indicator feature, value is the value. e.g.
    /// BollingerBands may have "upper", "lower", and "average" values for each
    /// point on a time series.
    #[serde(flatten)]
    pub output: HashMap<String, f64>,
}

#[derive(Debug, Display)]
pub enum OutputError {
    #[display(
        fmt = "Output length {} does not match mapping length {}",
        output_len,
        mapping_len
    )]
    MismatchedSizes {
        output_len: usize,
        mapping_len: usize,
    },
}

impl Output {
    /// Create a new Outputs. Each element of `outputs` must have the same
    /// number of elements as `mapping`.
    pub fn new(outputs: Vec<f64>, mapping: Vec<String>) -> Result<Self, OutputError> {
        if outputs.len() != mapping.len() {
            return Err(OutputError::MismatchedSizes {
                output_len: outputs.len(),
                mapping_len: mapping.len(),
            });
        }

        let mut map = HashMap::new();
        for i in 0..outputs.len() {
            map.insert(mapping[i].clone(), outputs[i]);
        }

        Ok(Self { output: map })
    }
}
