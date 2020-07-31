use super::signals::Outputs;
use crate::signals::signals::Signals;
use serde::Serialize;
use ta::indicators::RelativeStrengthIndex;
use ta::Next;

#[derive(Serialize)]
pub struct RelativeStrengthIndexSignals {
    outputs: Vec<f64>,
    signals: Vec<f64>,
}

impl RelativeStrengthIndexSignals {
    pub fn new(prices: Vec<&f64>, mut rsi: RelativeStrengthIndex) -> Self {
        // Generate signals from RSI
        let mut signals = Vec::<f64>::new();
        let mut outputs = Vec::new();
        for price in prices.iter() {
            let rsi_val = rsi.next(**price);

            // TODO: we can create a "Signals" object that simply takes the math
            // to calculate signal
            // and abstracts out all this looping over prices and stuff

            // Instead of 0 to 100, signal is -1.0 to 1.0
            let signal = (rsi_val / 50.0) - 1.0;

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
    fn signals(&self) -> &Vec<f64> {
        &self.signals
    }

    fn outputs(&self) -> Outputs {
        let outputs = self.outputs.iter().map(|o| vec![o.clone()]).collect();
        Outputs::new(outputs, vec!["rsi".to_string()]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::nearly_equal;

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
        let prices = vec![&1.9, &2.0, &2.1, &2.2, &2.1, &1.5];
        let l = prices.len(); // pacify the borrow checker
        let signals =
            RelativeStrengthIndexSignals::new(prices, RelativeStrengthIndex::new(14).unwrap());

        // println!("{:?}", signals.signals());
        // println!("{:?}", signals.outputs);
        // println!("{:?}", prices);

        assert_eq!(signals.signals().len(), l);
        assert!(nearly_equal(signals.signals()[0], 0.0));
        assert!(nearly_equal(signals.signals()[1], 0.0714285714285714));
        assert!(nearly_equal(signals.signals()[2], 0.14213197969543168));
        assert!(nearly_equal(signals.signals()[3], 0.21141421392677695));
        assert!(nearly_equal(signals.signals()[4], 0.1081504306316774));
        assert!(nearly_equal(signals.signals()[5], -0.3031110904761263));
    }

    /// This test is mostly just to see if to_json worked
    #[test]
    fn test_json() {
        let prices = vec![&1.9, &2.0, &2.1, &2.2, &2.1, &1.5];

        let signals =
            RelativeStrengthIndexSignals::new(prices, RelativeStrengthIndex::new(14).unwrap());

        let _ = signals.to_json();
        // print!("{}", s);
    }
}
