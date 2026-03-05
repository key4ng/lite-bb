use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

use crate::auth::Credentials;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(PathBuf),
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("no credentials configured — run `bb auth login`")]
    NoCredentials,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Provider {
    Cloud,
    Server { base_url: String },
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub token: Option<String>,
    pub username: Option<String>,
    pub app_password: Option<String>,
    pub workspace: Option<String>,
    pub default_repo: Option<String>,
    pub server_url: Option<String>,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("BB_CONFIG_DIR") {
            PathBuf::from(dir)
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/.config"))
                .join("bb")
        }
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.yml")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let contents = serde_yaml::to_string(self)?;
        std::fs::write(Self::config_path(), contents)?;
        Ok(())
    }

    pub fn credentials(&self) -> Result<Credentials, ConfigError> {
        // Env vars take priority
        if let Ok(token) = std::env::var("BB_TOKEN") {
            return Ok(Credentials::Token(token));
        }
        if let (Ok(username), Ok(app_password)) =
            (std::env::var("BB_USERNAME"), std::env::var("BB_APP_PASSWORD"))
        {
            return Ok(Credentials::AppPassword { username, app_password });
        }

        // Fall back to config file
        if let Some(token) = &self.token {
            return Ok(Credentials::Token(token.clone()));
        }
        if let (Some(username), Some(app_password)) = (&self.username, &self.app_password) {
            return Ok(Credentials::AppPassword {
                username: username.clone(),
                app_password: app_password.clone(),
            });
        }

        Err(ConfigError::NoCredentials)
    }

    pub fn provider(&self) -> Provider {
        // Env var takes priority
        if let Ok(url) = std::env::var("BB_SERVER_URL") {
            return Provider::Server {
                base_url: url.trim_end_matches('/').to_string(),
            };
        }
        // Fall back to config
        if let Some(url) = &self.server_url {
            return Provider::Server {
                base_url: url.trim_end_matches('/').to_string(),
            };
        }
        Provider::Cloud
    }
}
