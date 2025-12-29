---
id: migrate-existing-webhook-data-to
level: task
title: "Migrate existing webhook data to new encryption format"
short_code: "BROKKR-T-0053"
created_at: 2025-12-29T14:27:13.085386+00:00
updated_at: 2025-12-29T15:04:59.952526+00:00
parent: BROKKR-I-0005
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0005
---

# Migrate existing webhook data to new encryption format

## Description

Migrate existing webhook subscription data from XOR encryption to AES-256-GCM. This requires a careful migration process to avoid data loss.

**Depends on:** BROKKR-T-0052 (AES-256-GCM implementation)

## Files to Modify

- `crates/brokkr-models/migrations/` - Create migration for new columns
- `crates/brokkr-broker/src/` - Create migration script/CLI command

## Implementation

### Phase 1: Add new columns (migration)
```sql
-- up.sql
ALTER TABLE webhook_subscriptions 
    ADD COLUMN url_encrypted_v2 BYTEA,
    ADD COLUMN auth_header_encrypted_v2 BYTEA;
```

### Phase 2: Migration script (Rust CLI)
```rust
// New CLI command: brokkr-broker migrate-encryption
pub async fn migrate_webhook_encryption(dal: &Dal, old_key: &[u8], new_key: &[u8; 32]) {
    let old_encryption = XorEncryption::new(old_key);
    let new_encryption = AesGcmEncryption::new(new_key);
    
    let subscriptions = dal.webhook_subscriptions().list_all()?;
    
    for sub in subscriptions {
        // Decrypt with old method
        let url = old_encryption.decrypt(&sub.url_encrypted)?;
        let auth = sub.auth_header_encrypted
            .map(|h| old_encryption.decrypt(&h))
            .transpose()?;
        
        // Re-encrypt with new method
        let url_v2 = new_encryption.encrypt(&url)?;
        let auth_v2 = auth.map(|a| new_encryption.encrypt(&a)).transpose()?;
        
        // Update record
        dal.webhook_subscriptions().update_encryption(sub.id, url_v2, auth_v2)?;
    }
}
```

### Phase 3: Update DAL to use v2 columns
- Modify read methods to prefer v2 columns
- Modify write methods to only write v2 columns

### Phase 4: Drop old columns (future migration)
```sql
ALTER TABLE webhook_subscriptions 
    DROP COLUMN url_encrypted,
    DROP COLUMN auth_header_encrypted;
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration script successfully re-encrypts all webhook data
- [ ] Zero data loss during migration
- [ ] Rollback procedure documented
- [ ] v2 columns populated for all records
- [ ] DAL reads from v2, writes to v2
- [ ] Old columns can be safely dropped

## Critical Prerequisite: Key Configuration

**WARNING:** If `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` was NOT configured, the broker generated a random key on each startup. This means:
- Existing encrypted data is **unrecoverable** if service restarted without configured key
- Migration only works if the SAME key used to encrypt data is still available

**Before starting this task, verify:**
1. Is `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` configured in production?
2. Has the service restarted since webhooks were created?
3. If key is unknown, existing webhook URLs must be re-entered by users

## Deployment Sequence

```
1. Deploy code with BROKKR-T-0052 (AES-GCM support + legacy XOR read)
   - New writes use AES-GCM
   - Old reads still work via legacy decoder

2. Run database migration (add v2 columns)
   - Does NOT touch existing data

3. Run migration CLI command
   - brokkr-broker migrate-encryption --old-key <hex> --new-key <hex>
   - Re-encrypts all existing data
   - Writes to v2 columns

4. Update DAL to read from v2 columns (next release)

5. Drop v1 columns (future release, after verification)
```

## Handling Unknown Key Scenario

If the original encryption key is unknown:
```rust
// CLI flag to handle unrecoverable data
brokkr-broker migrate-encryption --mark-unrecoverable

// Sets url_encrypted_v2 to NULL for records that can't be decrypted
// Logs affected subscription IDs for manual remediation
```

## Dependencies

- **BROKKR-T-0052**: Requires AES-GCM implementation with legacy XOR support

## Rollback Procedure

1. Keep v1 columns until v2 data verified
2. If issues found: revert DAL to read v1 columns
3. Delete v2 column data
4. Re-attempt migration after fix