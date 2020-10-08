//! Golden cross and death cross signals with simple moving averages. Intermediate signals are calculated
//! from an unsophisticated heuristic of slope with some arbitrary arithmetic
//! applied.

use super::{Output, Signal, SignalsIter};
use crate::util::{clamp, slope};
use serde::{Deserialize, Serialize};
use ta::{errors::ErrorKind, indicators::SimpleMovingAverage, Next, Reset};

/// Generates buy and sell signals golden/death crosses and long-term SMA trends.
#[derive(Debug, Serialize, Deserialize)]
pub struct SmaCrossoversSignalsIter {
    fast: SimpleMovingAverage,
    slow: SimpleMovingAverage,
    last_fast: f64,
    last_slow: f64,
}

impl Default for SmaCrossoversSignalsIter {
    fn default() -> Self {
        Self::new(50, 200).unwrap()
    }
}

impl Reset for SmaCrossoversSignalsIter {
    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
    }
}

impl SmaCrossoversSignalsIter {
    /// Constructs an SmaCrossoversSignalsIter. Panics if the windows are equal.
    // TODO: use a different, native error type
    pub fn new(fast_window: u32, slow_window: u32) -> Result<Self, ta::errors::ErrorKind> {
        if fast_window == slow_window {
            return Err(ErrorKind::InvalidParameter);
        }

        Ok(Self {
            fast: SimpleMovingAverage::new(fast_window)?,
            slow: SimpleMovingAverage::new(slow_window)?,
            last_fast: 0.0,
            last_slow: 0.0,
        })
    }
}

fn signal_output_pair(signal: f64, fast: f64, slow: f64) -> (Signal, Output) {
    (
        Signal::new(signal),
        // TODO: figure out a way to do this without an unwrap
        Output::new(
            vec![fast, slow],
            vec!["fast".to_string(), "slow".to_string()],
        )
        .unwrap(),
    )
}

#[typetag::serde]
impl SignalsIter for SmaCrossoversSignalsIter {
    fn next(&mut self, price: f64) -> (Signal, Output) {
        // TODO: document this algorithm
        let fast = self.fast.next(price);
        let slow = self.slow.next(price);

        // a new crossover is an instant +/- 1.0.
        if self.last_fast <= self.last_slow && fast > slow {
            self.last_fast = fast;
            self.last_slow = slow;
            return signal_output_pair(1.0, fast, slow);
        } else if self.last_fast >= self.last_slow && fast < slow {
            self.last_fast = fast;
            self.last_slow = slow;
            return signal_output_pair(-1.0, fast, slow);
        }

        // When fast is above slow, scale it from 0 to 1 depending on slope with
        // 0.5 being a flat line. Negate this logic for below.

        // multiplying by an arbitrary constant to provide more of a signal
        // TODO: instead of 5.0, maybe some kind of stock-specific factor can
        // be calculated to give us more regular results.
        let slope = slope(fast, self.last_fast, 1.0) * 5.0;
        let signal = if fast > slow {
            clamp(slope + 0.2, 0.0, 1.0).unwrap()
        } else if fast < slow {
            clamp(slope - 0.2, -1.0, 0.0).unwrap()
        } else {
            0.0
        };

        self.last_fast = fast;
        self.last_slow = slow;

        signal_output_pair(signal, fast, slow)
    }
}

#[cfg(test)]
mod tests {
    use super::SmaCrossoversSignalsIter;
    use crate::{
        datasource::{techalyzerjson::TechalyzerJson, DataSource},
        date::Date,
        marketdata::Prices,
        signals::{Output, Signal, SignalsIter},
        util::TimeSeries,
    };
    use std::path::PathBuf;

    #[test]
    fn test_signals() {
        let prices = fixture_prices();
        let mut signals = SmaCrossoversSignalsIter::new(2, 8).unwrap();
        let sigs: Vec<(Signal, Output)> = prices.iter().map(|f| signals.next(*f.1)).collect();
        assert_eq!(sigs[2].0, Signal::new(1.0));
        assert_eq!(sigs[10].0, Signal::new(-1.0));
    }

    fn fixture_prices() -> Prices {
        let map: TimeSeries<f64> = vec![
            // trending up
            (Date::from_ymd(2020, 03, 1), 1.9),
            (Date::from_ymd(2020, 03, 2), 1.9),
            (Date::from_ymd(2020, 03, 3), 1.95),
            (Date::from_ymd(2020, 03, 4), 2.0),
            (Date::from_ymd(2020, 03, 5), 2.2),
            (Date::from_ymd(2020, 03, 6), 2.3),
            (Date::from_ymd(2020, 03, 7), 2.4),
            (Date::from_ymd(2020, 03, 8), 2.5),
            // trending down now
            (Date::from_ymd(2020, 03, 9), 2.4),
            (Date::from_ymd(2020, 03, 10), 2.3),
            (Date::from_ymd(2020, 03, 11), 2.2),
            (Date::from_ymd(2020, 03, 12), 2.0),
            (Date::from_ymd(2020, 03, 13), 1.8),
        ]
        .iter()
        .cloned()
        .collect();
        Prices {
            map,
            symbol: "jpm".to_string(),
        }
    }

    #[test]
    fn test_aphria_crossovers() {
        let prices = TechalyzerJson::new(&PathBuf::from("test/json/apha_smacrossover.json"))
            .unwrap()
            .get("apha")
            .unwrap();
        let mut sma = SmaCrossoversSignalsIter::default();
        let signals: Vec<Signal> = prices.iter().map(|e| sma.next(*e.1).0).collect();
        let signals: Vec<(Date, Signal)> = prices
            .iter()
            .enumerate()
            .map(|(i, e)| (*e.0, signals[i]))
            .collect();

        let crossovers: Vec<&(Date, Signal)> = signals
            .iter()
            .filter(|e| e.1 == Signal::new(1.0) || e.1 == Signal::new(-1.0))
            .collect();

        let expected = vec![
            (
                Date::parse_from_str("2015-05-29", "%Y-%m-%d").unwrap(),
                Signal(1.0),
            ),
            (
                Date::parse_from_str("2015-07-10", "%Y-%m-%d").unwrap(),
                Signal(-1.0),
            ),
            (
                Date::parse_from_str("2015-11-18", "%Y-%m-%d").unwrap(),
                Signal(1.0),
            ),
            (
                Date::parse_from_str("2018-01-09", "%Y-%m-%d").unwrap(),
                Signal(1.0),
            ),
            (
                Date::parse_from_str("2018-01-10", "%Y-%m-%d").unwrap(),
                Signal(1.0),
            ),
            (
                Date::parse_from_str("2018-05-22", "%Y-%m-%d").unwrap(),
                Signal(-1.0),
            ),
            (
                Date::parse_from_str("2018-09-25", "%Y-%m-%d").unwrap(),
                Signal(1.0),
            ),
            (
                Date::parse_from_str("2018-12-20", "%Y-%m-%d").unwrap(),
                Signal(-1.0),
            ),
            (
                Date::parse_from_str("2019-04-11", "%Y-%m-%d").unwrap(),
                Signal(1.0),
            ),
            (
                Date::parse_from_str("2019-04-16", "%Y-%m-%d").unwrap(),
                Signal(-1.0),
            ),
            (
                Date::parse_from_str("2020-07-21", "%Y-%m-%d").unwrap(),
                Signal(1.0),
            ),
        ];

        assert_eq!(
            crossovers
                .into_iter()
                .cloned()
                .collect::<Vec<(Date, Signal)>>(),
            expected,
        );
    }

    // impl FromIterator<(Date, Signal)> for Vec {

    // }
}
