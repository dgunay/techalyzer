use super::tradingmodel::{Trades, TradingModel};
use crate::Date;
use crate::{backtester::Position, marketdata::prices::Prices, signals::signals::SignalsIter};
use derive_more::Display;
use rustlearn::prelude::*;
use rustlearn::{
    multiclass::OneVsRestWrapper,
    trees::decision_tree::{DecisionTree, Hyperparameters},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// TODO: Try doing this to make forgetting to train the model a compile-time error:
// https://stackoverflow.com/questions/42036826/using-the-rust-compiler-to-prevent-forgetting-to-call-a-method

// TODO: document
#[derive(Serialize, Deserialize)]
pub struct DecisionTreeTrader {
    /// Our multi-class decision tree classifier
    learner: OneVsRestWrapper<DecisionTree>,

    /// Signals that will inform the model.
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
// const SHORT: f32 = 0.0;
const OUT: f32 = 0.0;
const SHORT: f32 = -1.0;

impl DecisionTreeTrader {
    pub fn new(signal_generators: Vec<Box<dyn SignalsIter>>) -> Self {
        let learner = Hyperparameters::new(signal_generators.len()).one_vs_rest();
        Self {
            learner,
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
        threshold: f32,
    ) -> Result<(), DecisionTreeError> {
        let mut x = Vec::new();
        let mut y = Vec::new();

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
            let label = match future_return {
                r if r >= threshold => LONG,
                r if r <= -threshold => SHORT,
                _ => OUT,
            };

            // X is the signals, Y our long/short/out decision based on future return
            x.push(signals);
            y.push(label);
        }

        // Construct X train, Y train data out of the prices
        self.learner
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
            // signal generator. Problematic. Should we pre-emptively error out
            // if all the signals are a contant value (0/1/-1)

            let signals: Vec<f32> = self.next_signals(price);

            // TODO: start submitting PRs to improve rustlearn, it has no
            // error enums for one thing
            let prediction = match self.learner.predict(&Array::from(signals)) {
                Ok(r) => r,
                Err(msg) => return Err(DecisionTreeError::TrainingError(msg.to_string())),
            };

            // println!("{} {:?}", day, prediction);
            // TODO: don't hardcode traded shares
            let position = match prediction.get(0, 0) {
                val if val == LONG => Position::Long(1),
                val if val == OUT => Position::Out,
                val if val == SHORT => Position::Short(1),
                val => {
                    return Err(DecisionTreeError::TrainingError(format!(
                        "Invalid prediction '{}'",
                        val
                    )))
                }
            };

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
        signals::{
            macdsignals::MACDSignalsIter, relativestrengthindexsignals::RSISignalsIter,
            signals::SignalsIter,
        },
        trading::tradingmodel::{Trades, TradingModel},
    };
    use chrono::Duration;
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
    fn smoke_test() {
        // Can we make it run and then serialize/deserialize?

        let indics: Vec<Box<dyn SignalsIter>> = vec![Box::new(RSISignalsIter::default())];

        // Construct the model
        let mut dt_trader = DecisionTreeTrader::new(indics);

        // Train it
        let prices = fixture_setup();
        let range = Date::range(Date::from_ymd(2012, 01, 2), Date::from_ymd(2012, 01, 30));
        dt_trader.train(&prices, range, 3, 0.03).unwrap();

        // Can we turn it into bincode and back?
        let bytes = bincode::serialize(&dt_trader).unwrap();
        let loaded: DecisionTreeTrader = bincode::deserialize(&bytes).unwrap();

        // Predict what trades to make for profit
        let trades = dt_trader.get_trades(&prices).unwrap();

        // Predict it again from the deserialized copy
        let again_trades = loaded.get_trades(&prices).unwrap();
        assert_eq!(trades, again_trades);
    }

    #[test]
    fn bull_market() {
        let new_prices: Vec<f64> = (15..55).map(|f| f.into()).collect();
        let trades = run_trader_test(new_prices, 3, 0.03);
        assert!(trades.trades.iter().all(|p| *p.1 == Position::Long(1)));
    }

    // edits the prices used to train the model before running a test over the
    // fixture_data()
    fn run_trader_test(new_prices: Vec<f64>, horizon: u32, threshold: f32) -> Trades {
        // TODO: parameterize the indicators
        let indics: Vec<Box<dyn SignalsIter>> = vec![Box::new(MACDSignalsIter::default())];

        // Construct the model
        let mut dt_trader = DecisionTreeTrader::new(indics);

        // Train it (in a bull market where stonks only go up)
        let mut prices = fixture_setup();
        for (i, (_, price)) in prices.iter_mut().enumerate() {
            *price = new_prices[i];
        }

        let range = Date::range(Date::from_ymd(2012, 01, 2), Date::from_ymd(2012, 01, 30));
        dt_trader.train(&prices, range, horizon, threshold).unwrap();

        dt_trader.get_trades(&prices).unwrap()
    }

    #[test]
    fn bear_market() {
        let new_prices: Vec<f64> = (15..55).map(|f| f.into()).rev().collect();
        let trades = run_trader_test(new_prices, 3, 0.03);
        assert!(trades.trades.iter().all(|p| *p.1 == Position::Short(1)));
    }

    #[test]
    fn afraid_to_invest() {
        let new_prices: Vec<f64> = (15..55).map(|f| f.into()).rev().collect();
        let trades = run_trader_test(new_prices, 3, 1.0);
        assert!(trades.trades.iter().all(|p| *p.1 == Position::Out));
    }

    #[test]
    fn up_and_down() {}
}
