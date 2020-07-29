use alphavantage::time_series::TimeSeries;
use chrono::DateTime;
use chrono_tz::Tz;
use ndarray::prelude::*;
use std::iter::FromIterator;

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
    // TODO: it is not clear to me yet how to efficiently implement a
    // date-indexed series of prices like pandas, so I'll just use two ndarrays
    // instead
    // TODO: maybe use a map of dates to prices, or a map to indices on the
    // prices array?
    dates: Array1<DateTime<Tz>>,
    prices: Array1<f64>,
    symbol: String,
}

impl Into<Prices> for TimeSeries {
    fn into(self) -> Prices {
        Prices {
            dates: Array::from_iter(self.entries.iter().map(|entry| entry.date)),
            prices: Array::from_iter(self.entries.iter().map(|entry| entry.close)),
            symbol: self.symbol,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alphavantage::time_series::Entry;
    use chrono::prelude::*;
    use chrono_tz::US::Eastern;

    #[test]
    fn create_prices_from_alphavantage_time_series() {
        let dt = Eastern.ymd(2012, 2, 2).and_hms(12, 0, 0);
        let entry = Entry {
            date: dt,
            open: 30.0,
            high: 32.0,
            low: 28.0,
            close: 30.0,
            volume: 300,
        };
        // FIXME: this is highly obnoxious to create fixtures for
        let ts = TimeSeries {
            entries: vec![entry],
            symbol: "JPM".to_string(),
            last_refreshed: dt,
        };

        let p: Prices = ts.into();

        assert!(p.prices[0] == 30.0);
        assert!(p.dates[0] == dt);
    }
}
