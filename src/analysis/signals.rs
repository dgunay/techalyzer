/// buy/sell signals given by a technical indicator.
pub trait Signals {
    /// 1.0 for an absolute buy, -1.0 for an absolute short, 0.0 for do nothing.
    fn signals(&mut self) -> &Vec<f64>;
}
