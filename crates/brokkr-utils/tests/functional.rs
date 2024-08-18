use brokkr_utils::Settings;
use std::env;
use std::fs;
use tempfile::tempdir;

#[test]
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
    env::set_var("VULCAN__LOG__LEVEL", "debug");

    // Load settings from the test file
    let settings = Settings::new(Some(file_path.to_str().unwrap().to_string()))
        .expect("Failed to load settings");

    // Assert that settings are loaded correctly from the file
    assert_eq!(
        settings.database.url,
        "postgres://user:pass@testhost:5432/testdb"
    );

    // Assert that the environment variable override worked
    assert_eq!(settings.log.level, "debug");

    // Clean up: remove the temporary directory and unset the environment variable
    temp_dir.close().expect("Failed to remove temp dir");
    env::remove_var("VULCAN__LOG__LEVEL");
}

#[test]
fn test_settings_default() {
    // Test loading default settings
    let settings = Settings::new(None).expect("Failed to load default settings");

    assert_eq!(
        settings.database.url,
        "postgres://brokkr:brokkr@localhost:5432/brokkr"
    );
    // Add more assertions for default values as needed
}
