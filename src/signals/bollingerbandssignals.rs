use super::signals::Output;
use crate::{marketdata::prices::Prices, signals::signals::Signals};
use serde::Serialize;
use std::collections::HashMap;
use ta::indicators::{BollingerBands, BollingerBandsOutput};
use ta::Next;

#[derive(Debug, Serialize)]
pub struct BollingerBandsSignals {
    pub outputs: Vec<Output>,
    pub signals: Vec<f64>,
}

impl BollingerBandsSignals {
    pub fn new(prices: &Prices, mut bb: BollingerBands) -> Self {
        // Generate signals as %BB
        // [(Price – Lower Band) / (Upper Band – Lower Band)] * 100
        // https://www.fidelity.com/learning-center/trading-investing/technical-analysis/technical-indicator-guide/percent-b
        let mut signals = Vec::<f64>::new();
        let mut outputs = Vec::new();
        for (_, price) in prices.iter() {
            let o = bb.next(*price);

            // how far along from the average to the bounds is the price?
            let signal = -(2.0 * (((price - o.lower) / (o.upper - o.lower)) - 0.5));

            signals.push(signal);
            outputs.push(o.into());
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
    fn signals(&self) -> &Vec<f64> {
        &self.signals
    }

    fn outputs(&self) -> &Vec<Output> {
        &self.outputs
    }
}

#[cfg(test)]
mod tests {
    use super::{BollingerBands, BollingerBandsSignals, Signals};
    use crate::{marketdata::prices::Prices, util::nearly_equal};
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
    fn test_signals_from_bollinger_bands() {
        let _bb = BollingerBands::new(5, 2.0).unwrap();

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
        let l = map.len();
        let prices = Prices {
            map,
            symbol: "jpm".to_string(),
        };
        let signals = BollingerBandsSignals::new(&prices, BollingerBands::new(5, 2.0).unwrap());

        assert_eq!(signals.signals().len(), l);
        assert!(nearly_equal(signals.signals()[1], -0.5));
        assert!(nearly_equal(signals.signals()[5], 0.9669875568304561));
    }
}
