//! Utility functions, traits, and structs with general use in various parts of
//! Techalyzer.

mod symbol;
pub use symbol::*;

use crate::date::Date;
use serde::Serialize;
use std::collections::BTreeMap;

/// Used for every time series type - aliases a BTreeMap of Date to whatever.
pub type TimeSeries<T> = BTreeMap<Date, T>;

/// Error returned in the `clamp` function.
#[derive(Debug)]
pub struct ClampError {
    min: f64,
    max: f64,
}

impl std::fmt::Display for crate::util::ClampError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!("Bounds {} and {} overlap", self.min, self.max)
        )
    }
}

/// Clamps `a` between `min` and `max` inclusive.
pub fn clamp(a: f64, min: f64, max: f64) -> Result<f64, ClampError> {
    if min > max {
        return Err(ClampError { min, max });
    }

    let res = if a > max {
        max
    } else if a < min {
        min
    } else {
        a
    };

    Ok(res)
}

/// Very quick and dirty floating point epsilon comparison for testing - NOT in
/// any way meant to be a robust function.
pub fn nearly_equal(a: f64, b: f64) -> bool {
    f64::abs(a - b) < 0.000001
}

/// Gets the first key from a BTreeMap
pub fn first_key<K, V>(map: &BTreeMap<K, V>) -> Option<&K> {
    map.keys().next()
}

/// Gets the last key from a BTreeMap
pub fn last_key<K, V>(map: &BTreeMap<K, V>) -> Option<&K> {
    map.keys().last()
}

/// Gets the first value from a BTreeMap
pub fn first_value<K, V>(map: &BTreeMap<K, V>) -> Option<&V> {
    map.values().next()
}

/// Gets the last value from a BTreeMap
pub fn last_value<K, V>(map: &BTreeMap<K, V>) -> Option<&V> {
    map.values().last()
}

/// Trait that quickly and easily grafts a `to_json()` method onto any struct
/// that supports serde.
pub trait ToJson: Serialize {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }
}

/// Computes the slope between two floating point values. Normalized to a scale
/// where 0.5 is a 45 degree angle upwards. The scaling is not perfect due to
/// the use of arc tangent.
pub fn slope(current: f64, prev: f64, run: f64) -> f64 {
    let rise = current - prev;
    (rise / run).atan() / std::f64::consts::FRAC_PI_2

    // TODO: is there any way to do this with a more even scaling than using
    // trig functions?

    // let run = current - prev;

    // // Requires special logic to avoid NaN
    // let scaling_factor = if current == 0.0 {
    //     2.0
    // } else {
    //     1.0 / (1.0 * current)
    // };

    // let left = run  / scaling_factor;

    // // Normalize across run size (so (0,0) -> (1,1) is the same as (0,0) -> (5,5))
    // let run_scaling_factor = 1.0 / run;

    // left * run_scaling_factor
}

#[cfg(test)]
mod tests {
    use super::clamp;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(0.5, -1.0, 1.0).unwrap(), 0.5);
        assert_eq!(clamp(2.0, -1.0, 1.0).unwrap(), 1.0);
    }

    #[test]
    fn test_slope() {
        // simple one step cases

        // TODO: get these tests to work
        // No slope (flat)
        // assert!(nearly_equal(slope(1.0, 1.0, 1.0), 0.0));
        // assert!(nearly_equal(slope(10.0, 10.0, 1.0), 0.0));
        // let res = slope(0.0, 0.0, 1.0);
        // assert!(nearly_equal(res, 0.0));

        // // Slope (not flat)
        // assert!(nearly_equal(slope(0.0, 1.0, 1.0), -0.5));
        // let res = slope(0.5, 0.0, 1.0);
        // print!("{}", res);
        // assert!(nearly_equal(res, 0.25));
        // assert!(nearly_equal(slope(2.0, 0.0, 1.0), 0.75));
        // assert!(nearly_equal(slope(99.0, 0.0, 1.0), 0.99));

        // // No run/infinite slope
        // assert_eq!(slope(0.0, 100.0, 0.0), f64::NEG_INFINITY);
    }
}
