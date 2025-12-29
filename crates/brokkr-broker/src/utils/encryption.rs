/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Encryption utilities for protecting sensitive data at rest.
//!
//! This module provides encryption and decryption functionality for webhook URLs
//! and authentication headers stored in the database.
//!
//! # Security Note
//!
//! The current implementation uses XOR-based obfuscation with a key-derived mask.
//! This is NOT cryptographically secure and should be replaced with proper
//! AES-256-GCM encryption before production use.
//!
//! TODO: Replace with proper AES-256-GCM using the `aes-gcm` crate.

use brokkr_utils::logging::prelude::*;
use once_cell::sync::OnceCell;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Global encryption key storage.
static ENCRYPTION_KEY: OnceCell<Arc<EncryptionKey>> = OnceCell::new();

/// Encryption key wrapper with derived material.
#[derive(Debug)]
pub struct EncryptionKey {
    /// The raw 32-byte key.
    key: [u8; 32],
}

impl EncryptionKey {
    /// Creates a new encryption key from raw bytes.
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Creates a new random encryption key.
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self { key }
    }

    /// Creates a key from a hex-encoded string.
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(hex).map_err(|e| format!("Invalid hex encoding: {}", e))?;

        if bytes.len() != 32 {
            return Err(format!("Key must be 32 bytes, got {} bytes", bytes.len()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(Self { key })
    }

    /// Returns the key as a hex string (for logging key fingerprint only).
    pub fn fingerprint(&self) -> String {
        let hash = Sha256::digest(&self.key);
        hex::encode(&hash[..8])
    }

    /// Derives a mask for a given nonce using the key.
    fn derive_mask(&self, nonce: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.key);
        hasher.update(nonce);
        hasher.finalize().to_vec()
    }

    /// Encrypts data using XOR with a derived mask.
    ///
    /// # Format
    /// The output format is: `nonce (16 bytes) || ciphertext`
    ///
    /// # Security Warning
    /// This is NOT cryptographically secure. It's XOR-based obfuscation
    /// that should be replaced with AES-GCM before production use.
    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        // Generate random nonce
        let mut nonce = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut nonce);

        // Derive mask from key and nonce
        let mask = self.derive_mask(&nonce);

        // XOR plaintext with repeated mask
        let ciphertext: Vec<u8> = plaintext
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ mask[i % mask.len()])
            .collect();

        // Prepend nonce to ciphertext
        let mut output = Vec::with_capacity(16 + ciphertext.len());
        output.extend_from_slice(&nonce);
        output.extend(ciphertext);
        output
    }

    /// Decrypts data encrypted with encrypt().
    ///
    /// # Arguments
    /// * `ciphertext` - The encrypted data (nonce || ciphertext).
    ///
    /// # Returns
    /// The decrypted plaintext, or an error if decryption fails.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        if ciphertext.len() < 16 {
            return Err("Ciphertext too short (missing nonce)".to_string());
        }

        // Extract nonce and actual ciphertext
        let nonce = &ciphertext[..16];
        let encrypted = &ciphertext[16..];

        // Derive same mask
        let mask = self.derive_mask(nonce);

        // XOR to decrypt
        let plaintext: Vec<u8> = encrypted
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ mask[i % mask.len()])
            .collect();

        Ok(plaintext)
    }
}

/// Initializes the global encryption key from configuration.
///
/// This should be called once during broker startup.
///
/// # Arguments
/// * `key_hex` - Optional hex-encoded 32-byte key. If None, a random key is generated.
///
/// # Returns
/// Ok(()) if initialization succeeded, Err if already initialized or key is invalid.
pub fn init_encryption_key(key_hex: Option<&str>) -> Result<(), String> {
    let key = match key_hex {
        Some(hex) if !hex.is_empty() => {
            info!("Initializing encryption key from configuration");
            EncryptionKey::from_hex(hex)?
        }
        _ => {
            warn!(
                "No encryption key configured, generating random key. \
                 Configure BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY for production use."
            );
            EncryptionKey::generate()
        }
    };

    info!("Encryption key fingerprint: {}", key.fingerprint());

    ENCRYPTION_KEY
        .set(Arc::new(key))
        .map_err(|_| "Encryption key already initialized".to_string())
}

/// Gets the global encryption key.
///
/// # Panics
/// Panics if called before init_encryption_key().
pub fn get_encryption_key() -> Arc<EncryptionKey> {
    ENCRYPTION_KEY
        .get()
        .expect("Encryption key not initialized. Call init_encryption_key() first.")
        .clone()
}

/// Encrypts a string value for storage.
///
/// # Arguments
/// * `value` - The plaintext string to encrypt.
///
/// # Returns
/// The encrypted bytes.
pub fn encrypt_string(value: &str) -> Vec<u8> {
    get_encryption_key().encrypt(value.as_bytes())
}

/// Decrypts bytes back to a string.
///
/// # Arguments
/// * `encrypted` - The encrypted bytes.
///
/// # Returns
/// The decrypted string, or an error if decryption fails.
pub fn decrypt_string(encrypted: &[u8]) -> Result<String, String> {
    let bytes = get_encryption_key().decrypt(encrypted)?;
    String::from_utf8(bytes).map_err(|e| format!("Decrypted value is not valid UTF-8: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_key_from_hex() {
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key = EncryptionKey::from_hex(hex).unwrap();
        assert_eq!(key.key[0], 0x01);
        assert_eq!(key.key[31], 0xef);
    }

    #[test]
    fn test_encryption_key_from_hex_invalid() {
        let short = "0123456789abcdef";
        assert!(EncryptionKey::from_hex(short).is_err());

        let invalid = "xyz123";
        assert!(EncryptionKey::from_hex(invalid).is_err());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = EncryptionKey::generate();
        let plaintext = b"https://example.com/webhook?token=secret123";

        let encrypted = key.encrypt(plaintext);
        let decrypted = key.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let key = EncryptionKey::generate();
        let plaintext = b"";

        let encrypted = key.encrypt(plaintext);
        let decrypted = key.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_output() {
        let key = EncryptionKey::generate();
        let plaintext = b"test data";

        // Same plaintext should produce different ciphertext due to random nonce
        let encrypted1 = key.encrypt(plaintext);
        let encrypted2 = key.encrypt(plaintext);

        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same value
        assert_eq!(key.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(key.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = EncryptionKey::generate();
        let key2 = EncryptionKey::generate();
        let plaintext = b"secret message";

        let encrypted = key1.encrypt(plaintext);
        let decrypted = key2.decrypt(&encrypted).unwrap();

        // Wrong key produces garbage, not the original
        assert_ne!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_too_short() {
        let key = EncryptionKey::generate();
        let short = vec![0u8; 10]; // Less than 16 bytes

        assert!(key.decrypt(&short).is_err());
    }

    #[test]
    fn test_fingerprint() {
        let key = EncryptionKey::generate();
        let fingerprint = key.fingerprint();

        // Fingerprint should be 16 hex chars (8 bytes)
        assert_eq!(fingerprint.len(), 16);
        assert!(fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
