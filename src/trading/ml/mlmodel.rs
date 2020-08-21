// TODO: is it worth using a generic T?
use derive_more::Display;

/// Wrapper interface for using a vendor's machine learning algorithm.
/// `T` is a row type collection.
pub trait MachineLearningAlgorithm<T> {
    fn fit(&mut self, x: &Vec<T>, y: &T) -> Result<(), Error>;
    fn predict(&self, x: &Vec<T>) -> Result<Vec<T>, Error>;
}

/// Errors that can happen during machine learning training or prediction.
/// Mostly just string wrappers.

#[derive(Display)]
pub enum Error {
    #[display(fmt = "Error fitting model: {}", _0)]
    FitError(String),

    #[display(fmt = "Error during prediction: {}", _0)]
    PredictionError(String),
}
