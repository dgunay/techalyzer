use super::signals::Output;
use crate::signals::signals::Signals;
use crate::{marketdata::prices::Prices, util::clamp};
use serde::Serialize;
use std::collections::HashMap;
use ta::indicators::MovingAverageConvergenceDivergence;
use ta::indicators::MovingAverageConvergenceDivergenceOutput;
use ta::Next;

#[derive(Serialize)]
pub struct MovingAverageConvergenceDivergenceSignals {
    pub outputs: Vec<Output>,
    signals: Vec<f64>,
}

impl MovingAverageConvergenceDivergenceSignals {
    pub fn new(prices: &Prices, mut macd: MovingAverageConvergenceDivergence) -> Self {
        // TODO: should I check prices not empty?

        // Generate signals from MACD
        let mut signals = Vec::<f64>::new();

        let mut outputs = Vec::new();
        let mut macd_line_prev = 0.0;
        for (_, price) in prices.iter() {
            // FIXME: for some reason I have to clone the price or next() won't
            // work - maybe an upstream PR is necessary
            let output = macd.next(*price);

            // FIXME: I can't think of a great way to do a normalized -1.0 to 1.0
            // scale on the MACD, so for now I'll go with having above/below be
            // 0.5/-0.5, and then just add the slope of the MACD.
            let above_or_below = if output.macd > output.signal {
                0.5
            } else if output.macd < output.signal {
                -0.5
            } else {
                0.0
            };

            // slope is normalized and clamped to 0.5/-0.5 (with 0.5 being a 45
            // degree angle trending upwards)
            // FIXME: ensure no div by zero
            let slope = output.macd - macd_line_prev;
            let norm_macd_slope = (slope / macd_line_prev) / 2.0;
            let signal = clamp(norm_macd_slope, -0.5, 0.5).unwrap() + above_or_below;

            macd_line_prev = output.macd;
            signals.push(signal);
            outputs.push(output.into());
        }

        Self { outputs, signals }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl From<MovingAverageConvergenceDivergenceOutput> for Output {
    fn from(m: MovingAverageConvergenceDivergenceOutput) -> Self {
        let map: HashMap<String, f64> = [
            ("macd".to_string(), m.macd),
            ("signal".to_string(), m.signal),
            ("histogram".to_string(), m.histogram),
        ]
        .iter()
        .cloned()
        .collect();

        Output { output: map }
    }
}

impl Signals for MovingAverageConvergenceDivergenceSignals {
    fn signals(&self) -> &Vec<f64> {
        &self.signals
    }

    fn outputs(&self) -> &Vec<Output> {
        &self.outputs
    }
}

#[cfg(test)]
mod tests {
    use super::MovingAverageConvergenceDivergenceSignals;
    use super::Signals;
    use crate::marketdata::prices::Prices;
    use chrono::NaiveDate;
    use std::collections::BTreeMap;
    use ta::indicators::MovingAverageConvergenceDivergence;

    struct Close {
        price: f64,
    }

    impl ta::Close for Close {
        fn close(&self) -> f64 {
            self.price
        }
    }

    #[test]
    fn test_signals_from_macd() {
        let map: BTreeMap<NaiveDate, f64> = vec![
            (NaiveDate::from_ymd(2020, 03, 1), 1.9),
            (NaiveDate::from_ymd(2020, 03, 2), 2.0),
            (NaiveDate::from_ymd(2020, 03, 3), 2.1),
            (NaiveDate::from_ymd(2020, 03, 4), 2.2),
            (NaiveDate::from_ymd(2020, 03, 5), 2.1),
            (NaiveDate::from_ymd(2020, 03, 6), 1.5),
            (NaiveDate::from_ymd(2020, 03, 7), 1.3),
            (NaiveDate::from_ymd(2020, 03, 8), 1.2),
            (NaiveDate::from_ymd(2020, 03, 9), 1.1),
            (NaiveDate::from_ymd(2020, 03, 10), 1.0),
        ]
        .iter()
        .cloned()
        .collect();
        let prices = Prices {
            map,
            symbol: "jpm".to_string(),
        };

        let signals = MovingAverageConvergenceDivergenceSignals::new(
            &prices,
            MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap(),
        );

        // TODO: test more specific values/calculations after plotting is
        // implemented
        let nan = 0. / 0.;
        assert!(signals.signals().iter().cloned().fold(nan, f64::max) <= 1.0);
        assert!(signals.signals().iter().cloned().fold(nan, f64::min) >= -1.0);
    }
}
