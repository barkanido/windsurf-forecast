//! Configuration File I/O Operations
//!
//! This module handles all file system operations for configuration management,
//! including loading from and saving to TOML files.
//!
//! # File Format
//!
//! Configuration is stored in TOML format at `~/.windsurf-config.toml` by default:
//!
//! ```toml
//! [general]
//! timezone = "Asia/Jerusalem"
//! default_provider = "stormglass"
//! lat = 32.486722
//! lng = 34.888722
//! ```
//!
//! # Structures
//!
//! - [`Config`]: Top-level configuration with `[general]` section
//! - [`GeneralConfig`]: Application configuration fields
//!
//! # Functions
//!
//! - [`load_config()`]: Load configuration from file (returns default if missing)
//! - [`save_config()`]: Persist configuration to file
//! - [`get_default_config_path()`]: Resolve default config file path
//!
//! # Design Principles
//!
//! ## Graceful Defaults
//!
//! Missing config file is **not an error** - returns [`Config::default()`] instead.
//! This allows first-time users to run the application without manual setup.
//!
//! ## Error Context
//!
//! All file I/O errors include:
//! - Operation that failed (read/write/parse)
//! - File path involved
//! - Underlying error cause
//!
//! Example error message:
//! ```text
//! Failed to parse config file: /home/user/.windsurf-config.toml
//! Caused by: TOML parse error at line 3, column 5
//! ```
//!
//! ## Backward Compatibility
//!
//! The TOML structure is **stable** and maintains backward compatibility:
//! - Field names never change
//! - New fields use `#[serde(default)]`
//! - Optional fields use `#[serde(skip_serializing_if)]`
//!
//! # Example Usage
//!

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
/// Optional fields use `Option<T>`.
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

pub fn load_config_from_file(path: Option<&PathBuf>) -> Result<Config> {
    let config_path = if let Some(p) = path {
        p.clone()
    } else {
        get_default_config_path()?
    };

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&config_path).context(format!(
        "Failed to read config file: {}",
        config_path.display()
    ))?;

    let config: Config = toml::from_str(&contents).context(format!(
        "Failed to parse config file: {}",
        config_path.display()
    ))?;

    Ok(config)
}

pub fn save_config(config: &Config, path: Option<&PathBuf>) -> Result<()> {
    let config_path = if let Some(p) = path {
        p.clone()
    } else {
        get_default_config_path()?
    };

    let toml_string =
        toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;

    fs::write(&config_path, toml_string).context(format!(
        "Failed to write config file: {}",
        config_path.display()
    ))?;

    Ok(())
}
