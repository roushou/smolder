//! Secure key storage using OS keychain
//!
//! Uses the `keyring` crate for cross-platform secret storage:
//! - macOS: Keychain Services
//! - Windows: Credential Manager
//! - Linux: Secret Service (GNOME Keyring, KWallet)

use crate::error::Error;

/// Service name for keyring entries
const KEYRING_SERVICE: &str = "smolder";

/// Store a private key securely in the OS keychain
pub fn store_private_key(wallet_name: &str, private_key: &str) -> Result<(), Error> {
    let key = format!("wallet:{}", wallet_name);
    let entry = keyring::Entry::new(KEYRING_SERVICE, &key)
        .map_err(|e| Error::Keyring(format!("Failed to create keyring entry: {}", e)))?;

    entry
        .set_password(private_key)
        .map_err(|e| Error::Keyring(format!("Failed to store key: {}", e)))?;

    Ok(())
}

/// Retrieve a private key from the OS keychain
pub fn get_private_key(wallet_name: &str) -> Result<String, Error> {
    let key = format!("wallet:{}", wallet_name);
    let entry = keyring::Entry::new(KEYRING_SERVICE, &key)
        .map_err(|e| Error::Keyring(format!("Failed to create keyring entry: {}", e)))?;

    entry
        .get_password()
        .map_err(|e| Error::Keyring(format!("Failed to retrieve key: {}", e)))
}

/// Delete a private key from the OS keychain
pub fn delete_private_key(wallet_name: &str) -> Result<(), Error> {
    let key = format!("wallet:{}", wallet_name);
    let entry = keyring::Entry::new(KEYRING_SERVICE, &key)
        .map_err(|e| Error::Keyring(format!("Failed to create keyring entry: {}", e)))?;

    entry
        .delete_credential()
        .map_err(|e| Error::Keyring(format!("Failed to delete key: {}", e)))?;

    Ok(())
}

/// Check if a private key exists in the OS keychain
pub fn has_private_key(wallet_name: &str) -> bool {
    get_private_key(wallet_name).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a working keyring backend
    // They may fail in CI environments without proper setup

    #[test]
    #[ignore = "Requires OS keyring backend"]
    fn test_store_and_retrieve_key() {
        let wallet_name = "test_wallet_smolder";
        let private_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        // Store
        store_private_key(wallet_name, private_key).unwrap();

        // Retrieve
        let retrieved = get_private_key(wallet_name).unwrap();
        assert_eq!(retrieved, private_key);

        // Cleanup
        delete_private_key(wallet_name).unwrap();
    }

    #[test]
    #[ignore = "Requires OS keyring backend"]
    fn test_delete_key() {
        let wallet_name = "test_wallet_delete_smolder";
        let private_key = "0xabcdef";

        store_private_key(wallet_name, private_key).unwrap();
        assert!(has_private_key(wallet_name));

        delete_private_key(wallet_name).unwrap();
        assert!(!has_private_key(wallet_name));
    }
}
