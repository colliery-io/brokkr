use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

const DEFAULT_SETTINGS: &str = include_str!("../default.toml");

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub database: Database,
    pub log: Log,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub level: String,
}

impl Settings {
    pub fn new(file: Option<String>) -> Result<Self, ConfigError> {
        // we start with defaults
        let mut s = Config::builder()
            .add_source(File::from_str(DEFAULT_SETTINGS, config::FileFormat::Toml));

        // layer in a passed in file if it is passed
        s = match file {
            Some(x) => s.add_source(File::with_name(x.as_str())),
            None => s,
        };

        // then environment variables (Prefixed with "VULCAN", separated with dunders for trees)
        s = s.add_source(Environment::with_prefix("VULCAN").separator("__"));

        let settings = s.build().unwrap();

        settings.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;
    #[test]
    fn test_settings() {
        let settings = Settings::new(None).unwrap();
        assert_eq!(
            settings.database.url,
            "postgres://brokkr:brokkr@localhost:5432/brokkr"
        );
    }
}
