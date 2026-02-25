use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Application configuration, persisted to `config.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerSection,
    #[serde(default)]
    pub ui: UiSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerSection {
    #[serde(default = "default_models")]
    pub models: Vec<String>,
    #[serde(default = "default_device")]
    pub device: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub python: Option<String>,
    #[serde(default = "default_script_path")]
    pub script_path: String,
    #[serde(default = "default_model_size")]
    pub model_size: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UiSection {
    #[serde(default)]
    pub dark_mode: bool,
}

impl Default for ServerSection {
    fn default() -> Self {
        Self {
            models: default_models(),
            device: default_device(),
            port: default_port(),
            python: None,
            script_path: default_script_path(),
            model_size: default_model_size(),
        }
    }
}

fn default_models() -> Vec<String> {
    vec![
        "base".to_owned(),
        "voice_design".to_owned(),
        "custom_voice".to_owned(),
    ]
}

fn default_device() -> String {
    "auto".to_owned()
}

fn default_port() -> u16 {
    8000
}

fn default_script_path() -> String {
    "python/start_server.py".to_owned()
}

fn default_model_size() -> String {
    "1.7B".to_owned()
}

/// Return the path to `config.toml` in the data directory.
pub fn config_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("qvox").join("config.toml")
}

/// Load config from disk, returning defaults if the file does not exist.
pub fn load() -> AppConfig {
    let path = config_path();
    if !path.exists() {
        return AppConfig::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

/// Save config to disk.
pub fn save(config: &AppConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("failed to create config directory")?;
    }
    let contents = toml::to_string_pretty(config).context("failed to serialize config")?;
    std::fs::write(&path, contents).context("failed to write config file")?;
    Ok(())
}

impl AppConfig {
    /// Convert to `ServerConfig` for the server manager.
    pub fn to_server_config(&self) -> crate::server::manager::ServerConfig {
        crate::server::manager::ServerConfig {
            models: self.server.models.clone(),
            device: self.server.device.clone(),
            port: self.server.port,
            python_path: self.server.python.clone(),
            script_path: self.server.script_path.clone(),
            model_size: self.server.model_size.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trip() {
        let config = AppConfig::default();
        let toml_str = toml::to_string_pretty(&config).expect("serialize");
        let decoded: AppConfig = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(config, decoded);
    }

    #[test]
    fn deserialize_empty_toml() {
        let config: AppConfig = toml::from_str("").expect("deserialize empty");
        assert_eq!(config.server.port, 8000);
        assert_eq!(config.server.device, "auto");
        assert!(!config.ui.dark_mode);
    }

    #[test]
    fn deserialize_partial_toml() {
        let toml_str = r#"
[server]
port = 9000
models = ["base", "voice_design"]

[ui]
dark_mode = true
"#;
        let config: AppConfig = toml::from_str(toml_str).expect("deserialize");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.server.models, vec!["base", "voice_design"]);
        assert!(config.ui.dark_mode);
        assert_eq!(config.server.device, "auto");
    }

    #[test]
    fn to_server_config() {
        let config = AppConfig::default();
        let sc = config.to_server_config();
        assert_eq!(sc.port, 8000);
        assert_eq!(sc.models, vec!["base", "voice_design", "custom_voice"]);
        assert_eq!(sc.device, "auto");
    }

    #[test]
    fn config_path_has_filename() {
        let path = config_path();
        assert_eq!(path.file_name().and_then(|f| f.to_str()), Some("config.toml"));
    }
}
