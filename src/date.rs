//! A wrapper for Date with some conveniences for Techalyzer.

use chrono::{Duration, NaiveDate, ParseError};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, Sub},
    str::FromStr,
};

/// Wraps Date and provides a default value of today's date.
#[derive(Debug, Display, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Eq, Clone, Copy)]
pub struct Date(NaiveDate);

impl Date {
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        NaiveDate::from_ymd(year, month, day).into()
    }

    pub fn parse_from_str(s: &str, fmt: &str) -> Result<Self, ParseError> {
        NaiveDate::parse_from_str(s, fmt).map(|ok| ok.into())
    }

    /// Creates an inclusive range of Dates from start to end.
    // TODO: this is mainly to get around limitations of Range/RangeBounds.
    // try replacing it with some type that guarantees sorting
    pub fn range(start: Date, end: Date) -> Vec<Date> {
        let mut day = start;
        let mut days = Vec::new();
        while day != end {
            days.push(day);
            day = day + Duration::days(1);
        }

        days
    }
}

pub fn today() -> Date {
    Date(chrono::Utc::now().naive_local().date())
}

impl Default for Date {
    fn default() -> Self {
        today()
    }
}

impl FromStr for Date {
    type Err = chrono::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(NaiveDate::from_str(s)?))
    }
}

impl From<NaiveDate> for Date {
    fn from(d: NaiveDate) -> Self {
        Self(d)
    }
}

impl Sub<Duration> for Date {
    type Output = Date;
    fn sub(self, rhs: Duration) -> Self::Output {
        Date(self.0 - rhs)
    }
}

impl Add<Duration> for Date {
    type Output = Date;
    fn add(self, rhs: Duration) -> Self::Output {
        Date(self.0 + rhs)
    }
}
