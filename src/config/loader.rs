// Configuration file loading and persistence
//
// This module handles all file I/O operations for configuration:
// - Load TOML configuration from file
// - Save TOML configuration to file
// - Path resolution for config file location

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Top-level configuration file structure
///
/// TOML format preserved for backward compatibility.
/// File location: ~/.windsurf-config.toml (default)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
}

/// General configuration section
///
/// Field names match existing TOML structure.
/// Optional fields use Option<T>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Timezone identifier (e.g., "UTC", "Asia/Jerusalem")
    #[serde(default = "default_timezone")]
    pub timezone: String,
    
    /// Default weather provider
    #[serde(default = "default_provider")]
    pub default_provider: String,
    
    /// Latitude for forecast location (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    
    /// Longitude for forecast location (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            timezone: default_timezone(),
            default_provider: default_provider(),
            lat: None,
            lng: None,
        }
    }
}

fn default_timezone() -> String {
    "UTC".to_string()
}

fn default_provider() -> String {
    "stormglass".to_string()
}

/// Get the default config file path
///
/// Default location: ~/.windsurf-config.toml
/// Fallback to current directory if home dir unavailable
pub fn get_default_config_path() -> Result<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        Ok(home.join(".windsurf-config.toml"))
    } else {
        Ok(PathBuf::from("windsurf-config.toml"))
    }
}

/// Load configuration from TOML file
///
/// Returns default Config if file doesn't exist (not an error).
/// Returns error for malformed TOML with helpful context.
/// Path parameter: Some(path) for custom location, None for default.
pub fn load_config(path: Option<&PathBuf>) -> Result<Config> {
    let config_path = if let Some(p) = path {
        p.clone()
    } else {
        get_default_config_path()?
    };

    // Return default if file doesn't exist (not an error)
    if !config_path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&config_path)
        .context(format!("Failed to read config file: {}", config_path.display()))?;
    
    let config: Config = toml::from_str(&contents)
        .context(format!("Failed to parse config file: {}", config_path.display()))?;
    
    Ok(config)
}

/// Save configuration to TOML file
///
/// Serializes to pretty-printed TOML.
/// Returns error with file path on I/O failure.
/// Path parameter: Some(path) for custom location, None for default.
pub fn save_config(config: &Config, path: Option<&PathBuf>) -> Result<()> {
    let config_path = if let Some(p) = path {
        p.clone()
    } else {
        get_default_config_path()?
    };

    let toml_string = toml::to_string_pretty(config)
        .context("Failed to serialize config to TOML")?;
    
    fs::write(&config_path, toml_string)
        .context(format!("Failed to write config file: {}", config_path.display()))?;
    
    Ok(())
}