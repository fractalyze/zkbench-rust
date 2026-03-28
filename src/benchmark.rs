// Copyright 2026 zkbench-rust Authors
// SPDX-License-Identifier: Apache-2.0

//! Template-based benchmark harness for Rust benchmarks.
//!
//! Mirrors the `JaxBenchmark` pattern from zkbench-py: subclasses override
//! `get_config()` and `get_ops()`, while [`run`] handles CLI parsing,
//! warmup, timing, statistics, and JSON output.
//!
//! # Example
//!
//! ```ignore
//! use zkbench::benchmark::{BenchmarkConfig, BenchmarkOp, RustBenchmark, run};
//!
//! struct MyBench;
//!
//! impl RustBenchmark for MyBench {
//!     fn get_config(&self) -> BenchmarkConfig {
//!         BenchmarkConfig {
//!             implementation: "my-impl",
//!             version: "0.1.0",
//!             default_iterations: 50,
//!             default_warmup: 10,
//!         }
//!     }
//!
//!     fn get_ops(&mut self, sizes: &[usize]) -> Vec<BenchmarkOp> {
//!         vec![BenchmarkOp::new("my_op", || { /* work */ })]
//!     }
//! }
//!
//! fn main() { std::process::exit(run(&mut MyBench)); }
//! ```

use std::collections::HashMap;
use std::time::Instant;

use serde_json::Value;

use crate::schema::{BenchmarkReport, BenchmarkResult, Metadata, MetricValue, TestVectors};
use crate::statistics::{calculate_confidence_interval, calculate_statistics};

/// Configuration for a benchmark suite.
pub struct BenchmarkConfig {
    pub implementation: &'static str,
    pub version: &'static str,
    pub default_iterations: usize,
    pub default_warmup: usize,
}

/// Parsed CLI arguments (standard + custom).
pub struct BenchArgs {
    pub iterations: usize,
    pub warmup: usize,
    pub output: Option<String>,
    pub sizes: Vec<usize>,
}

/// A single benchmarkable operation.
///
/// Only `name` and `fn_` are required; all other fields are optional.
pub struct BenchmarkOp {
    pub name: String,
    /// The timed operation.
    pub fn_: Box<dyn FnMut()>,
    /// Per-iteration setup (e.g., re-upload data). Called before each timed run.
    pub setup: Option<Box<dyn FnMut()>>,
    /// Post-operation sync (e.g., `cuda_device_synchronize`). Called before
    /// and after the timed region.
    pub sync: Option<Box<dyn FnMut()>>,
    /// Arbitrary metadata (field, degree, etc.).
    pub metadata: HashMap<String, Value>,
    /// Pre-computed input hash for test vector verification.
    pub input_hash: String,
    /// Callable that returns the output hash after timing completes.
    pub output_hash_fn: Option<Box<dyn FnMut() -> String>>,
    /// Callable that verifies output correctness.
    pub verify_fn: Option<Box<dyn FnMut() -> bool>>,
}

impl BenchmarkOp {
    /// Create a minimal op with just a name and function.
    pub fn new(name: impl Into<String>, fn_: impl FnMut() + 'static) -> Self {
        Self {
            name: name.into(),
            fn_: Box::new(fn_),
            setup: None,
            sync: None,
            metadata: HashMap::new(),
            input_hash: String::new(),
            output_hash_fn: None,
            verify_fn: None,
        }
    }

    /// Builder: set metadata.
    pub fn with_metadata(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.metadata.insert(key.to_string(), value.into());
        self
    }

    /// Builder: set setup function.
    pub fn with_setup(mut self, setup: impl FnMut() + 'static) -> Self {
        self.setup = Some(Box::new(setup));
        self
    }

    /// Builder: set sync function.
    pub fn with_sync(mut self, sync: impl FnMut() + 'static) -> Self {
        self.sync = Some(Box::new(sync));
        self
    }

    /// Builder: set input hash.
    pub fn with_input_hash(mut self, hash: impl Into<String>) -> Self {
        self.input_hash = hash.into();
        self
    }

    /// Builder: set output hash function.
    pub fn with_output_hash_fn(mut self, f: impl FnMut() -> String + 'static) -> Self {
        self.output_hash_fn = Some(Box::new(f));
        self
    }

    /// Builder: set verify function.
    pub fn with_verify_fn(mut self, f: impl FnMut() -> bool + 'static) -> Self {
        self.verify_fn = Some(Box::new(f));
        self
    }
}

/// Template trait for Rust benchmarks.
///
/// Implement [`get_config`](RustBenchmark::get_config) and
/// [`get_ops`](RustBenchmark::get_ops); call [`run`] to execute.
pub trait RustBenchmark {
    /// Return benchmark configuration (implementation name, defaults).
    fn get_config(&self) -> BenchmarkConfig;

    /// Return the list of operations to benchmark at the given sizes.
    fn get_ops(&mut self, sizes: &[usize]) -> Vec<BenchmarkOp>;
}

/// Compute median of a f64 slice.
pub fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    }
}

/// Run a single operation with warmup + measured iterations.
fn run_single_op(op: &mut BenchmarkOp, iterations: usize, warmup: usize) -> BenchmarkResult {
    let call_setup = |op: &mut BenchmarkOp| {
        if let Some(setup) = &mut op.setup {
            setup();
        }
    };
    let call_sync = |op: &mut BenchmarkOp| {
        if let Some(sync) = &mut op.sync {
            sync();
        }
    };

    // Warmup
    for _ in 0..warmup {
        call_setup(op);
        call_sync(op);
        (op.fn_)();
        call_sync(op);
    }

    // Measured iterations
    let mut times_ns = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        call_setup(op);
        call_sync(op);
        let start = Instant::now();
        (op.fn_)();
        call_sync(op);
        times_ns.push(start.elapsed().as_nanos() as f64);
    }

    // Statistics (convert to μs for reporting)
    let times_us: Vec<f64> = times_ns.iter().map(|t| t / 1000.0).collect();
    let (mean, stdev) = calculate_statistics(&times_us);
    let med = median(&times_us);
    let (lower, upper) = calculate_confidence_interval(mean, stdev, iterations, 0.95);

    // Test vectors
    let test_vectors = if op.output_hash_fn.is_some() || op.verify_fn.is_some() {
        let output_hash = op.output_hash_fn.as_mut().map_or(String::new(), |f| f());
        let verified = op.verify_fn.as_mut().map_or(true, |f| f());
        Some(TestVectors {
            input_hash: op.input_hash.clone(),
            output_hash,
            verified,
        })
    } else {
        None
    };

    BenchmarkResult {
        latency: Some(MetricValue::with_bounds(med, "us", lower, upper)),
        iterations,
        test_vectors,
        metadata: op.metadata.clone(),
        ..Default::default()
    }
}

/// Parse standard CLI arguments: --iterations, --warmup, --output, --sizes.
fn parse_args(config: &BenchmarkConfig) -> BenchArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut iterations = config.default_iterations;
    let mut warmup = config.default_warmup;
    let mut output = None;
    let mut sizes = vec![16, 18, 20, 22];

    let mut i = 1;
    while i < args.len() {
        if let Some(val) = args[i].strip_prefix("--iterations=") {
            iterations = match val.parse() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("Invalid --iterations value: {val}");
                    std::process::exit(1);
                }
            };
        } else if let Some(val) = args[i].strip_prefix("--warmup=") {
            warmup = match val.parse() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("Invalid --warmup value: {val}");
                    std::process::exit(1);
                }
            };
        } else if let Some(val) = args[i].strip_prefix("--output=") {
            output = Some(val.to_string());
        } else if let Some(val) = args[i].strip_prefix("--sizes=") {
            sizes = match val.split(',').map(|s| s.trim().parse()).collect() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("Invalid --sizes value: {val}");
                    std::process::exit(1);
                }
            };
        }
        i += 1;
    }

    BenchArgs {
        iterations,
        warmup,
        output,
        sizes,
    }
}

/// Template function — orchestrates the entire benchmark workflow.
///
/// 1. Parse CLI arguments
/// 2. Call `get_ops()` to get operations
/// 3. For each op: warmup → measure → stats → verify
/// 4. Assemble `BenchmarkReport`
/// 5. Write JSON to stdout (or `--output` file)
///
/// Returns 0 on success, 1 on verification failure.
pub fn run(bench: &mut dyn RustBenchmark) -> i32 {
    let config = bench.get_config();
    let args = parse_args(&config);

    eprintln!(
        "{} v{} — {} warmup + {} measured iterations",
        config.implementation, config.version, args.warmup, args.iterations,
    );

    let mut ops = bench.get_ops(&args.sizes);
    let mut report = BenchmarkReport {
        metadata: Metadata::create(config.implementation, config.version),
        benchmarks: HashMap::new(),
    };

    let mut all_verified = true;
    for op in &mut ops {
        eprintln!("  {}...", op.name);
        let result = run_single_op(op, args.iterations, args.warmup);

        if let Some(tv) = &result.test_vectors {
            if !tv.verified {
                eprintln!("  VERIFICATION FAILED: {}", op.name);
                all_verified = false;
            }
        }

        if let Some(lat) = &result.latency {
            eprintln!("    median: {:.1} us", lat.value);
        }

        report.benchmarks.insert(op.name.clone(), result);
    }

    // Output JSON
    let json = serde_json::to_string_pretty(&report).unwrap();
    match &args.output {
        Some(path) => {
            if let Err(e) = std::fs::write(path, &json) {
                eprintln!("Error writing {path}: {e}");
                return 1;
            }
            eprintln!("Wrote {}", path);
        }
        None => {
            println!("{json}");
        }
    }

    if all_verified {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyBench;

    impl RustBenchmark for DummyBench {
        fn get_config(&self) -> BenchmarkConfig {
            BenchmarkConfig {
                implementation: "test",
                version: "0.0.1",
                default_iterations: 5,
                default_warmup: 1,
            }
        }

        fn get_ops(&mut self, _sizes: &[usize]) -> Vec<BenchmarkOp> {
            vec![BenchmarkOp::new("noop", || {})]
        }
    }

    #[test]
    fn test_median_odd() {
        assert!((median(&[3.0, 1.0, 2.0]) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_median_even() {
        assert!((median(&[1.0, 2.0, 3.0, 4.0]) - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_run_single_op() {
        let mut op = BenchmarkOp::new("test_op", || {
            std::hint::black_box(1 + 1);
        });
        let result = run_single_op(&mut op, 3, 1);
        assert!(result.latency.is_some());
        assert_eq!(result.iterations, 3);
    }
}
