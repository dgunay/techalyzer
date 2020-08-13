use super::signals::{Output, Signal, SignalsIter};
use crate::{marketdata::prices::Prices, signals::signals::Signals, util::clamp};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, slice::Iter};
use ta::indicators::{BollingerBands, BollingerBandsOutput};
use ta::{Next, Reset};

#[derive(Default, Debug)]
pub struct BBSignalsIter {
    bb: BollingerBands,
}

impl Reset for BBSignalsIter {
    fn reset(&mut self) {
        self.bb.reset()
    }
}

// #[typetag::serde]
impl SignalsIter for BBSignalsIter {
    fn next(&mut self, price: f64) -> (Signal, Output) {
        let o = self.bb.next(price);

        // how far along from the average to the bounds is the price?
        // floor the range to 0
        let calculation = -(2.0 * ((price - o.lower) / (o.upper - o.lower) - 0.5));
        match calculation {
            c if c.is_nan() => {
                // warn!(format!("Computing signal from price {} was NaN", price));
                (Signal::new(0.0), o.into())
            }
            _ => (
                Signal::new(clamp(calculation, -1.0, 1.0).unwrap()),
                o.into(),
            ),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BollingerBandsSignals {
    pub outputs: Vec<Output>,
    pub signals: Vec<Signal>,
}

impl BollingerBandsSignals {
    pub fn new(prices: &Prices, bb: BollingerBands) -> Self {
        // Generate signals as %BB
        // [(Price – Lower Band) / (Upper Band – Lower Band)] * 100
        // https://www.fidelity.com/learning-center/trading-investing/technical-analysis/technical-indicator-guide/percent-b
        let mut signals = Vec::new();
        let mut outputs = Vec::new();
        let mut signal_iterator = BBSignalsIter { bb };
        for (_, price) in prices.iter() {
            let (signal, output) = signal_iterator.next(*price);

            signals.push(signal);
            outputs.push(output);
        }

        Self { outputs, signals }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl From<BollingerBandsOutput> for Output {
    fn from(b: BollingerBandsOutput) -> Self {
        let map: HashMap<String, f64> = [
            ("average".to_string(), b.average),
            ("upper".to_string(), b.upper),
            ("lower".to_string(), b.lower),
        ]
        .iter()
        .cloned()
        .collect();

        Output { output: map }
    }
}

impl Signals for BollingerBandsSignals {
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
    use super::{BollingerBands, BollingerBandsSignals, Signals};
    use crate::Date;
    use crate::{marketdata::prices::Prices, util::nearly_equal};
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
    fn test_signals_from_bollinger_bands() {
        let _bb = BollingerBands::new(5, 2.0).unwrap();

        let map: BTreeMap<Date, f64> = vec![
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
        let l = map.len();
        let prices = Prices {
            map,
            symbol: "jpm".to_string(),
        };
        let signals = BollingerBandsSignals::new(&prices, BollingerBands::new(5, 2.0).unwrap());

        assert_eq!(signals.signals().len(), l);
        assert!(nearly_equal(signals.signals()[0].val, 0.0));
        assert!(nearly_equal(signals.signals()[1].val, -0.5));
        assert!(nearly_equal(signals.signals()[5].val, 0.9669875568304561));
    }
}
