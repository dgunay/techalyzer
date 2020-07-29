use crate::analysis::signals::Signals;
use ta::indicators::{BollingerBands, BollingerBandsOutput};
use ta::Next;

struct BollingerBandsSignals<'a> {
    outputs: Vec<BollingerBandsOutput>,
    prices: &'a Vec<f64>,
    signals: Vec<f64>,
}

impl<'a> BollingerBandsSignals<'a> {
    pub fn new(prices: &'a Vec<f64>, mut bb: BollingerBands) -> Self {
        // Generate signals as %BB
        // [(Price – Lower Band) / (Upper Band – Lower Band)] * 100
        // https://www.fidelity.com/learning-center/trading-investing/technical-analysis/technical-indicator-guide/percent-b
        let mut signals = Vec::<f64>::new();
        let mut outputs = Vec::new();
        for price in prices.iter() {
            let o = bb.next(*price);

            // how far along from the average to the bounds is the price?
            let signal = 2.0 * (((price - o.lower) / (o.upper - o.lower)) - 0.5);

            signals.push(signal);
            outputs.push(o);
        }

        Self {
            outputs: outputs,
            prices: prices,
            signals: signals,
        }
    }
}

impl Signals for BollingerBandsSignals<'_> {
    fn signals(&mut self) -> &Vec<f64> {
        &self.signals
    }
}

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
    fn test_signals_from_bollinger_bands() {
        let mut bb = BollingerBands::new(5, 2.0).unwrap();

        let prices = vec![1.9, 2.0, 2.1, 2.2, 2.1, 1.5];

        let mut signals = BollingerBandsSignals::new(&prices, BollingerBands::new(5, 2.0).unwrap());

        assert_eq!(signals.signals().len(), prices.len());
        assert!(nearly_equal(signals.signals()[1], 0.5));
        assert!(nearly_equal(signals.signals()[5], -0.96698));
    }
}
