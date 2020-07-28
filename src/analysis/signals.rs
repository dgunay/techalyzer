use ta::indicators::{BollingerBands, BollingerBandsOutput};
use ta::Next;

/// buy/sell signals given by a technical indicator.
trait Signals {
    /// 1.0 for an absolute buy, -1.0 for an absolute short, 0.0 for do nothing.
    fn signals(&mut self) -> &Vec<f64>;
}

struct BollingerBandsSignals<'a> {
    bb: BollingerBands,
    outputs: Vec<BollingerBandsOutput>,
    prices: &'a Vec<f64>,
    signals: Vec<f64>
}

impl<'a> BollingerBandsSignals<'a> {
    pub fn new(prices: &'a Vec<f64>, bb: BollingerBands) -> Self {
        Self {
            bb: bb,
            outputs: Default::default(),
            prices: prices,
            signals: Vec::<f64>::new()
        }
    }
}

impl Signals for BollingerBandsSignals<'_> {
    fn signals(&mut self) -> &Vec<f64> {
        if self.prices.len() == self.signals.len() {
            return &self.signals;
        }

        for price in self.prices.iter() {
            self.outputs.push(self.bb.next(*price));
        }

        // Generate signals as %BB
        todo!()
        // &self.signals
    }
}

mod tests {
    use super::*;
    use ta::Next;

    struct Close {
        price: f64,
    }

    impl ta::Close for Close {
        fn close(&self) -> f64 {
            self.price
        }
    }

    // impl ta::Next<Close> for BollingerBands {
    //     type Output;
    //     fn next(&mut self, input: Close) -> Self::Output {
    //         todo!()
    //     }
    // }

    #[test]
    fn test_signals_from_bollinger_bands() {
        let mut bb = BollingerBands::new(5, 2.0).unwrap();

        let prices = vec![
            2.0, 2.0, 2.0, 2.0, 2.2, 2.3, 2.4, 2.5, 2.1, 1.9, 1.8, 1.9, 2.0, 2.0, 2.0, 2.0, 2.0,
            2.2, 2.3, 2.4, 2.5, 2.1, 1.9, 1.8, 1.9, 2.0, 2.0, 2.0, 2.0, 2.0, 2.2, 2.3, 2.4, 2.5,
            2.1, 1.9, 1.8, 1.9, 2.0,
        ];
        
        let signals = BollingerBandsSignals::new(
            &prices, 
            BollingerBands::new(5, 2.0).unwrap()
        );


    }
}
