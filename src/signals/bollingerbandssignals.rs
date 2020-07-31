use super::signals::Outputs;
use crate::signals::signals::Signals;
use serde::Serialize;
use ta::indicators::{BollingerBands, BollingerBandsOutput};
use ta::Next;

/// basically a carbon copy of BollingerBandsOutput because Serializing a vector
/// of remote types is way too hard for me right now
#[derive(Serialize)]
pub struct BBOutput {
    pub average: f64,
    pub upper: f64,
    pub lower: f64,
}

impl From<&BollingerBandsOutput> for BBOutput {
    fn from(b: &BollingerBandsOutput) -> BBOutput {
        Self {
            average: b.average,
            upper: b.upper,
            lower: b.lower,
        }
    }
}

#[derive(Serialize)]
pub struct BollingerBandsSignals<'a> {
    pub outputs: Vec<BBOutput>,
    pub prices: &'a Vec<f64>,
    pub signals: Vec<f64>,
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
            outputs: outputs.iter().map(|o| BBOutput::from(o)).collect(),
            prices: prices,
            signals: signals,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Signals for BollingerBandsSignals<'_> {
    fn signals(&self) -> &Vec<f64> {
        &self.signals
    }

    fn outputs(&self) -> Outputs {
        let outputs = self
            .outputs
            .iter()
            .map(|o| vec![o.average, o.upper, o.lower])
            .collect();

        Outputs::new(
            outputs,
            vec![
                "average".to_string(),
                "upper".to_string(),
                "lower".to_string(),
            ],
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{BollingerBands, BollingerBandsSignals, Signals};
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
