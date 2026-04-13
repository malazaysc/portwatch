use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Refresh interval in seconds
    pub refresh_interval: u64,
    /// Terminal emulator for "open folder" action
    pub terminal: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_interval: 3,
            terminal: "finder".to_string(),
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("portwatch").join("config.toml"))
}

pub fn load() -> Result<Config> {
    let Some(path) = config_path() else {
        return Ok(Config::default());
    };

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}
