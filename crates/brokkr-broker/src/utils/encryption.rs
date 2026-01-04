/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Encryption utilities for protecting sensitive data at rest.
//!
//! This module provides AES-256-GCM encryption and decryption functionality for webhook URLs
//! and authentication headers stored in the database.
//!
//! # Format
//!
//! Encrypted data format: `version (1 byte) || nonce (12 bytes) || ciphertext || tag (16 bytes)`
//!
//! Version bytes:
//! - 0x00: Legacy XOR encryption (read-only, for migration)
//! - 0x01: AES-256-GCM encryption

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use tracing::{debug, error, info, warn};
use once_cell::sync::OnceCell;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Version byte for AES-256-GCM encrypted data
const VERSION_AES_GCM: u8 = 0x01;

/// Version byte for legacy XOR encrypted data (read-only)
const VERSION_LEGACY_XOR: u8 = 0x00;

/// Nonce size for AES-256-GCM (96 bits)
const AES_GCM_NONCE_SIZE: usize = 12;

/// Legacy XOR nonce size (128 bits)
const LEGACY_XOR_NONCE_SIZE: usize = 16;

/// Global encryption key storage.
static ENCRYPTION_KEY: OnceCell<Arc<EncryptionKey>> = OnceCell::new();

/// Encryption error types
#[derive(Debug)]
pub enum EncryptionError {
    /// Encryption operation failed
    EncryptionFailed,
    /// Decryption operation failed (wrong key or corrupted data)
    DecryptionFailed,
    /// Invalid data format
    InvalidData(String),
    /// Unsupported encryption version
    UnsupportedVersion(u8),
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::EncryptionFailed => write!(f, "Encryption failed"),
            EncryptionError::DecryptionFailed => write!(f, "Decryption failed"),
            EncryptionError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            EncryptionError::UnsupportedVersion(v) => write!(f, "Unsupported encryption version: {}", v),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// Encryption key wrapper with AES-256-GCM cipher.
pub struct EncryptionKey {
    /// The raw 32-byte key.
    key: [u8; 32],
    /// Pre-initialized AES-256-GCM cipher
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionKey")
            .field("fingerprint", &self.fingerprint())
            .finish()
    }
}

impl EncryptionKey {
    /// Creates a new encryption key from raw bytes.
    pub fn new(key: [u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key size");
        Self { key, cipher }
    }

    /// Creates a new random encryption key.
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self::new(key)
    }

    /// Creates a key from a hex-encoded string.
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(hex).map_err(|e| format!("Invalid hex encoding: {}", e))?;

        if bytes.len() != 32 {
            return Err(format!("Key must be 32 bytes, got {} bytes", bytes.len()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(Self::new(key))
    }

    /// Returns the key as a hex string (for logging key fingerprint only).
    pub fn fingerprint(&self) -> String {
        let hash = Sha256::digest(&self.key);
        hex::encode(&hash[..8])
    }

    /// Encrypts data using AES-256-GCM.
    ///
    /// # Format
    /// The output format is: `version (1 byte) || nonce (12 bytes) || ciphertext || tag (16 bytes)`
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; AES_GCM_NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt with AES-256-GCM
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| EncryptionError::EncryptionFailed)?;

        // Build output: version || nonce || ciphertext (includes auth tag)
        let mut output = Vec::with_capacity(1 + AES_GCM_NONCE_SIZE + ciphertext.len());
        output.push(VERSION_AES_GCM);
        output.extend_from_slice(&nonce_bytes);
        output.extend(ciphertext);
        Ok(output)
    }

    /// Decrypts data, automatically detecting the encryption version.
    ///
    /// Supports:
    /// - Version 0x01: AES-256-GCM
    /// - Version 0x00 or no version byte: Legacy XOR (for migration)
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.is_empty() {
            return Err(EncryptionError::InvalidData("Empty data".to_string()));
        }

        // Check version byte
        let version = data[0];

        match version {
            VERSION_AES_GCM => self.decrypt_aes_gcm(&data[1..]),
            VERSION_LEGACY_XOR => self.decrypt_legacy_xor(&data[1..]),
            _ => {
                // No version byte - assume legacy XOR format
                // Legacy format: nonce (16 bytes) || ciphertext
                if data.len() >= LEGACY_XOR_NONCE_SIZE {
                    self.decrypt_legacy_xor(data)
                } else {
                    Err(EncryptionError::InvalidData("Data too short".to_string()))
                }
            }
        }
    }

    /// Decrypts AES-256-GCM encrypted data.
    fn decrypt_aes_gcm(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.len() < AES_GCM_NONCE_SIZE {
            return Err(EncryptionError::InvalidData(
                "Ciphertext too short (missing nonce)".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(AES_GCM_NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| EncryptionError::DecryptionFailed)
    }

    /// Decrypts legacy XOR-encrypted data (for migration support).
    ///
    /// # Security Warning
    /// This method exists only for backward compatibility during migration.
    /// XOR encryption is NOT cryptographically secure.
    fn decrypt_legacy_xor(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.len() < LEGACY_XOR_NONCE_SIZE {
            return Err(EncryptionError::InvalidData(
                "Legacy ciphertext too short (missing nonce)".to_string(),
            ));
        }

        // Extract nonce and actual ciphertext
        let nonce = &data[..LEGACY_XOR_NONCE_SIZE];
        let encrypted = &data[LEGACY_XOR_NONCE_SIZE..];

        // Derive same mask using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&self.key);
        hasher.update(nonce);
        let mask = hasher.finalize();

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
/// The encrypted bytes, or an error if encryption fails.
pub fn encrypt_string(value: &str) -> Result<Vec<u8>, EncryptionError> {
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
    let bytes = get_encryption_key()
        .decrypt(encrypted)
        .map_err(|e| e.to_string())?;
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

        let encrypted = key.encrypt(plaintext).unwrap();
        let decrypted = key.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let key = EncryptionKey::generate();
        let plaintext = b"";

        let encrypted = key.encrypt(plaintext).unwrap();
        let decrypted = key.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_output() {
        let key = EncryptionKey::generate();
        let plaintext = b"test data";

        // Same plaintext should produce different ciphertext due to random nonce
        let encrypted1 = key.encrypt(plaintext).unwrap();
        let encrypted2 = key.encrypt(plaintext).unwrap();

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

        let encrypted = key1.encrypt(plaintext).unwrap();

        // Wrong key should fail decryption (AES-GCM has authentication)
        assert!(key2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_decrypt_tampered_data() {
        let key = EncryptionKey::generate();
        let plaintext = b"secret message";

        let mut encrypted = key.encrypt(plaintext).unwrap();

        // Tamper with the ciphertext
        if let Some(byte) = encrypted.last_mut() {
            *byte ^= 0xFF;
        }

        // Tampered data should fail authentication
        assert!(key.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_decrypt_too_short() {
        let key = EncryptionKey::generate();
        let short = vec![VERSION_AES_GCM, 0u8, 1, 2]; // Too short

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

    #[test]
    fn test_version_byte_present() {
        let key = EncryptionKey::generate();
        let plaintext = b"test";

        let encrypted = key.encrypt(plaintext).unwrap();

        // First byte should be version
        assert_eq!(encrypted[0], VERSION_AES_GCM);
    }

    #[test]
    fn test_legacy_xor_decryption() {
        // Test that we can decrypt legacy XOR format
        let key = EncryptionKey::generate();
        let plaintext = b"legacy data";

        // Manually create legacy XOR encrypted data
        let mut nonce = [0u8; LEGACY_XOR_NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce);

        let mut hasher = Sha256::new();
        hasher.update(&key.key);
        hasher.update(&nonce);
        let mask = hasher.finalize();

        let ciphertext: Vec<u8> = plaintext
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ mask[i % mask.len()])
            .collect();

        // Legacy format without version byte
        let mut legacy_encrypted = nonce.to_vec();
        legacy_encrypted.extend(ciphertext);

        // Should be able to decrypt
        let decrypted = key.decrypt(&legacy_encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
