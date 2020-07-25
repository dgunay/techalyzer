use alphavantage::time_series::TimeSeries;
use peroxide::structure::dataframe::DataFrame;

/// TODO: This is only here because I haven't figured out my standard MarketData
/// object yet
pub struct StubMarketData {
    prices: Prices,
}

impl From<TimeSeries> for StubMarketData {
    /// Builds from an alphavantage TimeSeries object
    fn from(t: TimeSeries) -> Self {
        t.into()
    }
}

/// Wraps DataFrame to enable conversion from various data sources.
struct Prices {
    df: DataFrame,
    symbol: String,
}

impl Into<Prices> for TimeSeries {
    fn into(self) -> Prices {
        todo!("implement Into<Prices> for TimeSeries")
    }
}
