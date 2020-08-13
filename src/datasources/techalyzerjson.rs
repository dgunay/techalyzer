/// Reingests JSON text that the techalyzer program outputs, in order to
/// get data.
use crate::datasources::datasource::{DataSource, Error};
use crate::output::TechalyzerPrintOutput;

use crate::marketdata::prices::Prices;

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
    fn get(
        &self,
        symbol: &str,
        // start: Option<Date>,
        // end: Option<Date>,
    ) -> Result<Prices, Error> {
        // Read in the JSON and deserialize in a stream
        let reader = BufReader::new(&self.file);
        let data: TechalyzerPrintOutput = match serde_json::from_reader(reader) {
            Ok(d) => d,
            Err(e) => {
                return Err(Error::Other(
                    "Failed to deserialize from file as JSON".to_string(),
                    e.to_string(),
                ))
            }
        };

        // TODO: does this hold up to UTF-8?
        if data.symbol.to_lowercase() != symbol.to_lowercase() {
            return Err(Error::SymbolMismatch {
                expected: symbol.to_string(),
                actual: data.symbol,
            });
        }

        Ok(Prices {
            map: data.map.iter().map(|e| (*e.0, e.1.price)).collect(),
            symbol: data.symbol,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;

    use crate::Date;
    use std::env::current_dir;

    #[test]
    fn test_full_time_series() {
        let tj = TechalyzerJson::new(Path::new("./test/json/jpm_rsi.json")).unwrap();
        let p = tj.get("jpm").unwrap();
        assert_eq!(p.map.len(), 100);
    }

    #[test]
    fn test_prices_from_json() {
        print!("{:?}", current_dir().unwrap());

        let path = Path::new("./test/json/jpm_rsi.json");
        let tj = TechalyzerJson::new(path).unwrap();
        let begin = Date::from_ymd(2020, 3, 10);
        let end = Date::from_ymd(2020, 3, 12);
        let p = tj.get_date_range("jpm", begin..=end).unwrap();

        let mut m = BTreeMap::new();
        m.insert(
            Date::parse_from_str("2020-03-10", "%Y-%m-%d").unwrap(),
            100.7,
        );
        m.insert(
            Date::parse_from_str("2020-03-11", "%Y-%m-%d").unwrap(),
            95.96,
        );
        m.insert(
            Date::parse_from_str("2020-03-12", "%Y-%m-%d").unwrap(),
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
