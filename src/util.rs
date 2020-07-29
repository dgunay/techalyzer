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
    (a - b) < 0.000001
}

#[cfg(test)]
mod tests {
    use super::clamp;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(0.5, -1.0, 1.0).unwrap(), 0.5);
        assert_eq!(clamp(2.0, -1.0, 1.0).unwrap(), 1.0);
    }
}
