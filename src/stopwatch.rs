// Copyright 2026 zkbench-rust Authors
// SPDX-License-Identifier: Apache-2.0

//! Stopwatch for timing benchmark code regions.
//!
//! Mirrors the C++ `zkbench::Stopwatch` API so that callers porting bench
//! binaries from C++ to Rust have a one-to-one analogue. Wraps
//! `std::time::Instant`; `start`/`stop` accumulate, `pause`/`resume`
//! preserve the running total, `reset` clears it.

use std::time::{Duration, Instant};

/// A simple stopwatch for timing benchmarks. Accumulates across
/// start/stop pairs and supports pause/resume.
#[derive(Debug, Default)]
pub struct Stopwatch {
    accumulated: Duration,
    started_at: Option<Instant>,
}

impl Stopwatch {
    /// Creates a new, stopped stopwatch with zero accumulated time.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts the stopwatch. Has no effect if already running.
    pub fn start(&mut self) {
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }
    }

    /// Stops the stopwatch and accumulates elapsed time. Has no effect
    /// if already stopped.
    pub fn stop(&mut self) {
        if let Some(t0) = self.started_at.take() {
            self.accumulated += t0.elapsed();
        }
    }

    /// Pauses the stopwatch, preserving accumulated time. Equivalent
    /// to `stop` — separate name kept for parity with the C++ API.
    pub fn pause(&mut self) {
        self.stop();
    }

    /// Resumes a paused stopwatch. Equivalent to `start`.
    pub fn resume(&mut self) {
        self.start();
    }

    /// Resets the stopwatch to zero and stops it.
    pub fn reset(&mut self) {
        self.accumulated = Duration::ZERO;
        self.started_at = None;
    }

    /// Returns true if the stopwatch is currently running.
    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    /// Returns the total elapsed time. If the stopwatch is currently
    /// running, includes time since the last `start`/`resume`.
    pub fn elapsed(&self) -> Duration {
        match self.started_at {
            Some(t0) => self.accumulated + t0.elapsed(),
            None => self.accumulated,
        }
    }

    /// Returns the elapsed time in milliseconds (fractional).
    pub fn elapsed_millis(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1_000.0
    }

    /// Returns the elapsed time in seconds (fractional).
    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }
}

/// RAII helper: starts the stopwatch on construction and stops it on
/// drop. Useful for timing a single scope without manual stop calls.
///
/// # Example
///
/// ```
/// use zkbench::{ScopedStopwatch, Stopwatch};
///
/// let mut sw = Stopwatch::new();
/// {
///     let _scope = ScopedStopwatch::new(&mut sw);
///     // ... timed work ...
/// } // sw stops here
/// assert!(!sw.is_running());
/// ```
pub struct ScopedStopwatch<'a> {
    sw: &'a mut Stopwatch,
}

impl<'a> ScopedStopwatch<'a> {
    pub fn new(sw: &'a mut Stopwatch) -> Self {
        sw.start();
        Self { sw }
    }
}

impl Drop for ScopedStopwatch<'_> {
    fn drop(&mut self) {
        self.sw.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn new_is_zero_and_stopped() {
        let sw = Stopwatch::new();
        assert!(!sw.is_running());
        assert_eq!(sw.elapsed(), Duration::ZERO);
        assert_eq!(sw.elapsed_millis(), 0.0);
    }

    #[test]
    fn start_then_stop_accumulates() {
        let mut sw = Stopwatch::new();
        sw.start();
        thread::sleep(Duration::from_millis(2));
        sw.stop();
        assert!(!sw.is_running());
        assert!(sw.elapsed_millis() >= 1.0);
    }

    #[test]
    fn double_start_is_idempotent() {
        let mut sw = Stopwatch::new();
        sw.start();
        let first = sw.started_at;
        sw.start();
        assert_eq!(sw.started_at.unwrap(), first.unwrap());
    }

    #[test]
    fn pause_resume_accumulates_total() {
        let mut sw = Stopwatch::new();
        sw.start();
        thread::sleep(Duration::from_millis(2));
        sw.pause();
        let after_pause = sw.elapsed_millis();
        thread::sleep(Duration::from_millis(2));
        // Time spent paused does not count.
        assert!((sw.elapsed_millis() - after_pause).abs() < 0.5);
        sw.resume();
        thread::sleep(Duration::from_millis(2));
        sw.stop();
        assert!(sw.elapsed_millis() >= after_pause + 1.0);
    }

    #[test]
    fn reset_clears() {
        let mut sw = Stopwatch::new();
        sw.start();
        thread::sleep(Duration::from_millis(1));
        sw.stop();
        assert!(sw.elapsed_millis() > 0.0);
        sw.reset();
        assert_eq!(sw.elapsed(), Duration::ZERO);
        assert!(!sw.is_running());
    }

    #[test]
    fn scoped_stopwatch_starts_and_stops() {
        let mut sw = Stopwatch::new();
        {
            let _scope = ScopedStopwatch::new(&mut sw);
            thread::sleep(Duration::from_millis(1));
        }
        assert!(!sw.is_running());
        assert!(sw.elapsed_millis() >= 0.5);
    }

    #[test]
    fn elapsed_while_running_includes_live_segment() {
        let mut sw = Stopwatch::new();
        sw.start();
        thread::sleep(Duration::from_millis(1));
        // Reading without stopping should still report nonzero.
        assert!(sw.elapsed_millis() > 0.0);
        assert!(sw.is_running());
    }
}
