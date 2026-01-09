//! Bytecode handling utilities
//!
//! Provides type-safe bytecode operations including parsing, validation,
//! and hash computation.

use crate::error::{Error, Result};
use alloy::primitives::keccak256;

/// Represents compiled contract bytecode
#[derive(Debug, Clone)]
pub struct Bytecode {
    bytes: Vec<u8>,
}

impl Bytecode {
    /// Create bytecode from a hex string (with or without 0x prefix)
    pub fn from_hex(hex: &str) -> Result<Self> {
        let clean = hex.trim_start_matches("0x");
        if clean.is_empty() {
            return Ok(Self { bytes: Vec::new() });
        }
        let bytes = hex::decode(clean)?;
        Ok(Self { bytes })
    }

    /// Create bytecode from raw bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Compute the keccak256 hash of the bytecode
    pub fn hash(&self) -> String {
        if self.bytes.is_empty() {
            return String::new();
        }
        format!("{:x}", keccak256(&self.bytes))
    }

    /// Check if the bytecode is empty or invalid
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Get the bytecode length in bytes
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to hex string (with 0x prefix)
    pub fn to_hex(&self) -> String {
        if self.bytes.is_empty() {
            return "0x".to_string();
        }
        format!("0x{}", hex::encode(&self.bytes))
    }
}

/// Check if a hex string represents valid bytecode (non-empty and decodable)
pub fn is_valid_bytecode(hex: &str) -> bool {
    let clean = hex.trim_start_matches("0x");
    !clean.is_empty() && hex::decode(clean).is_ok()
}

/// Compute keccak256 hash of bytecode hex string
pub fn compute_bytecode_hash(hex: &str) -> Result<String> {
    let bytecode = Bytecode::from_hex(hex)?;
    Ok(bytecode.hash())
}

/// Parse a hex block number (e.g., "0x1a4" -> 420)
pub fn parse_hex_block_number(hex: &str) -> Result<i64> {
    let clean = hex.trim_start_matches("0x");
    i64::from_str_radix(clean, 16)
        .map_err(|_| Error::Validation(format!("Invalid hex block number: {}", hex)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_from_hex() {
        let bytecode = Bytecode::from_hex("0x6080604052").unwrap();
        assert!(!bytecode.is_empty());
        assert_eq!(bytecode.len(), 5);
    }

    #[test]
    fn test_bytecode_from_hex_no_prefix() {
        let bytecode = Bytecode::from_hex("6080604052").unwrap();
        assert!(!bytecode.is_empty());
        assert_eq!(bytecode.len(), 5);
    }

    #[test]
    fn test_bytecode_empty() {
        let bytecode = Bytecode::from_hex("").unwrap();
        assert!(bytecode.is_empty());
        assert_eq!(bytecode.hash(), "");
    }

    #[test]
    fn test_bytecode_hash() {
        let bytecode = Bytecode::from_hex("0x6080604052").unwrap();
        let hash = bytecode.hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_bytecode_to_hex() {
        let bytecode = Bytecode::from_hex("6080604052").unwrap();
        assert_eq!(bytecode.to_hex(), "0x6080604052");
    }

    #[test]
    fn test_is_valid_bytecode() {
        assert!(is_valid_bytecode("0x6080604052"));
        assert!(is_valid_bytecode("6080604052"));
        assert!(!is_valid_bytecode(""));
        assert!(!is_valid_bytecode("0x"));
        assert!(!is_valid_bytecode("not_hex"));
    }

    #[test]
    fn test_compute_bytecode_hash() {
        let hash = compute_bytecode_hash("0x6080604052").unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_parse_hex_block_number() {
        assert_eq!(parse_hex_block_number("0x1a4").unwrap(), 420);
        assert_eq!(parse_hex_block_number("1a4").unwrap(), 420);
        assert_eq!(parse_hex_block_number("0x0").unwrap(), 0);
        assert_eq!(parse_hex_block_number("0xff").unwrap(), 255);
    }

    #[test]
    fn test_parse_hex_block_number_invalid() {
        assert!(parse_hex_block_number("not_hex").is_err());
    }
}
