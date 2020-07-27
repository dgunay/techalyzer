use ta::indicators::*;

/// buy/sell signals given by a technical indicator.
struct Signals {
    /// 1.0 for an absolute buy, -1.0 for an absolute short, 0.0 for do nothing.
    signals: Vec<f64>
}

impl From<BollingerBands> for Signals {
    fn from(bb: BollingerBands) -> Self {
        todo!()
    }
}