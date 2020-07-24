// TODO: library code goes here

mod datasources;
pub mod source;
pub mod secret;

use chrono::NaiveDate;
use crate::datasources::alphavantage;
use crate::datasources::datasource::DataSource;
use crate::source::Source;
use secret::Secret;

pub fn get_market_data(source: Source, start_date: NaiveDate, end_date: NaiveDate, secret: Secret) {
  match source {
    Source::AlphaVantage => {
      let key = secret.data.unwrap();
      let cl = ::alphavantage::blocking::Client::new(key.as_str());
      let av = alphavantage::AlphaVantage::new(cl);
      let res = av.get("JPM", start_date, end_date);      
      print!("{:?}", res.entries.first());
    }
  }
}