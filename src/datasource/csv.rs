//! Gets price info from a CSV file.

use super::{DataSource, Error};
use crate::{date::Date, marketdata, util::TimeSeries};
use csv::StringRecord;
use marketdata::Prices;
use std::{fs::File, path::Path, str::FromStr};

/// The CSV file must have "date" and "adjusted close" or "adj. close" columns.
pub struct CsvFile {
    file: File,
}

impl CsvFile {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        Ok(Self { file })
    }
}

impl DataSource for CsvFile {
    fn get(&self, symbol: &str) -> Result<marketdata::Prices, Error> {
        // Must be at least a Date and Adjusted Close column.
        let mut reader = csv::Reader::from_reader(&self.file);
        let headers = reader
            .headers()
            .map_err(|e| Error::CsvError(e.to_string()))?;

        let (date_idx, adj_close_idx) = required_field_indices(headers)?;

        let mut time_series = TimeSeries::new();
        for row in reader.records() {
            let row = row.map_err(|e| Error::CsvError(e.to_string()))?;
            let date = parse_date_in_csv(get_at_row_idx(&row, date_idx)?)?;
            let price = f64::from_str(get_at_row_idx(&row, adj_close_idx)?)
                .map_err(|e| Error::CsvError(e.to_string()))?;

            time_series.insert(date, price);
        }

        Ok(Prices {
            map: time_series,
            symbol: symbol.into(),
        })
    }
}

fn parse_date_in_csv(datestr: &str) -> Result<Date, Error> {
    // try multiple date formats
    let result = Date::parse_from_str(datestr, "%Y-%m-%d")
        .or_else(|_| Date::parse_from_str(datestr, "%Y/%m/%d"));
    result.map_err(|e| Error::CsvError(e.to_string()))
}

fn get_at_row_idx(row: &StringRecord, idx: usize) -> Result<&str, Error> {
    row.get(idx)
        .ok_or(Error::CsvError(format!("No field at index {}", idx)))
}

fn required_field_indices(row: &StringRecord) -> Result<(usize, usize), Error> {
    let mut date_index = -1;
    let mut adj_close_index = -1;
    for (i, field) in row.iter().enumerate() {
        let lowered = field.to_lowercase();
        match lowered.as_str() {
            "date" => date_index = i as i32,
            "adjusted close" | "adj. close" => adj_close_index = i as i32,
            _ => {}
        }
    }

    if date_index == -1 || adj_close_index == -1 {
        return Err(Error::CsvError(
            "Header does not have required columns (\"date\" and either \"adjusted close\" or \"adj. close\")".to_string()
        ));
    }

    Ok((date_index as usize, adj_close_index as usize))
}

#[cfg(test)]
mod tests {
    use super::CsvFile;
    use crate::{datasource::DataSource, date::Date};
    use std::path::Path;

    #[test]
    fn open_well_formed_csv() {
        let a = CsvFile::new(Path::new("test/csv/tsla.csv")).unwrap();
        let prices = a.get("TSLA").unwrap();

        assert_eq!(
            prices.first_entry().unwrap(),
            (&Date::from_ymd(2018, 10, 02), &301.0200)
        );
    }
}
