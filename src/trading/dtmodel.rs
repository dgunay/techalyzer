use super::tradingmodel::{Trades, TradingModel};
use crate::{marketdata::prices::Prices, signals::signals::SignalsIter};
use chrono::NaiveDate;
use derive_more::Display;
use rustlearn::prelude::*;
use rustlearn::trees::decision_tree::DecisionTree;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DecisionTreeTrader<'a> {
    model: DecisionTree,

    // TODO: solve these problems to have arbitrary injectable signal generators:
    // - TODO: not yet sure how to serialize
    // - TODO: can we "reset" the generator between train and test runs?
    // - TODO: Can we rewrite the TradingModel trait so that we don't have to
    //         cheat with RefCell to mutate our signal generators?
    #[serde(skip)]
    // signal_generators: RefCell<Vec<&'a mut dyn SignalsIter>>,
    signal_generators: Vec<&'a mut dyn SignalsIter>,
}

#[derive(Display, Debug)]
pub enum DecisionTreeError<'a> {
    #[display(fmt = "No price information found looking ahead at day {}", _0)]
    NoLookAheadPriceData(NaiveDate),

    #[display(fmt = "Error while fitting model: {}", _0)]
    TrainingError(&'a str),
}

impl<'a> DecisionTreeTrader<'a> {
    pub fn new(model: DecisionTree, signal_generators: Vec<&'a mut dyn SignalsIter>) -> Self {
        Self {
            model,
            signal_generators: signal_generators.into(),
        }
    }

    /// Trains the model using technical indicator signal generators for the
    /// given Prices time series.
    pub fn train(&mut self, train_prices: &Prices, horizon: u32) -> Result<(), DecisionTreeError> {
        // Get our indicators
        // let signals = Vec::new();
        let mut x = Vec::new();
        let mut y = Vec::new();

        for (day, price) in train_prices.iter() {
            let signals: Vec<f32> = self.next_signals(price);

            // look ahead for n-day future return
            let future_day = *day + chrono::Duration::days(horizon as i64);
            let future_price = train_prices
                .get(&future_day)
                .ok_or(DecisionTreeError::NoLookAheadPriceData(future_day))?;
            let future_return = ((future_price / price) - 1.0) as f32;

            // X is the signals, Y is the future return
            // TODO: how do we structure it for rustlearn
            x.push(signals);
            y.push(future_return);
        }

        // Construct X train, Y train data out of the prices
        self.model
            .fit(&Array::from(&x), &y.into())
            .map_err(|msg| DecisionTreeError::TrainingError(msg))
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

impl<'a> TradingModel for DecisionTreeTrader<'a> {
    type Error = DecisionTreeError<'a>;

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
                Err(msg) => return Err(DecisionTreeError::TrainingError(msg)),
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
