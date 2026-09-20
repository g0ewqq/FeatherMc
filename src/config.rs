use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub motd: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "FeatherMC".to_owned(),
            motd: "A FeatherMC Server".to_owned(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&text)?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    pub fn load_or_create(path: &Path) -> Result<Self> {
        if !path.exists() {
            let config = Self::default();
            config.save(path)?;
            return Ok(config);
        }
        Self::load(path)
    }

    pub fn validate(&self) -> Result<()> {
        if self.server.name.trim().is_empty() {
            return Err(Error::InvalidConfig(
                "server.name must not be empty".to_owned(),
            ));
        }
        if self.server.name.len() > 64 {
            return Err(Error::InvalidConfig(
                "server.name must be at most 64 characters".to_owned(),
            ));
        }
        if self.server.motd.len() > 256 {
            return Err(Error::InvalidConfig(
                "server.motd must be at most 256 characters".to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        Config::default().validate().unwrap();
    }

    #[test]
    fn load_or_create_writes_default_once() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("server.toml");

        let first = Config::load_or_create(&path).unwrap();
        assert_eq!(first, Config::default());
        assert!(path.is_file());

        let second = Config::load_or_create(&path).unwrap();
        assert_eq!(second, Config::default());
    }

    #[test]
    fn does_not_overwrite_existing_config() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("server.toml");
        std::fs::write(&path, "[server]\nname = \"Custom\"\nmotd = \"Hi\"\n").unwrap();

        let loaded = Config::load_or_create(&path).unwrap();
        assert_eq!(loaded.server.name, "Custom");

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("Custom"));
    }

    #[test]
    fn rejects_empty_name() {
        let config = Config {
            server: ServerConfig {
                name: "   ".to_owned(),
                motd: "Hi".to_owned(),
            },
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_malformed_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("server.toml");
        std::fs::write(&path, "[server\nname = ").unwrap();
        assert!(Config::load(&path).is_err());
    }
}
