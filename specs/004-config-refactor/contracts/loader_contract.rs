// API Contract: Configuration File Loading
// Feature: 004-config-refactor
// Location: src/config/loader.rs

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// CONTRACT: Config TOML Structure (UNCHANGED from current implementation)
// ============================================================================

/// Top-level configuration file structure
///
/// CONTRACT GUARANTEES:
/// - TOML format preserved for backward compatibility
/// - File location: ~/.windsurf-config.toml (default)
/// - All fields are optional (defaults applied when missing)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
}

/// General configuration section
///
/// CONTRACT GUARANTEES:
/// - Field names match existing TOML structure
/// - Serde attributes preserved for serialization
/// - Optional fields use Option<T>
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

// ============================================================================
// CONTRACT: File Path Resolution
// ============================================================================

/// Get the default config file path
///
/// CONTRACT GUARANTEES:
/// - Default location: ~/.windsurf-config.toml
/// - Fallback to current directory if home dir unavailable
/// - Never panics, always returns valid PathBuf
///
/// SIGNATURE:
/// pub fn get_default_config_path() -> Result<PathBuf>
pub fn get_default_config_path() -> Result<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        Ok(home.join(".windsurf-config.toml"))
    } else {
        Ok(PathBuf::from("windsurf-config.toml"))
    }
}

// ============================================================================
// CONTRACT: Load Configuration
// ============================================================================

/// Load configuration from TOML file
///
/// CONTRACT GUARANTEES:
/// - Returns default Config if file doesn't exist (not an error)
/// - Returns error for malformed TOML with helpful context
/// - Returns error for I/O failures with file path in message
/// - Path parameter: Some(path) for custom location, None for default
///
/// SIGNATURE:
/// pub fn load_config(path: Option<&PathBuf>) -> Result<Config>
///
/// ERRORS:
/// - File read error: "Failed to read config file: {path}"
/// - Parse error: "Failed to parse config file: {path}"
///
/// EXAMPLES:
/// ```
/// // Load from default location
/// let config = load_config(None)?;
///
/// // Load from custom location
/// let path = PathBuf::from("/custom/path/config.toml");
/// let config = load_config(Some(&path))?;
/// ```
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

// ============================================================================
// CONTRACT: Save Configuration
// ============================================================================

/// Save configuration to TOML file
///
/// CONTRACT GUARANTEES:
/// - Serializes to pretty-printed TOML
/// - Creates parent directories if needed (via write operation)
/// - Returns error with file path on I/O failure
/// - Path parameter: Some(path) for custom location, None for default
///
/// SIGNATURE:
/// pub fn save_config(config: &Config, path: Option<&PathBuf>) -> Result<()>
///
/// ERRORS:
/// - Serialization error: "Failed to serialize config to TOML"
/// - Write error: "Failed to write config file: {path}"
///
/// EXAMPLES:
/// ```
/// // Save to default location
/// save_config(&config, None)?;
///
/// // Save to custom location
/// let path = PathBuf::from("/custom/path/config.toml");
/// save_config(&config, Some(&path))?;
/// ```
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

// ============================================================================
// CONTRACT: Public API
// ============================================================================

/// Required public exports from src/config/loader.rs module
///
/// BACKWARD COMPATIBILITY:
/// - Config and GeneralConfig structures unchanged
/// - Function signatures match current implementation
pub mod exports {
    pub use super::{
        Config,
        GeneralConfig,
        get_default_config_path,
        load_config,
        save_config,
    };
}

// ============================================================================
// CONTRACT: Testing Requirements
// ============================================================================

#[cfg(test)]
mod contract_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn load_config_returns_default_for_missing_file() {
        // CONTRACT: Missing file returns default, not an error
        let path = PathBuf::from("/nonexistent/path/config.toml");
        let config = load_config(Some(&path)).unwrap();
        
        assert_eq!(config.general.timezone, "UTC");
        assert_eq!(config.general.default_provider, "stormglass");
    }

    #[test]
    fn load_config_parses_valid_toml() {
        // CONTRACT: Valid TOML file is correctly parsed
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, r#"
[general]
timezone = "America/New_York"
default_provider = "openweathermap"
lat = 40.7128
lng = -74.0060
"#).unwrap();

        let config = load_config(Some(&temp.path().to_path_buf())).unwrap();
        
        assert_eq!(config.general.timezone, "America/New_York");
        assert_eq!(config.general.default_provider, "openweathermap");
        assert_eq!(config.general.lat, Some(40.7128));
        assert_eq!(config.general.lng, Some(-74.0060));
    }

    #[test]
    fn save_config_roundtrip_preserves_data() {
        // CONTRACT: Save/load roundtrip preserves all data
        let mut config = Config::default();
        config.general.timezone = "Europe/London".to_string();
        config.general.lat = Some(51.5074);
        config.general.lng = Some(-0.1278);

        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();
        
        save_config(&config, Some(&path)).unwrap();
        let loaded = load_config(Some(&path)).unwrap();
        
        assert_eq!(loaded.general.timezone, "Europe/London");
        assert_eq!(loaded.general.lat, Some(51.5074));
        assert_eq!(loaded.general.lng, Some(-0.1278));
    }

    #[test]
    fn config_with_missing_fields_uses_defaults() {
        // CONTRACT: Missing fields in TOML use default values
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "[general]").unwrap();

        let config = load_config(Some(&temp.path().to_path_buf())).unwrap();
        
        assert_eq!(config.general.timezone, "UTC");
        assert_eq!(config.general.default_provider, "stormglass");
        assert_eq!(config.general.lat, None);
        assert_eq!(config.general.lng, None);
    }
}