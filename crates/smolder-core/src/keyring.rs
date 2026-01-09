//! Secure key storage using AES-256-GCM encryption
//!
//! Private keys are encrypted with an app-derived key before storage in SQLite.
//! This provides obfuscation rather than true security - the encryption key
//! is embedded in the binary. For higher security, consider password-based
//! key derivation.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;

use crate::error::Error;

/// App-derived encryption key (32 bytes for AES-256)
/// In production, this should ideally be derived from user input
const APP_KEY: &[u8; 32] = b"smolder-wallet-encrypt-key-0032!";

/// Nonce size for AES-GCM (96 bits / 12 bytes)
const NONCE_SIZE: usize = 12;

/// Encrypt a private key for storage
///
/// Returns the encrypted data with the nonce prepended (nonce || ciphertext)
pub fn encrypt_private_key(private_key: &str) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new(APP_KEY.into());

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, private_key.as_bytes())
        .map_err(|e| Error::Keyring(format!("Encryption failed: {}", e)))?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);

    Ok(result)
}

/// Decrypt a private key from storage
///
/// Expects data in format: nonce (12 bytes) || ciphertext
pub fn decrypt_private_key(encrypted_data: &[u8]) -> Result<String, Error> {
    if encrypted_data.len() < NONCE_SIZE {
        return Err(Error::Keyring("Invalid encrypted data: too short".into()));
    }

    let cipher = Aes256Gcm::new(APP_KEY.into());

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| Error::Keyring(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext).map_err(|e| Error::Keyring(format!("Invalid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let private_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let encrypted = encrypt_private_key(private_key).unwrap();
        let decrypted = decrypt_private_key(&encrypted).unwrap();

        assert_eq!(decrypted, private_key);
    }

    #[test]
    fn test_different_nonces() {
        let private_key = "0xabcdef";

        let encrypted1 = encrypt_private_key(private_key).unwrap();
        let encrypted2 = encrypt_private_key(private_key).unwrap();

        // Same plaintext should produce different ciphertext due to random nonce
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt correctly
        assert_eq!(decrypt_private_key(&encrypted1).unwrap(), private_key);
        assert_eq!(decrypt_private_key(&encrypted2).unwrap(), private_key);
    }

    #[test]
    fn test_decrypt_invalid_data() {
        // Too short
        assert!(decrypt_private_key(&[0u8; 5]).is_err());

        // Invalid ciphertext
        assert!(decrypt_private_key(&[0u8; 20]).is_err());
    }
}
