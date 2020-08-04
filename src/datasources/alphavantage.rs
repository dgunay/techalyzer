use crate::datasources::datasource::{DataSource, Error};
use crate::Prices;
use alphavantage::blocking::Client; // TODO: use async client
use alphavantage::time_series::Entry;
use chrono::NaiveDate;
use std::collections::BTreeMap;

pub struct AlphaVantage {
    client: Client,
}

impl AlphaVantage {
    pub fn new(client: Client) -> AlphaVantage {
        AlphaVantage { client }
    }
}

impl DataSource for AlphaVantage {
    fn get(
        self,
        symbol: &str,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
    ) -> Result<Prices, Error> {
        // TODO: if start is in the last 100 market days, don't request the full
        // time series datas
        match self.client.get_time_series_daily_full(symbol) {
            Ok(t) => {
                // Slice from start to end date inclusive
                let slice: BTreeMap<NaiveDate, f64> = filter_entries(
                    &t.entries,
                    start.ok_or(entry_to_naivedate(t.entries.first())).unwrap(),
                    end.ok_or(entry_to_naivedate(t.entries.last())).unwrap(),
                );
                Ok(Prices {
                    map: slice,
                    symbol: symbol.to_string(),
                })
            }
            Err(e) => Err(Error::AlphaVantageError(e.to_string())),
        }
    }
}

/// Helper function to convert the date of an Entry into a NaiveDate
fn entry_to_naivedate(entry: Option<&Entry>) -> NaiveDate {
    entry.expect("No first Entry").date.naive_local().date()
}

// Slices the entries from start to end date inclusive
// TODO: can we extract the date filtering logic and generalize it across
// data sources? The consumer of that API need only specific where the date and
// close price of each entry is.
fn filter_entries(
    entries: &Vec<Entry>,
    start: NaiveDate,
    end: NaiveDate,
) -> BTreeMap<NaiveDate, f64> {
    entries
        .iter()
        .filter_map(|entry| {
            let naive_date = entry.date.naive_local().date();
            return if naive_date >= start && naive_date <= end {
                Some((naive_date, entry.close))
            } else {
                None
            };
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alphavantage::time_series::TimeSeries;
    use chrono::prelude::*;
    use chrono::Duration;
    use chrono_tz::US::Eastern;

    #[test]
    fn test_date_range_filters_inclusive() {
        let start = Eastern.ymd(2012, 1, 2).and_hms(12, 0, 0);
        let end = Eastern.ymd(2012, 2, 2).and_hms(12, 0, 0);
        let mut dt = start;
        let mut entries = Vec::new();
        while dt <= end {
            dt = dt + Duration::days(1);
            let entry = Entry {
                date: dt,
                open: 30.0,
                high: 32.0,
                low: 28.0,
                close: 30.0,
                volume: 300,
            };

            entries.push(entry);
        }

        let ts = TimeSeries {
            entries: entries,
            symbol: "JPM".to_string(),
            last_refreshed: dt,
        };

        let res = filter_entries(
            &ts.entries,
            NaiveDate::from_ymd(2012, 1, 15),
            NaiveDate::from_ymd(2012, 1, 16),
        );

        assert!(res.len() == 2);
        let first = res.keys().nth(0).unwrap();
        let last = res.keys().nth(1).unwrap();
        assert_eq!(first.clone(), NaiveDate::from_ymd(2012, 1, 15));
        assert_eq!(last.clone(), NaiveDate::from_ymd(2012, 1, 16));
    }
}
