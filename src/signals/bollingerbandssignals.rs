//! Signals generated with Bollinger Bands.

use super::{Output, Signal, SignalsIter};
use crate::util::clamp;
use dg_ta::indicators::{BollingerBands, BollingerBandsOutput};
use dg_ta::{Next, Reset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Contains a ta-rs BollingerBands object, from which it generates signals.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BBSignalsIter {
    bb: BollingerBands,
}

impl BBSignalsIter {
    pub fn new(length: u32, multiplier: f64) -> Result<Self, dg_ta::errors::ErrorKind> {
        Ok(Self {
            bb: BollingerBands::new(length, multiplier)?,
        })
    }
}

impl Reset for BBSignalsIter {
    fn reset(&mut self) {
        self.bb.reset()
    }
}

#[typetag::serde]
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

#[cfg(test)]
mod tests {
    use super::{BBSignalsIter, BollingerBands};
    use crate::Date;
    use crate::{
        marketdata::prices::Prices,
        signals::{Signal, SignalsIter},
        util::{nearly_equal, TimeSeries},
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
    fn test_signals_from_bollinger_bands() {
        let _bb = BollingerBands::new(5, 2.0).unwrap();

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
        let l = map.len();
        let prices = Prices {
            map,
            symbol: "jpm".to_string(),
        };

        let mut sig_gen = BBSignalsIter::new(5, 2.0).unwrap();
        let signals: Vec<Signal> = prices.iter().map(|p| sig_gen.next(*p.1).0).collect();

        assert_eq!(signals.len(), l);
        assert!(nearly_equal(signals[0].into(), 0.0));
        assert!(nearly_equal(signals[1].into(), -0.5));
        assert!(nearly_equal(signals[5].into(), 0.9669875568304561));
    }
}
