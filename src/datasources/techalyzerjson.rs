/// Reingests JSON text that the techalyzer program outputs, in order to
/// get data.
use crate::datasources::datasource::{DataSource, Error};
use crate::output::TechalyzerPrintOutput;
use crate::Prices;
use chrono::NaiveDate;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct TechalyzerJson {
    file: File,
}

impl TechalyzerJson {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        Ok(TechalyzerJson {
            file: match File::open(&path) {
                Ok(f) => f,
                Err(e) => return Err(e),
            },
        })
    }
}

impl DataSource for TechalyzerJson {
    fn get(self, symbol: &str, start: NaiveDate, end: NaiveDate) -> Result<Prices, Error> {
        let reader = BufReader::new(self.file);
        let data: TechalyzerPrintOutput = serde_json::from_reader(reader).unwrap(); // FIXME: don't unwrap
        let slice: BTreeMap<NaiveDate, f64> = data
            .map
            .range(start..=end)
            .map(|e| (e.0.clone(), e.1.price))
            .collect();
        Ok(Prices {
            map: slice,
            symbol: data.symbol,
        })
    }
}

// TODO: this should really be solved by just using a more sensible data structure
// in the first place like, idk, a map of date => value ordered by insertion or
// sorted by date.
// fn prices_from_date_range(t: TechalyzerPrintOutput,
//     start: NaiveDate,
//     end: NaiveDate,
// ) {
//     let mut dates = vec![];
//     let mut start_ix = 0;
//     let mut end_ix = t.dates.len() - 1;
//     for (i, date) in t.dates.iter().enumerate() {
//         if date == start {
//             start_ix = i;
//         }

//         if date == end {
//             end_ix = i;
//         }

//         // if date >= &start && date <= &end {
//         //     dates.push(date.clone());
//         //     if t.outputs.is_some() {
//         //         outputs.push(t.outputs.unwrap().);
//         //     }
//         // }
//     }
//     let dates = t.dates[start_ix..end_ix].to_vec();
//     let outputs =  if t.outputs.is_some() {
//         t.outputs.
//     } else { None };
//     let prices = t.prices[start_ix..end_ix].to_vec();
//     let signals = t.signals[start_ix..end_ix].to_vec();
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::TechalyzerEntry;
    use crate::signals::signals::Outputs;
    use std::collections::BTreeMap;

    use std::env::current_dir;

    #[test]
    fn test_prices_from_json() {
        print!("{:?}", current_dir().unwrap());

        let path = Path::new("./test/json/jpm_rsi.json");
        let tj = TechalyzerJson::new(path).unwrap();
        let begin = NaiveDate::from_ymd(2020, 3, 10);
        let end = NaiveDate::from_ymd(2020, 3, 12);
        let p = tj.get("jpm", begin, end).unwrap();

        let mut m = BTreeMap::new();
        m.insert(
            NaiveDate::parse_from_str("2020-03-10", "%Y-%m-%d").unwrap(),
            100.7,
        );
        m.insert(
            NaiveDate::parse_from_str("2020-03-11", "%Y-%m-%d").unwrap(),
            95.96,
        );
        m.insert(
            NaiveDate::parse_from_str("2020-03-12", "%Y-%m-%d").unwrap(),
            88.05,
        );
        assert_eq!(
            p,
            Prices {
                symbol: "jpm".to_string(),
                map: m
            }
        );
    }
}
