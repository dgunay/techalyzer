use crate::Date;
use crate::{datasources::alphavantage::entry_to_date, output::TechalyzerPrintOutput};
use alphavantage::time_series::TimeSeries;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::RangeBounds};

/// Contains a time series prices data
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Prices {
    pub map: BTreeMap<Date, f64>,
    pub symbol: String,
}

impl Prices {
    pub fn iter(&self) -> std::collections::btree_map::Iter<Date, f64> {
        self.map.iter()
    }

    pub fn date_range(&self, range: impl RangeBounds<Date>) -> Prices {
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

    pub fn get(&self, date: &Date) -> Option<&f64> {
        self.map.get(date)
    }

    pub fn get_after(&self, date: &Date, days_after: usize) -> Option<(Date, f64)> {
        // From the given date, go n days after
        // TODO: gross and probably inefficient, is there a way we can hash
        // straight to `date` and then start iterating?
        let mut i_after = 0;
        for pair in &self.map {
            if i_after == 0 && pair.0 == date {
                i_after += 1;
            } else if i_after > 0 {
                if i_after >= days_after {
                    return Some((*pair.0, *pair.1));
                }

                i_after += 1;
            }
        }

        None
    }
}

impl From<TimeSeries> for Prices {
    fn from(t: TimeSeries) -> Self {
        let mut m = std::collections::BTreeMap::new();
        for e in t.entries {
            m.insert(entry_to_date(Some(&e)), e.close);
        }

        Prices {
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
    use chrono::{Duration, TimeZone};
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

        let date = Date::from(dt.naive_local().date());
        assert!(p.map[&date] == 30.0);
        assert!(p.map.iter().next().unwrap().0 == &date);
    }

    /// Creates a month of Prices
    fn fixture_setup() -> Prices {
        let start = Date::from_ymd(2012, 1, 2);
        let end = Date::from_ymd(2012, 2, 2);
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
        let start = Date::from_ymd(2012, 1, 5);
        let end = Date::from_ymd(2012, 1, 6);
        let result = p.date_range(start..=end);
        assert_eq!(result.map.len(), 2);
    }

    #[test]
    fn test_get_after() {
        let p = fixture_setup();
        let date = Date::from_ymd(2012, 1, 14);
        let target = Date::from_ymd(2012, 1, 15);
        let result = p.get_after(&date, 1).unwrap();

        assert_eq!(result.0, target);
    }
}
