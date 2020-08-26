//! Newtype wrapper for a stock ticker symbol as a String.

use derive_more::FromStr;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// A stock ticker symbol.
#[derive(Debug, Default, Serialize, Deserialize, FromStr, PartialEq)]
#[serde(transparent)]
pub struct Symbol(String);
impl Symbol {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl Deref for Symbol {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
