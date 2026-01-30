// Copyright 2026 Plonky3 Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! zkbench - Reusable benchmarking library for zero-knowledge proofs.
//!
//! This crate provides common types and utilities for benchmark reporting
//! across different ZK implementations with a standardized JSON schema.
//!
//! # Example
//!
//! ```
//! use zkbench::{BenchmarkReport, BenchmarkResult, Metadata, MetricValue};
//! use std::collections::HashMap;
//!
//! let result = BenchmarkResult {
//!     latency: Some(MetricValue::new(120.5, "ns")),
//!     throughput: Some(MetricValue::new(8300.0, "ops/s")),
//!     ..Default::default()
//! };
//!
//! let mut benchmarks = HashMap::new();
//! benchmarks.insert("my_benchmark".to_string(), result);
//!
//! let report = BenchmarkReport {
//!     metadata: Metadata::create("my-impl", "0.1.0"),
//!     benchmarks,
//! };
//!
//! let json = serde_json::to_string_pretty(&report).unwrap();
//! ```

mod platform;
mod schema;
mod statistics;

pub use platform::{get_cpu_vendor, Platform};
pub use schema::{BenchmarkReport, BenchmarkResult, Metadata, MetricValue, TestVectors};
pub use statistics::{calculate_confidence_interval, calculate_statistics};
