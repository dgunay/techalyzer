// TODO: make this private to the crate (control exported symbols in general)

use chrono::NaiveDate;
use std::collections::BTreeMap;
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
        return Err(ClampError { min: min, max: max });
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

pub fn nearly_equal(a: f64, b: f64) -> bool {
    f64::abs(a - b) < 0.000001
}

pub fn first_key<K, V>(map: &BTreeMap<K, V>) -> Option<&K> {
    return map.keys().nth(0);
}

pub fn last_key<K, V>(map: &BTreeMap<K, V>) -> Option<&K> {
    return map.keys().last();
}

pub fn first_value<K, V>(map: &BTreeMap<K, V>) -> Option<&V> {
    return map.values().nth(0);
}

pub fn last_value<K, V>(map: &BTreeMap<K, V>) -> Option<&V> {
    return map.values().last();
}

pub fn today_naive() -> NaiveDate {
    chrono::Utc::now().naive_local().date()
}

/// Computes the slope between two floating point values. Normalized to a scale
/// where 0.5 is a 45 degree angle upwards.
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
