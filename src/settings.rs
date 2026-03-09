use anyhow::Context;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub server: ServerSettings,
    #[serde(default)]
    pub storage: StorageSettings,
    #[serde(default)]
    pub log: LogSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 9000,
            request_timeout_secs: 30,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StorageSettings {
    pub path: PathBuf,
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./data"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LogSettings {
    pub level: String,
    pub format: LogFormat,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("CORRO").separator("__"))
            .build()
            .context("Failed to build configuration")?;

        settings
            .try_deserialize()
            .context("Failed to deserialize configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let settings = Settings {
            server: ServerSettings::default(),
            storage: StorageSettings::default(),
            log: LogSettings::default(),
        };

        assert_eq!(settings.server.host, "0.0.0.0");
        assert_eq!(settings.server.port, 9000);
        assert_eq!(settings.storage.path, PathBuf::from("./data"));
        assert_eq!(settings.log.level, "info");
        assert_eq!(settings.log.format, LogFormat::Pretty);
    }
}
