//! Signals generated with Relative Strength Index (RSI).

use super::{Output, Signal, SignalsIter};
use crate::{marketdata::prices::Prices, signals::Signals};
use serde::{Deserialize, Serialize};

use dg_ta::indicators::RelativeStrengthIndex;
use dg_ta::{Next, Reset};
use std::slice::Iter;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RSISignalsIter {
    rsi: RelativeStrengthIndex,
}

impl Reset for RSISignalsIter {
    fn reset(&mut self) {
        self.rsi.reset()
    }
}

#[typetag::serde]
impl SignalsIter for RSISignalsIter {
    fn next(&mut self, price: f64) -> (Signal, Output) {
        let rsi_val = self.rsi.next(price);

        // Instead of 0 to 100, signal is -1.0 to 1.0
        (Signal::new(-((rsi_val / 50.0) - 1.0)), rsi_val.into())
    }
}

#[derive(Serialize)]
pub struct RelativeStrengthIndexSignals {
    outputs: Vec<Output>,
    signals: Vec<Signal>,
}

impl From<f64> for Output {
    fn from(f: f64) -> Self {
        Output {
            output: [("rsi".to_string(), f)].iter().cloned().collect(),
        }
    }
}

impl RelativeStrengthIndexSignals {
    pub fn new(prices: &Prices, rsi: RelativeStrengthIndex) -> Self {
        // Generate signals from RSI
        let mut signals = Vec::new();
        let mut outputs = Vec::new();
        let mut signal_iterator = RSISignalsIter { rsi };
        for (_, price) in prices.iter() {
            let (signal, rsi_val) = signal_iterator.next(*price);

            signals.push(signal);
            outputs.push(rsi_val);
        }

        Self { outputs, signals }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Signals for RelativeStrengthIndexSignals {
    fn signals(&self) -> &Vec<Signal> {
        &self.signals
    }

    fn outputs(&self) -> &Vec<Output> {
        &self.outputs
    }

    fn iter(&self) -> Iter<Output> {
        self.outputs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{nearly_equal, TimeSeries};
    use crate::Date;

    struct Close {
        price: f64,
    }

    impl dg_ta::Close for Close {
        fn close(&self) -> f64 {
            self.price
        }
    }

    #[test]
    fn test_signals_from_rsi() {
        let prices = fixture_prices();
        let l = prices.map.len(); // pacify the borrow checker
        let signals =
            RelativeStrengthIndexSignals::new(&prices, RelativeStrengthIndex::new(14).unwrap());

        assert_eq!(signals.signals().len(), l);
        assert!(nearly_equal(signals.signals()[0].val, 0.0));
        assert!(nearly_equal(signals.signals()[1].val, -0.0714285714285714));
        assert!(nearly_equal(signals.signals()[2].val, -0.14213197969543168));
        assert!(nearly_equal(signals.signals()[3].val, -0.21141421392677695));
        assert!(nearly_equal(signals.signals()[4].val, -0.1081504306316774));
        assert!(nearly_equal(signals.signals()[5].val, 0.3031110904761263));
    }

    fn fixture_prices() -> Prices {
        let map: TimeSeries<f64> = vec![
            (Date::from_ymd(2020, 03, 1), 1.9),
            (Date::from_ymd(2020, 03, 2), 2.0),
            (Date::from_ymd(2020, 03, 3), 2.1),
            (Date::from_ymd(2020, 03, 4), 2.2),
            (Date::from_ymd(2020, 03, 5), 2.1),
            (Date::from_ymd(2020, 03, 6), 1.5),
        ]
        .iter()
        .cloned()
        .collect();
        Prices {
            map,
            symbol: "jpm".to_string(),
        }
    }

    /// This test is mostly just to see if to_json worked
    #[test]
    fn test_json() {
        let prices = fixture_prices();
        let signals =
            RelativeStrengthIndexSignals::new(&prices, RelativeStrengthIndex::new(14).unwrap());

        let _ = signals.to_json();
        // print!("{}", s);
    }
}
