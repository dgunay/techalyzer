use super::signals::Output;
use crate::{marketdata::prices::Prices, signals::signals::Signals};
use serde::Serialize;

use ta::indicators::RelativeStrengthIndex;
use ta::Next;

#[derive(Serialize)]
pub struct RelativeStrengthIndexSignals {
    outputs: Vec<Output>,
    signals: Vec<f64>,
}

impl From<f64> for Output {
    fn from(f: f64) -> Self {
        Output {
            output: [("rsi".to_string(), f)].iter().cloned().collect(),
        }
    }
}

impl RelativeStrengthIndexSignals {
    // TODO: make this use reference to Prices, not vec of f64s
    pub fn new(prices: &Prices, mut rsi: RelativeStrengthIndex) -> Self {
        // pub fn new(prices: &Vec<f64>, mut rsi: RelativeStrengthIndex) -> Self {
        // Generate signals from RSI
        let mut signals = Vec::<f64>::new();
        let mut outputs = Vec::new();
        for (_, price) in prices.iter() {
            let rsi_val = rsi.next(*price);

            // TODO: we can create a "Signals" object that simply takes the math
            // to calculate signal
            // and abstracts out all this looping over prices and stuff

            // Instead of 0 to 100, signal is -1.0 to 1.0
            let signal = -((rsi_val / 50.0) - 1.0);

            signals.push(signal.into());
            outputs.push(rsi_val.into());
        }

        Self { outputs, signals }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Signals for RelativeStrengthIndexSignals {
    fn signals(&self) -> &Vec<f64> {
        &self.signals
    }

    fn outputs(&self) -> &Vec<Output> {
        &self.outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::nearly_equal;
    use chrono::NaiveDate;
    use std::collections::BTreeMap;

    struct Close {
        price: f64,
    }

    impl ta::Close for Close {
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
        assert!(nearly_equal(signals.signals()[0], 0.0));
        assert!(nearly_equal(signals.signals()[1], -0.0714285714285714));
        assert!(nearly_equal(signals.signals()[2], -0.14213197969543168));
        assert!(nearly_equal(signals.signals()[3], -0.21141421392677695));
        assert!(nearly_equal(signals.signals()[4], -0.1081504306316774));
        assert!(nearly_equal(signals.signals()[5], 0.3031110904761263));
    }

    fn fixture_prices() -> Prices {
        let map: BTreeMap<NaiveDate, f64> = vec![
            (NaiveDate::from_ymd(2020, 03, 1), 1.9),
            (NaiveDate::from_ymd(2020, 03, 2), 2.0),
            (NaiveDate::from_ymd(2020, 03, 3), 2.1),
            (NaiveDate::from_ymd(2020, 03, 4), 2.2),
            (NaiveDate::from_ymd(2020, 03, 5), 2.1),
            (NaiveDate::from_ymd(2020, 03, 6), 1.5),
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
