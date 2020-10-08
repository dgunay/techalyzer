//! Signals generated with Relative Strength Index (RSI).

use super::{Output, Signal, SignalsIter};
use serde::{Deserialize, Serialize};
use ta::indicators::RelativeStrengthIndex;
use ta::{Next, Reset};

/// Generates buy and sell signals from RSI.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RSISignalsIter {
    rsi: RelativeStrengthIndex,
}

impl Reset for RSISignalsIter {
    fn reset(&mut self) {
        self.rsi.reset()
    }
}

impl RSISignalsIter {
    /// Constructs an RSISignalsIter. `ema_window` is the window used for the
    /// exponential moving averages used to calculate RSI.
    pub fn new(ema_window: u32) -> Result<Self, ta::errors::ErrorKind> {
        Ok(Self {
            rsi: RelativeStrengthIndex::new(ema_window)?,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{nearly_equal, TimeSeries};
    use crate::{marketdata::Prices, Date};

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
                                  // let signals =
                                  //     RelativeStrengthIndexSignals::new(&prices, RelativeStrengthIndex::new(14).unwrap());
        let mut sig_gen = RSISignalsIter::default();
        let signals: Vec<Signal> = prices.iter().map(|p| sig_gen.next(*p.1).0).collect();

        assert_eq!(signals.len(), l);
        assert!(nearly_equal(signals[0].into(), 0.0));
        assert!(nearly_equal(signals[1].into(), -0.0714285714285714));
        assert!(nearly_equal(signals[2].into(), -0.14213197969543168));
        assert!(nearly_equal(signals[3].into(), -0.21141421392677695));
        assert!(nearly_equal(signals[4].into(), -0.1081504306316774));
        assert!(nearly_equal(signals[5].into(), 0.3031110904761263));
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
}
