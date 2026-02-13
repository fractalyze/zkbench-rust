// Copyright 2026 zkbench-rust Authors
// SPDX-License-Identifier: Apache-2.0

//! Statistical calculations for benchmark data.

/// Calculates mean and standard deviation.
///
/// # Arguments
/// * `values` - Slice of numeric values
///
/// # Returns
/// Tuple of (mean, standard_deviation)
///
/// # Panics
/// Panics if values is empty.
pub fn calculate_statistics(values: &[f64]) -> (f64, f64) {
    assert!(
        !values.is_empty(),
        "Cannot calculate statistics on empty slice"
    );

    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;

    if values.len() < 2 {
        return (mean, 0.0);
    }

    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let stdev = variance.sqrt();

    (mean, stdev)
}

/// Calculates confidence interval bounds.
///
/// Uses a simple z-score approximation:
/// - 95% confidence: z = 2.0 (rounded from 1.96)
/// - 99% confidence: z = 2.576
///
/// # Arguments
/// * `mean` - Sample mean
/// * `stdev` - Sample standard deviation
/// * `confidence` - Confidence level (0.95 for 95%, 0.99 for 99%)
///
/// # Returns
/// Tuple of (lower_bound, upper_bound)
pub fn calculate_confidence_interval(mean: f64, stdev: f64, confidence: f64) -> (f64, f64) {
    let z = if (confidence - 0.95).abs() < 0.001 {
        2.0
    } else if (confidence - 0.99).abs() < 0.001 {
        2.576
    } else {
        2.0
    };

    let margin = z * stdev;
    (mean - margin, mean + margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, stdev) = calculate_statistics(&values);
        assert!((mean - 3.0).abs() < 0.0001);
        assert!((stdev - 1.5811).abs() < 0.001);
    }

    #[test]
    fn test_calculate_confidence_interval() {
        let (lower, upper) = calculate_confidence_interval(100.0, 10.0, 0.95);
        assert!((lower - 80.0).abs() < 0.0001);
        assert!((upper - 120.0).abs() < 0.0001);
    }
}
