use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: Option<String>,
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("keentools-cloud").join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(Config::default()),
    };

    if !path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))
}

pub fn save_token(token: &str) -> Result<PathBuf> {
    let path = config_path().context("Cannot determine config directory")?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    let mut config = load().unwrap_or_default();
    config.auth.token = Some(token.to_string());

    let content = toml::to_string_pretty(&config).context("Failed to serialize config")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write config: {}", path.display()))?;

    Ok(path)
}

pub fn clear_token() -> Result<()> {
    let mut config = load().unwrap_or_default();
    config.auth.token = None;

    if let Some(path) = config_path() {
        if path.exists() {
            let content = toml::to_string_pretty(&config).context("Failed to serialize config")?;
            std::fs::write(&path, content)
                .with_context(|| format!("Failed to write config: {}", path.display()))?;
        }
    }
    Ok(())
}
