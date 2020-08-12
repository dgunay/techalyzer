use super::signals::{Output, Signal, SignalsIter};
use crate::signals::signals::Signals;
use crate::{marketdata::prices::Prices, util::clamp};
use serde::Serialize;
use std::{collections::HashMap, slice::Iter};
use ta::indicators::MovingAverageConvergenceDivergence;
use ta::indicators::MovingAverageConvergenceDivergenceOutput;
use ta::{Next, Reset};

#[derive(Serialize)]
pub struct MovingAverageConvergenceDivergenceSignals {
    pub outputs: Vec<Output>,
    signals: Vec<Signal>,
}

#[derive(Default)]
pub struct MACDSignalsIter {
    macd_line_prev: f64,
    macd: MovingAverageConvergenceDivergence,
}

impl Reset for MACDSignalsIter {
    fn reset(&mut self) {
        self.macd.reset()
    }
}

impl SignalsIter for MACDSignalsIter {
    fn next(&mut self, price: f64) -> (Signal, Output) {
        // let output: MovingAverageConvergenceDivergenceOutput = indicator_output.into();
        let o = self.macd.next(price);

        // FIXME: I can't think of a great way to do a normalized -1.0 to 1.0
        // scale on the MACD, so for now I'll go with having above/below be
        // 0.5/-0.5, and then just add the slope of the MACD.
        let above_or_below = if o.macd > o.signal {
            0.5
        } else if o.macd < o.signal {
            -0.5
        } else {
            0.0
        };

        // slope is normalized and clamped to 0.5/-0.5 (with 0.5 being a 45
        // degree angle trending upwards)
        // FIXME: ensure no div by zero
        let slope = o.macd - self.macd_line_prev;
        let norm_macd_slope = if self.macd_line_prev == 0.0 {
            0.0
        } else {
            (slope / self.macd_line_prev) / 2.0
        };

        self.macd_line_prev = o.macd;
        let signal = Signal::new(clamp(norm_macd_slope, -0.5, 0.5).unwrap() + above_or_below);
        (signal, o.into())
    }
}

impl MovingAverageConvergenceDivergenceSignals {
    pub fn new(prices: &Prices, macd: MovingAverageConvergenceDivergence) -> Self {
        // TODO: should I check prices not empty?

        // Generate signals from MACD
        let mut signals = Vec::new();

        let mut outputs = Vec::new();
        let mut signal_iterator = MACDSignalsIter {
            macd,
            ..Default::default()
        };
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

// impl From<Output> for MovingAverageConvergenceDivergenceOutput {
//     fn from(o: Output) -> Self {
//         Self {
//             macd: o.output["macd"],
//             signal: o.output["signal"],
//             histogram: o.output["histogram"],
//         }
//     }
// }

impl Signals for MovingAverageConvergenceDivergenceSignals {
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
    use super::MovingAverageConvergenceDivergenceSignals;
    use crate::marketdata::prices::Prices;
    use crate::Date;
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
        let map: BTreeMap<Date, f64> = vec![
            (Date::from_ymd(2020, 03, 1), 1.9),
            (Date::from_ymd(2020, 03, 2), 2.0),
            (Date::from_ymd(2020, 03, 3), 2.1),
            (Date::from_ymd(2020, 03, 4), 2.2),
            (Date::from_ymd(2020, 03, 5), 2.1),
            (Date::from_ymd(2020, 03, 6), 1.5),
            (Date::from_ymd(2020, 03, 7), 1.3),
            (Date::from_ymd(2020, 03, 8), 1.2),
            (Date::from_ymd(2020, 03, 9), 1.1),
            (Date::from_ymd(2020, 03, 10), 1.0),
        ]
        .iter()
        .cloned()
        .collect();
        let prices = Prices {
            map,
            symbol: "jpm".to_string(),
        };

        let _signals = MovingAverageConvergenceDivergenceSignals::new(
            &prices,
            MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap(),
        );

        // TODO: test more specific values/calculations after plotting is
        // implemented
        // TODO: maybe no longer necessary with the implementation of Signal struct.
        // let nan = 0. / 0.;
        // assert!(signals.signals().iter().cloned().fold(nan, f64::max) <= 1.0);
        // assert!(signals.signals().iter().cloned().fold(nan, f64::min) >= -1.0);
    }
}
