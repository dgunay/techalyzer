//! Signal generators that use technical indicators to generate bullish and
//! bearish signals in order to inform trading models.
//!
//! Technical indicators are provided by [ta-rs](https://github.com/dgunay/ta-rs), currently forked to support
//! serde serialization and other features.

pub mod bollingerbandssignals;
pub mod macdsignals;
pub mod relativestrengthindexsignals;

use derive_more::Display;
use dg_ta::Reset;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

/// Thin wrapper for a float - represents a bullish or bearish signal.
///
/// Signal is runtime checked in debug builds to be between -1.0 and 1.0
/// inclusive (the expected range for signal generators to be outputting).
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

impl From<&f64> for Signal {
    fn from(f: &f64) -> Self {
        Self::new(*f)
    }
}

impl From<Signal> for f64 {
    fn from(s: Signal) -> Self {
        s.val
    }
}

impl From<&Signal> for f64 {
    fn from(s: &Signal) -> Self {
        s.val
    }
}

impl From<Signal> for f32 {
    fn from(s: Signal) -> Self {
        s.val as f32
    }
}

impl Signal {
    /// Creates a new Signal. Panics if it is out of range.
    pub fn new(val: f64) -> Self {
        debug_assert!((-1.0..=1.0).contains(&val));
        Self { val }
    }
}

/// Iteratively generates buy/sell signals.
#[typetag::serde(tag = "type")] // allows Box<dyn SignalsIter> to work with serde.
pub trait SignalsIter: Reset + Debug {
    /// Return a tuple of the next Signal and technical indicator Output.
    fn next(&mut self, price: f64) -> (Signal, Output);
}

/// Represents a single point output of a ta technical indicator. Usually a
/// float, sometimes a float tuple depending on the indicator.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Output {
    /// Keys are the name of the indicator feature, value is the value. e.g.
    /// BollingerBands may have "upper", "lower", and "average" values for each
    /// point on a time series.
    #[serde(flatten)]
    pub output: HashMap<String, f64>,
}

/// Errors that can occur while using Output (mainly mapping length mismatches).
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
    /// Create a new Output. Each element of `outputs` must have the same
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

impl From<f64> for Output {
    fn from(f: f64) -> Self {
        Output {
            output: [("rsi".to_string(), f)].iter().cloned().collect(),
        }
    }
}
