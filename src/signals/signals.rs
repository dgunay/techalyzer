use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, slice::Iter};
use ta::Reset;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(transparent)]
pub struct Signal {
    pub val: f64,
}

impl From<f64> for Signal {
    fn from(f: f64) -> Self {
        Self::new(f)
    }
}

impl From<Signal> for f64 {
    fn from(s: Signal) -> Self {
        s.val
    }
}

impl From<Signal> for f32 {
    fn from(s: Signal) -> Self {
        s.val as f32
    }
}

// #[derive(Debug, Display)]
// pub enum SignalError {
//     #[display(fmt = "Signal value {} is not between -1.0 and 1.0", _0)]
//     InvalidSignalValue(f64),
// }

impl Signal {
    /// Creates a new Signal. Panics if it is out of range.
    pub fn new(val: f64) -> Self {
        debug_assert!((-1.0..=1.0).contains(&val));
        Self { val }
    }
}

/// buy/sell signals given by a technical indicator.
pub trait Signals {
    /// 1.0 for an absolute buy, -1.0 for an absolute short, 0.0 for do nothing.
    fn signals(&self) -> &Vec<Signal>;
    fn outputs(&self) -> &Vec<Output>;
    fn iter(&self) -> Iter<Output>;
}

/// Buy/sell signals, but done in a lazy fashion
pub trait SignalsIter: Reset {
    fn next(&mut self, price: f64) -> (Signal, Output);
}

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
