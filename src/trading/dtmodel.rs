use super::tradingmodel::{Trades, TradingModel};
use crate::Date;
use crate::{backtester::Position, marketdata::prices::Prices, signals::signals::SignalsIter};
use derive_more::Display;
use rustlearn::prelude::*;
use rustlearn::trees::decision_tree::DecisionTree;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ta::Next;

// TODO: Try doing this to make forgetting to train the model a compile-time error:
// https://stackoverflow.com/questions/42036826/using-the-rust-compiler-to-prevent-forgetting-to-call-a-method

#[derive(Serialize, Deserialize)]
pub struct DecisionTreeTrader {
    model: DecisionTree,

    // TODO: solve these problems to have arbitrary injectable signal generators:
    // how can we keep runtime polymorphism and have serde work?
    // maybe try
    // signal_generators: Vec<&'a mut dyn SignalsIter>,
    #[serde(skip)] // FIXME: remove when serde capability is fixed
    signal_generators: Vec<Box<dyn SignalsIter>>,
}

// struct SerializableSignalsIter<T: Next<f64>> {
//     indicator: T,
// }

// impl Serialize for Box<dyn SignalsIter> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         // Convert the signal generators into enums
//         // TODO: configuration won't be preserved - how can we do this?

//         // match self {

//         // }
//     }
// }

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

        let mut trades = BTreeMap::new();
        // Given each day and it's technical indicators, predict the return and
        // act accordingly
        for (day, price) in prices.iter() {
            // FIXME: for some reason, `signals` is empty sometimes and I have no
            // idea why. Debugging it, it appears that for some reason when the
            // for loop is making its last iteration, some action at a distance
            // just deletes whatever signal generator is in there, leaving it
            // empty. Then when it does iter/map over the signal generators,
            // there are none of them to do any mapping so we get an empty Vec.

            // Actually it happens because the deserialized one doesn't have a
            // signal generator. Problematic. TODO: how can we make Box<dyn SignalsIter>
            // survive serialization?
            let signals: Vec<f32> = self.next_signals(price);

            // TODO: start submitting PRs to improve rustlearn, it has no
            // error enums for one thing
            let prediction = match self.model.predict(&Array::from(signals)) {
                Ok(r) => r,
                Err(msg) => return Err(DecisionTreeError::TrainingError(msg.to_string())),
            };

            // println!("{} {:?}", day, prediction);
            // TODO: don't hardcode traded shares
            let position = match prediction.get(0, 0) {
                val if val == 1.0 => Position::Long(1),
                val if val == 0.0 => Position::Short(1),
                val => {
                    return Err(DecisionTreeError::TrainingError(format!(
                        "Invalid prediction '{}'",
                        val
                    )))
                }
            };

            println!("Day: {} {:?}", day, self.signal_generators);
            trades.insert(*day, position);
        }

        Ok(Trades { trades })
    }
}

#[cfg(test)]
mod tests {
    use super::DecisionTreeTrader;
    use crate::{
        backtester::Position,
        date::Date,
        marketdata::prices::Prices,
        signals::{relativestrengthindexsignals::RSISignalsIter, signals::SignalsIter},
        trading::tradingmodel::TradingModel,
    };
    use chrono::Duration;
    use rustlearn::trees::decision_tree::{DecisionTree, Hyperparameters};
    use std::collections::BTreeMap;

    /// Creates a month of Prices
    fn fixture_setup() -> Prices {
        let start = Date::from_ymd(2012, 1, 2);
        let end = Date::from_ymd(2012, 2, 2);
        let mut dt = start;
        let mut entries = BTreeMap::new();
        while dt <= end {
            entries.insert(dt, 30.0);
            // dt = dt.and_hms(1, 1, 1) + Duration::days(1);
            dt = dt + Duration::days(1);
        }

        Prices {
            map: entries,
            symbol: "jpm".to_string(),
        }
    }

    #[test]
    #[should_panic]
    fn smoke_test() {
        // Can we make it run and then serialize/deserialize?

        let indics: Vec<Box<dyn SignalsIter>> = vec![Box::new(RSISignalsIter::default())];

        // Construct the model
        let params = Hyperparameters::new(indics.len());
        let mut dt_trader = DecisionTreeTrader::new(params.build(), indics);

        // Train it
        let prices = fixture_setup();
        let range = Date::range(Date::from_ymd(2012, 01, 2), Date::from_ymd(2012, 01, 30));
        dt_trader.train(&prices, range, 3, 0.03).unwrap();

        // Can we turn it into bincode and back?
        let bytes = bincode::serialize(&dt_trader).unwrap();
        let loaded: DecisionTreeTrader = bincode::deserialize(&bytes).unwrap();

        // Predict something
        let trades = dt_trader.get_trades(&prices).unwrap();
        assert!(trades.trades.iter().all(|p| *p.1 == Position::Long(1)));

        // Predict it again from the deserialized copy
        // FIXME: panics because the Box<dyn SignalsIter> is lost during
        // serialization/deserialization.
        let again_trades = loaded.get_trades(&prices).unwrap();
        assert_eq!(trades, again_trades);
    }
}
