use brokkr_utils::logging::prelude::*;
use brokkr_utils::Settings;
use once_cell::sync::OnceCell;
use prefixed_api_key::PrefixedApiKeyController;
use rand::rngs::OsRng;
use sha2::Sha256;
use std::sync::Arc;
use prefixed_api_key::PrefixedApiKey;

static PAK_CONTROLLER: OnceCell<Arc<PrefixedApiKeyController<OsRng, Sha256>>> = OnceCell::new();

pub fn create_pak_controller(
    config: Option<&Settings>,
) -> Result<Arc<PrefixedApiKeyController<OsRng, Sha256>>, &'static str> {
    match (PAK_CONTROLLER.get(), config) {
        (Some(controller), _) => Ok(controller.clone()),
        (None, Some(cfg)) => {
            let controller = PAK_CONTROLLER.get_or_init(|| {
                info!("Initializing PAK_CONTROLLER for the first time");
                Arc::new(create_pak_controller_inner(cfg).expect("Failed to create PAK controller"))
            });
            Ok(controller.clone())
        }
        (None, None) => Err("PAK_CONTROLLER not initialized and no config provided"),
    }
}

fn create_pak_controller_inner(
    config: &Settings,
) -> Result<PrefixedApiKeyController<OsRng, Sha256>, Box<dyn std::error::Error>> {
    // This function remains unchanged
    let builder = PrefixedApiKeyController::configure()
        .prefix(config.pak.prefix.clone().unwrap())
        .short_token_length(config.pak.short_token_length.unwrap())
        .short_token_prefix(config.pak.short_token_prefix.clone())
        .long_token_length(config.pak.long_token_length.unwrap())
        .rng_osrng()
        .digest_sha256();

    builder.finalize().map_err(|e| e.into())
}

pub fn create_pak(config: &Settings) -> Result<(String, String), Box<dyn std::error::Error>> {
    let controller = create_pak_controller(Some(config))?;

    // Generate PAK and hash
    controller
        .try_generate_key_and_hash()
        .map(|(pak, hash)| (pak.to_string(), hash))
        .map_err(|e| e.into())
}

pub fn verify_pak(pak: String, stored_hash: String) -> bool {
    let pak = PrefixedApiKey::from_string(pak.as_str()).expect("Failed to parse PAK");
    let controller = create_pak_controller(None).expect("Failed to create PAK controller");
    let computed_hash = controller.long_token_hashed(&pak);
    stored_hash == computed_hash
}

pub fn generate_pak_hash(pak: String) -> String {
    let pak = PrefixedApiKey::from_string(pak.as_str()).expect("Failed to parse PAK");
    let controller = create_pak_controller(None).expect("Failed to create PAK controller");
    controller.long_token_hashed(&pak)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_pak_controller_singleton() {
        let result = create_pak_controller(None);
        assert!(
            result.is_err(),
            "Should fail when not initialized and no config provided"
        );

        let config = Settings::new(None).expect("Failed to load configuration");

        // First call should initialize the controller
        let controller1 =
            create_pak_controller(Some(&config)).expect("Failed to create controller");
        let address1 = Arc::as_ptr(&controller1) as usize;

        // Second call should return the same controller
        let controller2 = create_pak_controller(None).expect("Failed to get controller");
        let address2 = Arc::as_ptr(&controller2) as usize;

        // Check that both instances have the same memory address
        assert_eq!(
            address1, address2,
            "Controllers should have the same memory address"
        );

        // Test in multiple threads
        let threads: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(move || {
                    let controller =
                        create_pak_controller(None).expect("Failed to get controller in thread");
                    Arc::as_ptr(&controller) as usize
                })
            })
            .collect();

        let thread_addresses: Vec<_> = threads
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        // Check that all thread instances have the same memory address
        for thread_address in thread_addresses {
            assert_eq!(
                address1, thread_address,
                "Controller in thread should have the same memory address as the main thread"
            );
        }

        // Test PAK generation
        let (pak1, hash1) = create_pak(&config).unwrap();
        let (pak2, hash2) = create_pak(&config).unwrap();

        // PAKs should be different
        assert_ne!(pak1, pak2, "Generated PAKs should be different");
        assert_ne!(hash1, hash2, "Generated hashes should be different");
    }

    #[test]
    fn test_verify_pak() {
        let config = Settings::new(None).expect("Failed to load configuration");
        
        // Initialize the PAK controller
        create_pak_controller(Some(&config)).expect("Failed to create controller");

        // Generate a PAK and hash
        let (pak, hash) = create_pak(&config).unwrap();

        // Verify the PAK
        assert!(verify_pak(pak.clone(), hash.clone()), "PAK verification failed");

        // Test with an invalid PAK
        assert!(!verify_pak(pak.clone(), "0000000000000000000000000000000000000000000000000000000000000000".to_string()), "Invalid PAK should not verify");

        // Test thread safety
        let pak_clone = pak.clone();
        let hash_clone = hash.clone();
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let pak = pak_clone.clone();
                let hash = hash_clone.clone();
                std::thread::spawn(move || verify_pak(pak.clone(), hash.clone()))
            })
            .collect();

        for handle in handles {
            assert!(handle.join().unwrap(), "PAK verification failed in thread");
        }

        // Test consistency
        for _ in 0..100 {
            assert!(verify_pak(pak.clone(), hash.clone()), "PAK verification inconsistent");
        }
    }

    #[test]
    fn test_generate_pak_hash() {
        let config = Settings::new(None).expect("Failed to load configuration");
        
        // Initialize the PAK controller
        create_pak_controller(Some(&config)).expect("Failed to create controller");

        // Generate a PAK and hash
        let (pak, original_hash) = create_pak(&config).unwrap();

        // Generate hash from the PAK
        let generated_hash = generate_pak_hash(pak.clone());

        // Verify that the generated hash matches the original hash
        assert_eq!(original_hash, generated_hash, "Generated hash should match the original hash");

        // Test consistency
        for _ in 0..100 {
            assert_eq!(
                generated_hash,
                generate_pak_hash(pak.clone()),
                "Hash generation should be consistent"
            );
        }

        // Test thread safety
        let pak_clone = pak.clone();
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let pak = pak_clone.clone();
                std::thread::spawn(move || generate_pak_hash(pak))
            })
            .collect();

        for handle in handles {
            assert_eq!(
                generated_hash,
                handle.join().unwrap(),
                "Hash generation should be consistent across threads"
            );
        }

        // Test with different PAKs
        let (pak2, hash2) = create_pak(&config).unwrap();
        assert_ne!(
            generate_pak_hash(pak),
            generate_pak_hash(pak2),
            "Hashes for different PAKs should be different"
        );

    }
}
