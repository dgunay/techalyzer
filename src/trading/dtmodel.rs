use super::tradingmodel::{Trades, TradingModel};
use crate::{
    marketdata::prices::Prices,
    signals::signals::{Signal, Signals, SignalsIter},
};
use chrono::NaiveDate;
use derive_more::Display;
use rustlearn::prelude::*;
use rustlearn::trees::decision_tree::{DecisionTree, Hyperparameters};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Serialize, Deserialize)]
struct DecisionTreeTrader<'a> {
    model: DecisionTree,

    // TODO: solve these problems to have arbitrary injectable signal generators:
    // - TODO: not yet sure how to serialize
    // - TODO: can we "reset" the generator between train and test runs?
    // - TODO: Can we rewrite the TradingModel trait so that we don't have to
    //         cheat to mutate our signal generators?
    #[serde(skip)]
    signal_generators: RefCell<Vec<&'a mut dyn SignalsIter>>,
}

#[derive(Display)]
pub enum Error<'a> {
    #[display(fmt = "No price information found looking ahead at day {}", _0)]
    NoLookAheadPriceData(NaiveDate),

    #[display(fmt = "Error while fitting model: {}", _0)]
    TrainingError(&'a str),
}

impl<'a> DecisionTreeTrader<'a> {
    pub fn new(model: DecisionTree, signal_generators: Vec<&'a mut dyn SignalsIter>) -> Self {
        Self {
            model,
            signal_generators,
        }
    }

    /// Trains the model using technical indicator signal generators for the
    /// given Prices time series.
    pub fn train(&mut self, train_prices: &Prices, horizon: u32) -> Result<(), Error> {
        // Get our indicators
        // let signals = Vec::new();
        let mut X = Vec::new();
        let mut y = Vec::new();

        for (day, price) in train_prices.iter() {
            let signals: Vec<f32> = self.next_signals(price);

            // look ahead for n-day future return
            let future_day = *day + chrono::Duration::days(horizon as i64);
            let future_price = train_prices
                .get(&future_day)
                .ok_or(Error::NoLookAheadPriceData(future_day))?;
            let future_return = ((future_price / price) - 1.0) as f32;

            // X is the signals, Y is the future return
            // TODO: how do we structure it for rustlearn
            X.push(signals);
            y.push(future_return);
        }

        // Construct X train, Y train data out of the prices
        self.model
            .fit(&Array::from(&X), &y.into())
            .map_err(|msg| Error::TrainingError(msg))
    }

    /// Gets the next set of signals from the signal generators
    fn next_signals(&self, price: &f64) -> Vec<f32> {
        self.signal_generators
            .borrow_mut()
            .iter_mut()
            .map(|g| f32::from(g.next(*price).0))
            .collect()
    }
}

impl<'a> TradingModel for DecisionTreeTrader<'a> {
    fn get_trades(&self, prices: &Prices) -> Trades {
        // Reset our technical indicators

        // Given each day and it's technical indicators, predict the return and
        // act accordingly

        for (day, price) in prices.iter() {
            let signals: Vec<f32> = self.next_signals(price);
            let predicted_return = self.model.predict(&Array::from(signals));
        }

        todo!()
    }
}
