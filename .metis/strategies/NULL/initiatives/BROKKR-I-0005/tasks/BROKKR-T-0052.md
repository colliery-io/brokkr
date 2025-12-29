---
id: implement-aes-256-gcm-encryption
level: task
title: "Implement AES-256-GCM encryption to replace XOR obfuscation"
short_code: "BROKKR-T-0052"
created_at: 2025-12-29T14:27:13.001454+00:00
updated_at: 2025-12-29T14:59:54.481296+00:00
parent: BROKKR-I-0005
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0005
---

# Implement AES-256-GCM encryption to replace XOR obfuscation

## Description

Replace the insecure XOR-based encryption with proper AES-256-GCM authenticated encryption. This protects webhook URLs and auth headers stored in the database.

## Files to Modify

- `crates/brokkr-broker/Cargo.toml` - Add aes-gcm dependency
- `crates/brokkr-broker/src/utils/encryption.rs` - Rewrite implementation

## Implementation

Add dependency:
```toml
aes-gcm = "0.10"
```

New encryption implementation:
```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};

const NONCE_SIZE: usize = 12;

pub struct AesGcmEncryption {
    cipher: Aes256Gcm,
}

impl AesGcmEncryption {
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: Aes256Gcm::new_from_slice(key).expect("valid key size"),
        }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| EncryptionError::EncryptionFailed)?;
        
        // Return: nonce || ciphertext (includes auth tag)
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }
    
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.len() < NONCE_SIZE {
            return Err(EncryptionError::InvalidData);
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| EncryptionError::DecryptionFailed)
    }
}
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] AES-256-GCM encryption implemented
- [ ] All existing encryption tests pass (update as needed)
- [ ] Add tests for: empty data, wrong key, tampered ciphertext
- [ ] Nonce is randomly generated for each encryption
- [ ] Backward compatibility mode for reading old XOR data (temporary)

## Existing Key Management

Key is already managed via config at `crates/brokkr-broker/src/utils/encryption.rs:145-165`:
- Configured via: `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` (hex-encoded 32 bytes)
- Falls back to random generation if not set (WARNING logged)
- Same key config will be used for AES-GCM

## Architecture Approach

Keep the existing `EncryptionKey` struct interface but replace internals:

```rust
impl EncryptionKey {
    // Keep existing: new(), generate(), from_hex(), fingerprint()
    
    // Replace implementation:
    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> { /* AES-GCM */ }
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, String> { /* AES-GCM */ }
    
    // Add for migration (temporary):
    pub fn decrypt_legacy_xor(&self, ciphertext: &[u8]) -> Result<Vec<u8>, String> { /* old XOR */ }
}
```

## Format Change

| Version | Format |
|---------|--------|
| Old (XOR) | `nonce (16 bytes) \|\| ciphertext` |
| New (AES-GCM) | `version (1 byte) \|\| nonce (12 bytes) \|\| ciphertext \|\| tag (16 bytes)` |

Add version byte prefix to distinguish formats during migration.

## Dependencies

- None (foundational change)
- **Required by:** BROKKR-T-0053 (data migration)

## Notes

- Keep XOR decryption code temporarily for migration phase
- New data always encrypted with AES-GCM
- Version byte allows auto-detection of encryption format