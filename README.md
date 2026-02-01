<!-- Copyright 2026 zkbench-rust Authors -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# zkbench

Reusable benchmarking library for zero-knowledge proof implementations.

## Overview

zkbench provides common types and utilities for benchmark reporting across
different ZK implementations with a standardized JSON schema. It's designed to
facilitate consistent benchmark data collection and comparison across various
ZK proof systems.

## Features

- **Standardized Schema**: JSON schema for benchmark reports
- **Metric Values**: Support for values with optional confidence bounds
- **Platform Detection**: Automatic detection of OS, architecture, CPU info
- **Statistical Utilities**: Mean, standard deviation, and confidence interval
  calculations
- **Test Vector Support**: Input/output hash verification for reproducibility
- **Auto Metadata**: Git commit SHA and timestamp auto-detection

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zkbench = "0.1.0"
```

## Usage

### Basic Example

```rust
use zkbench::{BenchmarkReport, BenchmarkResult, Metadata, MetricValue};
use std::collections::HashMap;

// Create a benchmark result
let result = BenchmarkResult {
    latency: Some(MetricValue::new(120.5, "ns")),
    throughput: Some(MetricValue::new(8300.0, "ops/s")),
    ..Default::default()
};

// Build the report
let mut benchmarks = HashMap::new();
benchmarks.insert("my_benchmark".to_string(), result);

let report = BenchmarkReport {
    metadata: Metadata::create("my-impl", "0.1.0"),
    benchmarks,
};

// Serialize to JSON
let json = serde_json::to_string_pretty(&report).unwrap();
println!("{}", json);
```

### MetricValue with Confidence Bounds

```rust
use zkbench::MetricValue;

// Simple value
let latency = MetricValue::new(100.0, "ns");

// Value with confidence bounds
let latency_with_ci = MetricValue::with_bounds(100.0, "ns", 95.0, 105.0);
```

### Statistical Calculations

```rust
use zkbench::{calculate_statistics, calculate_confidence_interval};

let measurements = vec![98.0, 102.0, 99.0, 101.0, 100.0];

// Calculate mean and standard deviation
let (mean, stdev) = calculate_statistics(&measurements);

// Calculate 95% confidence interval
let (lower, upper) = calculate_confidence_interval(mean, stdev, 0.95);
```

### Test Vectors

```rust
use zkbench::{BenchmarkResult, TestVectors};

let result = BenchmarkResult {
    test_vectors: Some(TestVectors {
        input_hash: "sha256:abc123...".to_string(),
        output_hash: "sha256:def456...".to_string(),
        verified: true,
    }),
    ..Default::default()
};
```

## JSON Schema

The output follows a standardized schema:

```json
{
  "metadata": {
    "implementation": "my-impl",
    "version": "0.1.0",
    "commit_sha": "abc123def456",
    "timestamp": "2026-01-30T12:00:00Z",
    "platform": {
      "os": "linux",
      "arch": "x86_64",
      "cpu_count": 16,
      "cpu_vendor": "AMD Ryzen 9 5950X"
    }
  },
  "benchmarks": {
    "benchmark_name": {
      "latency": {
        "value": 120.5,
        "unit": "ns",
        "lower_value": 115.0,
        "upper_value": 126.0
      },
      "throughput": {
        "value": 8300.0,
        "unit": "ops/s"
      },
      "iterations": 1000
    }
  }
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
