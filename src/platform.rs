// Copyright 2026 zkbench-rust Authors
// SPDX-License-Identifier: Apache-2.0

//! Platform detection utilities.

use serde::{Deserialize, Serialize};

/// Platform information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub os: String,
    pub arch: String,
    pub cpu_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_vendor: Option<String>,
}

impl Platform {
    /// Creates Platform with auto-detected values.
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_count: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(1),
            cpu_vendor: get_cpu_vendor(),
        }
    }
}

/// Detects CPU vendor/model string.
///
/// Returns CPU vendor information from:
/// - Linux: /proc/cpuinfo
/// - macOS: sysctl -n machdep.cpu.brand_string
/// - Windows: PROCESSOR_IDENTIFIER environment variable
pub fn get_cpu_vendor() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        get_cpu_vendor_linux()
    }
    #[cfg(target_os = "macos")]
    {
        get_cpu_vendor_macos()
    }
    #[cfg(target_os = "windows")]
    {
        get_cpu_vendor_windows()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

#[cfg(target_os = "linux")]
fn get_cpu_vendor_linux() -> Option<String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open("/proc/cpuinfo").ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.ok()?;
        if line.starts_with("model name") {
            if let Some(pos) = line.find(':') {
                return Some(line[pos + 1..].trim().to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn get_cpu_vendor_macos() -> Option<String> {
    use std::process::Command;

    Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

#[cfg(target_os = "windows")]
fn get_cpu_vendor_windows() -> Option<String> {
    std::env::var("PROCESSOR_IDENTIFIER").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();

        // os should be one of the known operating systems
        assert!(!platform.os.is_empty());
        assert!(["linux", "macos", "windows", "freebsd", "openbsd", "netbsd"]
            .iter()
            .any(|&os| platform.os == os || platform.os.contains(os)));

        // arch should be one of the known architectures
        assert!(!platform.arch.is_empty());

        // cpu_count should be at least 1
        assert!(platform.cpu_count >= 1);
    }

    #[test]
    fn test_platform_serialization() {
        let platform = Platform::current();
        let json = serde_json::to_string(&platform).unwrap();

        assert!(json.contains("os"));
        assert!(json.contains("arch"));
        assert!(json.contains("cpu_count"));
    }

    #[test]
    fn test_platform_deserialization() {
        let json = r#"{"os": "linux", "arch": "x86_64", "cpu_count": 8}"#;
        let platform: Platform = serde_json::from_str(json).unwrap();

        assert_eq!(platform.os, "linux");
        assert_eq!(platform.arch, "x86_64");
        assert_eq!(platform.cpu_count, 8);
        assert!(platform.cpu_vendor.is_none());
    }

    #[test]
    fn test_platform_deserialization_with_cpu_vendor() {
        let json =
            r#"{"os": "linux", "arch": "x86_64", "cpu_count": 8, "cpu_vendor": "Intel Core i7"}"#;
        let platform: Platform = serde_json::from_str(json).unwrap();

        assert_eq!(platform.cpu_vendor, Some("Intel Core i7".to_string()));
    }

    #[test]
    fn test_platform_serialization_skips_none_cpu_vendor() {
        let platform = Platform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            cpu_count: 4,
            cpu_vendor: None,
        };
        let json = serde_json::to_string(&platform).unwrap();

        // cpu_vendor should be skipped when None
        assert!(!json.contains("cpu_vendor"));
    }

    #[test]
    fn test_platform_roundtrip() {
        let platform = Platform::current();
        let json = serde_json::to_string(&platform).unwrap();
        let deserialized: Platform = serde_json::from_str(&json).unwrap();

        assert_eq!(platform.os, deserialized.os);
        assert_eq!(platform.arch, deserialized.arch);
        assert_eq!(platform.cpu_count, deserialized.cpu_count);
        assert_eq!(platform.cpu_vendor, deserialized.cpu_vendor);
    }

    #[test]
    fn test_get_cpu_vendor() {
        // This test just ensures the function doesn't panic
        // The result depends on the platform
        let _vendor = get_cpu_vendor();
    }
}
