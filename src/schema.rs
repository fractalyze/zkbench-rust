// Copyright 2026 Plonky3 Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Schema types for benchmark reporting.

use std::collections::HashMap;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::platform::Platform;

/// Represents a benchmark metric with optional confidence bounds.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricValue {
    pub value: f64,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper_value: Option<f64>,
}

impl MetricValue {
    /// Creates a new MetricValue with just value and unit.
    pub fn new(value: f64, unit: &str) -> Self {
        Self {
            value,
            unit: unit.to_string(),
            lower_value: None,
            upper_value: None,
        }
    }

    /// Creates a new MetricValue with confidence bounds.
    pub fn with_bounds(value: f64, unit: &str, lower: f64, upper: f64) -> Self {
        Self {
            value,
            unit: unit.to_string(),
            lower_value: Some(lower),
            upper_value: Some(upper),
        }
    }
}

/// Test vector verification information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVectors {
    pub input_hash: String,
    pub output_hash: String,
    pub verified: bool,
}

/// Represents results from a single benchmark.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<MetricValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MetricValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput: Option<MetricValue>,
    #[serde(skip_serializing_if = "is_zero", default)]
    pub iterations: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_vectors: Option<TestVectors>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, Value>,
}

fn is_zero(val: &usize) -> bool {
    *val == 0
}

/// Benchmark metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub implementation: String,
    pub version: String,
    pub commit_sha: String,
    pub timestamp: String,
    pub platform: Platform,
}

impl Metadata {
    /// Creates metadata with auto-detected platform and git info.
    pub fn create(implementation: &str, version: &str) -> Self {
        Self {
            implementation: implementation.to_string(),
            version: version.to_string(),
            commit_sha: get_git_commit_sha(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            platform: Platform::current(),
        }
    }
}

/// Complete benchmark report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub metadata: Metadata,
    pub benchmarks: HashMap<String, BenchmarkResult>,
}

/// Gets the current git commit SHA (first 12 characters).
fn get_git_commit_sha() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim()[..12.min(s.trim().len())].to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_value_new() {
        let metric = MetricValue::new(100.5, "ns");
        assert!((metric.value - 100.5).abs() < f64::EPSILON);
        assert_eq!(metric.unit, "ns");
        assert!(metric.lower_value.is_none());
        assert!(metric.upper_value.is_none());
    }

    #[test]
    fn test_metric_value_with_bounds() {
        let metric = MetricValue::with_bounds(100.0, "ms", 95.0, 105.0);
        assert!((metric.value - 100.0).abs() < f64::EPSILON);
        assert_eq!(metric.unit, "ms");
        assert_eq!(metric.lower_value, Some(95.0));
        assert_eq!(metric.upper_value, Some(105.0));
    }

    #[test]
    fn test_metric_value_default() {
        let metric = MetricValue::default();
        assert!((metric.value - 0.0).abs() < f64::EPSILON);
        assert!(metric.unit.is_empty());
        assert!(metric.lower_value.is_none());
        assert!(metric.upper_value.is_none());
    }

    #[test]
    fn test_metric_value_serialization() {
        let metric = MetricValue::new(42.0, "ops/s");
        let json = serde_json::to_string(&metric).unwrap();
        assert!(json.contains("42"));
        assert!(json.contains("ops/s"));
        // lower_value and upper_value should be skipped when None
        assert!(!json.contains("lower_value"));
        assert!(!json.contains("upper_value"));
    }

    #[test]
    fn test_metric_value_serialization_with_bounds() {
        let metric = MetricValue::with_bounds(100.0, "ns", 90.0, 110.0);
        let json = serde_json::to_string(&metric).unwrap();
        assert!(json.contains("lower_value"));
        assert!(json.contains("upper_value"));
    }

    #[test]
    fn test_metric_value_deserialization() {
        let json = r#"{"value": 50.0, "unit": "MB"}"#;
        let metric: MetricValue = serde_json::from_str(json).unwrap();
        assert!((metric.value - 50.0).abs() < f64::EPSILON);
        assert_eq!(metric.unit, "MB");
    }

    #[test]
    fn test_test_vectors() {
        let tv = TestVectors {
            input_hash: "abc123".to_string(),
            output_hash: "def456".to_string(),
            verified: true,
        };
        let json = serde_json::to_string(&tv).unwrap();
        let deserialized: TestVectors = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.input_hash, "abc123");
        assert_eq!(deserialized.output_hash, "def456");
        assert!(deserialized.verified);
    }

    #[test]
    fn test_benchmark_result_default() {
        let result = BenchmarkResult::default();
        assert!(result.latency.is_none());
        assert!(result.memory.is_none());
        assert!(result.throughput.is_none());
        assert_eq!(result.iterations, 0);
        assert!(result.test_vectors.is_none());
        assert!(result.metadata.is_empty());
    }

    #[test]
    fn test_benchmark_result_serialization_skips_none() {
        let result = BenchmarkResult {
            latency: Some(MetricValue::new(100.0, "ns")),
            ..Default::default()
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("latency"));
        assert!(!json.contains("memory"));
        assert!(!json.contains("throughput"));
        assert!(!json.contains("iterations"));
        assert!(!json.contains("test_vectors"));
    }

    #[test]
    fn test_benchmark_result_full() {
        let result = BenchmarkResult {
            latency: Some(MetricValue::new(100.0, "ns")),
            memory: Some(MetricValue::new(1024.0, "KB")),
            throughput: Some(MetricValue::new(1000.0, "ops/s")),
            iterations: 100,
            test_vectors: Some(TestVectors {
                input_hash: "input".to_string(),
                output_hash: "output".to_string(),
                verified: true,
            }),
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: BenchmarkResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.latency.is_some());
        assert!(deserialized.memory.is_some());
        assert!(deserialized.throughput.is_some());
        assert_eq!(deserialized.iterations, 100);
        assert!(deserialized.test_vectors.is_some());
    }

    #[test]
    fn test_is_zero() {
        assert!(is_zero(&0));
        assert!(!is_zero(&1));
        assert!(!is_zero(&100));
    }

    #[test]
    fn test_metadata_create() {
        let metadata = Metadata::create("test-impl", "1.0.0");
        assert_eq!(metadata.implementation, "test-impl");
        assert_eq!(metadata.version, "1.0.0");
        // commit_sha should be either a valid sha or "unknown"
        assert!(!metadata.commit_sha.is_empty());
        // timestamp should be a valid RFC3339 string
        assert!(metadata.timestamp.contains('T'));
    }

    #[test]
    fn test_benchmark_report() {
        let mut benchmarks = HashMap::new();
        benchmarks.insert(
            "bench1".to_string(),
            BenchmarkResult {
                latency: Some(MetricValue::new(50.0, "ns")),
                ..Default::default()
            },
        );

        let report = BenchmarkReport {
            metadata: Metadata::create("test", "0.1.0"),
            benchmarks,
        };

        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("metadata"));
        assert!(json.contains("benchmarks"));
        assert!(json.contains("bench1"));
    }

    #[test]
    fn test_benchmark_report_roundtrip() {
        let mut benchmarks = HashMap::new();
        benchmarks.insert(
            "my_bench".to_string(),
            BenchmarkResult {
                latency: Some(MetricValue::with_bounds(100.0, "ns", 95.0, 105.0)),
                throughput: Some(MetricValue::new(10000.0, "ops/s")),
                iterations: 1000,
                ..Default::default()
            },
        );

        let report = BenchmarkReport {
            metadata: Metadata::create("roundtrip-test", "2.0.0"),
            benchmarks,
        };

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: BenchmarkReport = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.metadata.implementation,
            report.metadata.implementation
        );
        assert_eq!(deserialized.metadata.version, report.metadata.version);
        assert!(deserialized.benchmarks.contains_key("my_bench"));
    }
}
