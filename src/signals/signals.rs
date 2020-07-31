use derive_more::Display;
use serde::Serialize;

/// buy/sell signals given by a technical indicator.
pub trait Signals {
    /// 1.0 for an absolute buy, -1.0 for an absolute short, 0.0 for do nothing.
    fn signals(&self) -> &Vec<f64>;
    fn outputs(&self) -> Outputs;
}

// TODO: consider making a way to stream serialization to json
// https://github.com/serde-rs/json/issues/345

/// Represents the outputs of a ta technical indicactor. Usually a sequence of
/// floats, sometimes a sequence of float tuples depending on the indicator.
#[derive(Serialize)]
pub struct Outputs {
    /// Outputs of a rust-ta technical indicator.
    pub outputs: Vec<Vec<f64>>,
    // TODO: is there a way we can make all the float arrays the same size
    // without a runtime check/const generics being unavailable?
    /// Name of what is at each index of the inner vector. e.g., for bollinger bands,
    /// might be:
    /// ["upper", "lower", "average"]
    ///
    /// whereas for RSI, it would just be a single string like:
    /// ["rsi_val"]
    pub mapping: Vec<String>,
}

#[derive(Debug, Display)]
pub enum OutputsError {
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

impl Outputs {
    /// Create a new Outputs. Each element of `outputs` must have the same
    /// number of elements as `mapping`.
    pub fn new(outputs: Vec<Vec<f64>>, mapping: Vec<String>) -> Result<Self, OutputsError> {
        for o in outputs.iter() {
            if o.len() != mapping.len() {
                return Err(OutputsError::MismatchedSizes {
                    output_len: o.len(),
                    mapping_len: mapping.len(),
                });
            }
        }

        Ok(Self { outputs, mapping })
    }
}
