use alphavantage::time_series::TimeSeries;
use chrono::NaiveDate;

use crate::{datasources::alphavantage::entry_to_naivedate, output::TechalyzerPrintOutput};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::RangeBounds};

/// Contains a time series prices data
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Prices {
    pub map: BTreeMap<NaiveDate, f64>,
    pub symbol: String,
}

impl Prices {
    pub fn iter(&self) -> std::collections::btree_map::Iter<NaiveDate, f64> {
        self.map.iter()
    }

    pub fn date_range(&self, range: impl RangeBounds<NaiveDate>) -> Prices {
        let slice = self
            .map
            .range(range)
            .map(|e| (e.0.clone(), e.1.clone()))
            .collect();

        Prices {
            map: slice,
            symbol: self.symbol.clone(),
        }
    }
}

// // structure helper for non-consuming iterator.
// struct PriceIterator {
//     iter: ::std::collections::btree_map::Iter<NaiveDate, f64>,
// }

// // implement the IntoIterator trait for a non-consuming iterator. Iteration will
// // borrow the Words structure
// impl IntoIterator for Prices {
//     type Item = (NaiveDate, f64);
//     type IntoIter = PriceIterator;

//     // note that into_iter() is consuming self
//     fn into_iter(self) -> Self::IntoIter {
//         PriceIterator {
//             iter: self.map.iter(),
//         }
//     }
// }

// // now, implements Iterator trait for the helper struct, to be used by adapters
// impl Iterator for PriceIterator {
//     type Item = (NaiveDate, f64);

//     // just return the str reference
//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next()
//     }
// }

impl From<TimeSeries> for Prices {
    fn from(t: TimeSeries) -> Self {
        let mut m = std::collections::BTreeMap::new();
        for e in t.entries {
            m.insert(entry_to_naivedate(Some(&e)), e.close);
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
    use chrono::{Duration, NaiveDate, TimeZone};
    use chrono_tz::US::Eastern;
    use std::collections::BTreeMap;

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

        assert!(p.map[&dt.naive_local().date()] == 30.0);
        assert!(p.map.iter().next().unwrap().0 == &dt.date().naive_local());
    }

    /// Creates a month of Prices
    fn fixture_setup() -> Prices {
        let start = NaiveDate::from_ymd(2012, 1, 2);
        let end = NaiveDate::from_ymd(2012, 2, 2);
        let mut dt = start;
        let mut entries = BTreeMap::new();
        while dt <= end {
            entries.insert(dt, 30.0);
            // dt = dt.and_hms(1, 1, 1) + Duration::days(1);
            dt = dt + Duration::days(1);
        }

        Prices {
            map: entries,
            symbol: "jpm".to_string(),
        }
    }

    #[test]
    fn prices_date_range() {
        let p = fixture_setup();
        let start = NaiveDate::from_ymd(2012, 1, 5);
        let end = NaiveDate::from_ymd(2012, 1, 6);
        let result = p.date_range(start..=end);
        assert_eq!(result.map.len(), 2);
    }
}
