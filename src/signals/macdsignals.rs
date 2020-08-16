//! Signals generated with Moving Average Convergence Divergence (MACD).

use super::{Output, Signal, SignalsIter};
use crate::util::clamp;
use dg_ta::indicators::MovingAverageConvergenceDivergence;
use dg_ta::indicators::MovingAverageConvergenceDivergenceOutput;
use dg_ta::{Next, Reset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MACDSignalsIter {
    macd_line_prev: f64,
    macd: MovingAverageConvergenceDivergence,
}

impl MACDSignalsIter {
    pub fn new(
        fast_length: u32,
        slow_length: u32,
        signal_length: u32,
    ) -> Result<Self, dg_ta::errors::ErrorKind> {
        Ok(Self {
            macd_line_prev: 0.0,
            macd: MovingAverageConvergenceDivergence::new(fast_length, slow_length, signal_length)?,
        })
    }
}

impl Reset for MACDSignalsIter {
    fn reset(&mut self) {
        self.macd.reset()
    }
}

#[typetag::serde]
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

#[cfg(test)]
mod tests {
    use super::MACDSignalsIter;
    use crate::marketdata::prices::Prices;
    use crate::{
        signals::{Signal, SignalsIter},
        util::TimeSeries,
        Date,
    };

    struct Close {
        price: f64,
    }

    impl dg_ta::Close for Close {
        fn close(&self) -> f64 {
            self.price
        }
    }

    #[test]
    fn test_signals_from_macd() {
        let map: TimeSeries<f64> = vec![
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

        let mut sig_gen = MACDSignalsIter::default();
        let signals: Vec<Signal> = prices.iter().map(|p| sig_gen.next(*p.1).0).collect();

        // TODO: test more specific values/calculations after plotting is
        // implemented
        // TODO: maybe no longer necessary with the implementation of Signal struct.
        let nan = 0. / 0.;
        let as_float: Vec<f64> = signals.iter().map(f64::from).collect();
        assert!(as_float.iter().cloned().fold(nan, f64::max) <= 1.0);
        assert!(as_float.iter().cloned().fold(nan, f64::min) >= -1.0);
    }
}
