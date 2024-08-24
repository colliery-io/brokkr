use brokkr_utils::Settings;
use std::env;
use std::fs;
use tempfile::tempdir;

#[test]
/// Tests the loading of settings from both a file and environment variables.
///
/// This test:
/// 1. Creates a temporary TOML configuration file with specific settings.
/// 2. Sets an environment variable to override one of the settings.
/// 3. Loads the settings using the Settings::new() method.
/// 4. Verifies that settings are correctly loaded from the file.
/// 5. Checks that the environment variable successfully overrides the file setting.
/// 6. Cleans up the temporary resources after the test.
fn test_settings_from_file_and_env() {
    // Create a temporary directory for our test file
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_config.toml");

    // Write test configuration to a temporary file
    let test_config = r#"
        [database]
        url = "postgres://user:pass@testhost:5432/testdb"

        [log]
        level = "info"
    "#;
    fs::write(&file_path, test_config).expect("Failed to write test config file");

    // Set an environment variable to override a setting
    env::set_var("BROKKR__LOG__LEVEL", "debug");

    // Load settings from the test file
    let settings = Settings::new(Some(file_path.to_str().unwrap().to_string()))
        .expect("Failed to load settings");

    // Assert that settings are loaded correctly from the file
    assert_eq!(
        settings.database.url,
        "postgres://user:pass@testhost:5432/testdb",
        "Database URL should match the one specified in the test config file"
    );

    // Assert that the environment variable override worked
    assert_eq!(
        settings.log.level, 
        "debug",
        "Log level should be overridden by the environment variable"
    );

    // Clean up: remove the temporary directory and unset the environment variable
    temp_dir.close().expect("Failed to remove temp dir");
    env::remove_var("BROKKR__LOG__LEVEL");
}

#[test]
/// Tests the loading of default settings when no configuration file is provided.
///
/// This test:
/// 1. Calls Settings::new() with None as the config file path.
/// 2. Verifies that the default settings are correctly loaded.
/// 3. Checks specific default values for database URL and log level.
///
/// Note: This test assumes knowledge of the expected default values. If defaults
/// change, this test will need to be updated accordingly.
fn test_settings_default() {
    // Test loading default settings
    let settings = Settings::new(None).expect("Failed to load default settings");

    assert_eq!(
        settings.database.url,
        "postgres://brokkr:brokkr@localhost:5432/brokkr",
        "Default database URL should match the expected value"
    );
    
    assert_eq!(
        settings.log.level,
        "debug",
        "Default log level should be set to 'debug'"
    );
}