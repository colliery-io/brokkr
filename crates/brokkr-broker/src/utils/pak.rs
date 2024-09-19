use std::sync::Arc;
use brokkr_utils::Settings;
use prefixed_api_key::PrefixedApiKeyController;
use rand::rngs::OsRng;
use sha2::Sha256;
use once_cell::sync::OnceCell;
use brokkr_utils::logging::prelude::*;

static PAK_CONTROLLER: OnceCell<Arc<PrefixedApiKeyController<OsRng, Sha256>>> = OnceCell::new();

pub fn create_pak_controller(config: Option<&Settings>) -> Result<Arc<PrefixedApiKeyController<OsRng, Sha256>>, &'static str> {
    match (PAK_CONTROLLER.get(), config) {
        (Some(controller), _) => Ok(controller.clone()),
        (None, Some(cfg)) => {
            let controller = PAK_CONTROLLER.get_or_init(|| {
                info!("Initializing PAK_CONTROLLER for the first time");
                Arc::new(create_pak_controller_inner(cfg).expect("Failed to create PAK controller"))
            });
            Ok(controller.clone())
        },
        (None, None) => Err("PAK_CONTROLLER not initialized and no config provided"),
    }
}

fn create_pak_controller_inner(config: &Settings) -> Result<PrefixedApiKeyController<OsRng, Sha256>, Box<dyn std::error::Error>> {
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
    controller.try_generate_key_and_hash()
        .map(|(pak, hash)| (pak.to_string(), hash))
        .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_pak_controller_singleton() {
        let config = Settings::new(None).expect("Failed to load configuration");

        // First call should initialize the controller
        let controller1 = create_pak_controller(Some(&config)).expect("Failed to create controller");
        let address1 = Arc::as_ptr(&controller1) as usize;

        // Second call should return the same controller
        let controller2 = create_pak_controller(None).expect("Failed to get controller");
        let address2 = Arc::as_ptr(&controller2) as usize;

        // Check that both instances have the same memory address
        assert_eq!(address1, address2, "Controllers should have the same memory address");

        // Test in multiple threads
        let threads: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(move || {
                    let controller = create_pak_controller(None).expect("Failed to get controller in thread");
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
    fn test_pak_controller_uninitialized() {
        // This should fail because the controller is not initialized
        let result = create_pak_controller(None);
        assert!(result.is_err(), "Should fail when not initialized and no config provided");
    }
}