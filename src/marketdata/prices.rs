use alphavantage::time_series::TimeSeries;
use chrono::NaiveDate;

use crate::output::TechalyzerPrintOutput;
use serde::Deserialize;

/// Contains a time series prices data
#[derive(Deserialize, Debug, PartialEq)]
pub struct Prices {
    pub map: std::collections::BTreeMap<NaiveDate, f64>,
    pub symbol: String,
}

impl From<TimeSeries> for Prices {
    fn from(t: TimeSeries) -> Self {
        let mut m = std::collections::BTreeMap::new();
        for e in t.entries {
            m.insert(e.date.naive_local().date(), e.close);
        }

        Prices {
            // dates: t.entries.iter().map(|e| e.date.naive_local().date()).collect(),
            // prices: t.entries.iter().map(|e| e.close).collect(),
            symbol: t.symbol,
            map: m,
        }
    }
}

impl From<TechalyzerPrintOutput> for Prices {
    fn from(t: TechalyzerPrintOutput) -> Self {
        Prices {
            symbol: t.symbol,
            map: t.map.iter().map(|e| (e.0.clone(), e.1.price)).collect(),
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
        // let dt = Eastern.ymd(2012, 2, 2);
        // let dt = NaiveDate::from_ymd(2012, 2, 2);
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

        // assert!(p..price == 30.0);
        assert!(p.map[&dt.naive_local().date()] == 30.0);
        assert!(p.map.iter().next().unwrap().0 == &dt.date().naive_local());
    }
}
