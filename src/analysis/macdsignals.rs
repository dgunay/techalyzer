use crate::analysis::signals::Signals;
use ta::indicators::MovingAverageConvergenceDivergence;
use ta::Next;
use crate::util::clamp;
use gnuplot::Figure;

struct MovingAverageConvergenceDivergenceSignals {
    // outputs: Vec<ta::Next::Output<MovingAverageConvergenceDivergence>, &'a f64>,
    outputs: Vec<(f64, f64, f64)>,
    // prices: &'a Vec<f64>,
    signals: Vec<f64>,
}

impl MovingAverageConvergenceDivergenceSignals {
    pub fn new(prices: &Vec<f64>, mut macd: MovingAverageConvergenceDivergence) -> Self {
        // TODO: should I check prices not empty?

        // Generate signals from MACD
        let mut signals = Vec::<f64>::new();

        // FIXME: can't get outputs to work - do we even need them?
        let mut outputs = Vec::new();
        let mut macd_line_prev = 0.0;
        for price in prices.iter() {
            // FIXME: for some reason I have to clone the price or next() won't
            // work - maybe an upstream PR is necessary
            let tuple = macd.next(price.clone());
            let (macd_line, signal_line, histo) = tuple;

            // FIXME: I can't think of a great way to do a normalized -1.0 to 1.0
            // scale on the MACD, so for now I'll go with having above/below be
            // 0.5/-0.5, and then just add the slope of the MACD.
            let above_or_below = if macd_line > signal_line {
                0.5
            } else if macd_line < signal_line {
                -0.5
            } else {
                0.0
            };
            
            // slope is normalized and clamped to 0.5/-0.5 (with 0.5 being a 45 
            // degree angle trending upwards)
            // FIXME: ensure no div by zero
            let slope = macd_line - macd_line_prev;
            let norm_macd_slope = (slope / macd_line_prev) / 2.0;
            let signal = clamp(norm_macd_slope, -0.5, 0.5).unwrap() + above_or_below;

            signals.push(signal);
            outputs.push(tuple);
        }

        Self {
            outputs: outputs,
            // prices: prices,
            signals: signals,
        }
    }
}

impl Signals for MovingAverageConvergenceDivergenceSignals {
    fn signals(&mut self) -> &Vec<f64> {
        &self.signals
    }
}

mod tests {
    use super::MovingAverageConvergenceDivergenceSignals;
    use ta::indicators::MovingAverageConvergenceDivergence;
    use super::Signals;
    // use crate::util::nearly_equal;
    use gnuplot::Figure;

    struct Close {
        price: f64,
    }

    impl ta::Close for Close {
        fn close(&self) -> f64 {
            self.price
        }
    }

    // struct SignalFloat {
    //     f: f64
    // }

    // impl PartialOrd for SignalFloat {

    // }

    #[test]
    fn test_signals_from_macd() {
        let prices = vec![1.9, 2.0, 2.1, 2.2, 2.1, 1.5, 1.3, 1.2, 1.1, 1.0];

        let mut signals = MovingAverageConvergenceDivergenceSignals::new(
            &prices,
            MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap(),
        );

        // println!("{:?}", signals.signals());
        // println!("{:?}", signals.outputs);
        // println!("{:?}", prices);

        // TODO: test more specific values/calculations after plotting is
        // implemented
        let nan = 0./0.;
        assert!(signals.signals().iter().cloned().fold(nan, f64::max) <= 1.0);
        assert!(signals.signals().iter().cloned().fold(nan, f64::min) >= -1.0);
    }
}
