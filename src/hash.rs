// Copyright 2026 zkbench-rust Authors
// SPDX-License-Identifier: Apache-2.0

use sha2::{Digest, Sha256};

/// Computes the SHA-256 hash of raw bytes.
///
/// Returns a 64-character lowercase hex string.
pub fn compute_hash(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    format!("{:x}", digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        assert_eq!(
            compute_hash(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn abc() {
        assert_eq!(
            compute_hash(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn uint32_array_le() {
        let data: Vec<u8> = [1u32, 2u32, 3u32]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        assert_eq!(
            compute_hash(&data),
            "4636993d3e1da4e9d6b8f87b79e8f7c6d018580d52661950eabc3845c5897a4d"
        );
    }

    #[test]
    fn same_data_same_hash() {
        let a: Vec<u8> = [10u32, 20u32, 30u32]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let b: Vec<u8> = [10u32, 20u32, 30u32]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        assert_eq!(compute_hash(&a), compute_hash(&b));
    }

    #[test]
    fn different_data_different_hash() {
        let a: Vec<u8> = [1u32, 2u32, 3u32]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let b: Vec<u8> = [1u32, 2u32, 4u32]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        assert_ne!(compute_hash(&a), compute_hash(&b));
    }
}
