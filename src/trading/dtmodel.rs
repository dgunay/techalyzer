use super::tradingmodel::{Trades, TradingModel};
use crate::Date;
use crate::{marketdata::prices::Prices, signals::signals::SignalsIter};
use derive_more::Display;
use rustlearn::prelude::*;
use rustlearn::trees::decision_tree::DecisionTree;
use serde::{Deserialize, Serialize};

// TODO: Try doing this to make forgetting to train the model a compile-time error:
// https://stackoverflow.com/questions/42036826/using-the-rust-compiler-to-prevent-forgetting-to-call-a-method

#[derive(Serialize, Deserialize)]
pub struct DecisionTreeTrader {
    model: DecisionTree,

    // TODO: solve these problems to have arbitrary injectable signal generators:
    // - TODO: not yet sure how to serialize
    // - TODO: can we "reset" the generator between train and test runs?
    // - TODO: Can we rewrite the TradingModel trait so that we don't have to
    //         cheat with RefCell to mutate our signal generators?
    #[serde(skip)]
    // signal_generators: Vec<&'a mut dyn SignalsIter>,
    signal_generators: Vec<Box<dyn SignalsIter>>,
}

#[derive(Display, Debug)]
pub enum DecisionTreeError {
    #[display(
        fmt = "No price information found looking ahead {} days after {}",
        _0,
        _1
    )]
    NoLookAheadPriceData(u32, Date),

    #[display(fmt = "Error while fitting model: {}", _0)]
    TrainingError(String),

    #[display(fmt = "No price found on date {}", _0)]
    NoPriceFound(Date),
}

const LONG: f32 = 1.0;
const SHORT: f32 = 0.0;
// const SHORT: f32 = -1.0;

impl DecisionTreeTrader {
    // pub fn new(model: DecisionTree, signal_generators: Vec<&'a mut dyn SignalsIter>) -> Self {
    pub fn new(model: DecisionTree, signal_generators: Vec<Box<dyn SignalsIter>>) -> Self {
        Self {
            model,
            signal_generators,
        }
    }

    /// Trains the model using technical indicator signal generators for the
    /// given Prices time series.
    ///
    /// ## Arguments
    ///
    /// * `train_prices` - Prices time series to train the model on.
    /// * `train_dates` - Range of dates to train on. Should be `horizon` less
    /// than the dates in `train_prices`.
    /// * `horizon` - Uses the returns this many days in the future for Y.
    /// * `threshold` - How good (or bad) the returns have to be for the model to go Long or Short.
    pub fn train(
        &mut self,
        train_prices: &Prices,
        train_dates: Vec<Date>,
        horizon: u32,
        _threshold: f32,
    ) -> Result<(), DecisionTreeError> {
        let mut x = Vec::new();
        let mut y = Vec::new();

        // for day in train_prices.iter() {
        for day in train_dates {
            let price = train_prices
                .get(&day)
                .ok_or(DecisionTreeError::NoPriceFound(day))?;

            let signals: Vec<f32> = self.next_signals(price);

            // look ahead for n-day future return
            let future_price = train_prices
                .get_after(&day, horizon)
                .ok_or(DecisionTreeError::NoLookAheadPriceData(horizon, day))?
                .1;
            let future_return = ((future_price / price) - 1.0) as f32;

            // Returns above threshold is Long, below is Short, otherwise Out.
            // TODO: change back to this when we have a multiclass classifier
            // let label = match future_return {
            //     r if r >= threshold => LONG,
            //     r if r <= -threshold => SHORT,
            //     _ => OUT
            // };
            let label = match future_return {
                r if r >= 0.0 => LONG,
                r if r < 0.0 => SHORT,
                _ => LONG,
            };

            // X is the signals, Y our long/short/out decision based on future return
            // TODO: how do we structure it for rustlearn
            x.push(signals);
            y.push(label);
        }

        // Construct X train, Y train data out of the prices
        self.model
            .fit(&Array::from(&x), &y.into())
            .map_err(|msg| DecisionTreeError::TrainingError(msg.to_string()))
    }

    /// Gets the next set of signals from the signal generators
    fn next_signals(&mut self, price: &f64) -> Vec<f32> {
        self.signal_generators
            // .borrow_mut()
            .iter_mut()
            .map(|g| f32::from(g.next(*price).0))
            .collect()
    }
}

impl TradingModel for DecisionTreeTrader {
    type Error = DecisionTreeError;

    fn get_trades(mut self, prices: &Prices) -> Result<Trades, Self::Error> {
        // Reset our technical indicators
        self.signal_generators.iter_mut().for_each(|g| g.reset());

        // Given each day and it's technical indicators, predict the return and
        // act accordingly
        for (_day, price) in prices.iter() {
            let signals: Vec<f32> = self.next_signals(price);
            // TODO: start submitting PRs to improve rustlearn, it has no
            // error enums for one thing
            let predicted_return = match self.model.predict(&Array::from(signals)) {
                Ok(r) => r,
                Err(msg) => return Err(DecisionTreeError::TrainingError(msg.to_string())),
            };

            print!("{:?}", predicted_return);
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_test() {
        // Can we make it run?
        todo!("write a smoke test for DecisionTreeTrader")
    }
}
