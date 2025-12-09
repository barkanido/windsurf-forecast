// API Contract: Core Configuration Types
// Feature: 004-config-refactor
// Location: src/config/types.rs

use anyhow::Result;
use chrono_tz::Tz;
use std::fmt;

// ============================================================================
// CONTRACT: ResolvedConfig
// ============================================================================

/// Final validated configuration containing all resolved values
///
/// CONTRACT GUARANTEES:
/// - All fields are populated (no Option<T>)
/// - All values have been validated according to business rules
/// - Immutable after creation
/// - Single source of truth for configuration during execution
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedConfig {
    /// Weather provider name (validated against registry)
    pub provider: String,
    
    /// Target timezone for output timestamps
    pub timezone: Tz,
    
    /// Latitude coordinate (validated: -90.0 to 90.0)
    pub lat: f64,
    
    /// Longitude coordinate (validated: -180.0 to 180.0)
    pub lng: f64,
    
    /// Number of days to forecast ahead (validated: 1-7)
    pub days_ahead: i32,
    
    /// Offset for forecast start date (validated: 0-7)
    pub first_day_offset: i32,
}

// ============================================================================
// CONTRACT: ConfigSources
// ============================================================================

/// Raw input sources before precedence resolution
///
/// CONTRACT GUARANTEES:
/// - Option<T> values enable source tracking
/// - All sources can be None (defaults will be applied)
/// - Precedence order: CLI > Config File > Default
#[derive(Debug, Clone, Default)]
pub struct ConfigSources {
    pub cli: CliSource,
    pub config_file: FileSource,
    pub defaults: DefaultSource,
}

/// CLI argument source
#[derive(Debug, Clone, Default)]
pub struct CliSource {
    pub provider: Option<String>,
    pub timezone: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub days_ahead: Option<i32>,
    pub first_day_offset: Option<i32>,
}

/// Config file source
#[derive(Debug, Clone, Default)]
pub struct FileSource {
    pub provider: Option<String>,
    pub timezone: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
}

/// Default values source
///
/// CONTRACT GUARANTEES:
/// - All fields except lat/lng have defaults
/// - Coordinates must be provided via CLI or config
#[derive(Debug, Clone)]
pub struct DefaultSource {
    pub provider: String,          // Default: "stormglass"
    pub timezone: String,           // Default: "UTC"
    pub days_ahead: i32,            // Default: 4
    pub first_day_offset: i32,      // Default: 0
}

impl Default for DefaultSource {
    fn default() -> Self {
        Self {
            provider: "stormglass".to_string(),
            timezone: "UTC".to_string(),
            days_ahead: 4,
            first_day_offset: 0,
        }
    }
}

// ============================================================================
// CONTRACT: ConfigSource Enum
// ============================================================================

/// Indicates which source provided a configuration value
///
/// CONTRACT GUARANTEES:
/// - Used in error messages for user guidance
/// - Display implementation provides human-readable names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    /// Value provided via CLI argument
    Cli,
    
    /// Value loaded from config file
    ConfigFile,
    
    /// Default value applied
    Default,
}

impl fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigSource::Cli => write!(f, "CLI argument"),
            ConfigSource::ConfigFile => write!(f, "config file"),
            ConfigSource::Default => write!(f, "default value"),
        }
    }
}

// ============================================================================
// CONTRACT: Public API
// ============================================================================

/// Required public exports from src/config/types.rs module
///
/// BACKWARD COMPATIBILITY:
/// - New types, no breaking changes to existing code
/// - ResolvedConfig replaces scattered variables in main.rs
pub mod exports {
    pub use super::{
        ResolvedConfig,
        ConfigSources,
        CliSource,
        FileSource,
        DefaultSource,
        ConfigSource,
    };
}

// ============================================================================
// CONTRACT: Testing Requirements
// ============================================================================

#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn resolved_config_contains_all_required_fields() {
        // CONTRACT: ResolvedConfig must have all fields populated
        let config = ResolvedConfig {
            provider: "stormglass".to_string(),
            timezone: chrono_tz::UTC,
            lat: 32.486722,
            lng: 34.888722,
            days_ahead: 4,
            first_day_offset: 0,
        };
        
        assert_eq!(config.provider, "stormglass");
        assert_eq!(config.timezone, chrono_tz::UTC);
        assert_eq!(config.lat, 32.486722);
        assert_eq!(config.lng, 34.888722);
        assert_eq!(config.days_ahead, 4);
        assert_eq!(config.first_day_offset, 0);
    }

    #[test]
    fn config_sources_default_has_all_none() {
        // CONTRACT: ConfigSources default constructor creates empty sources
        let sources = ConfigSources::default();
        
        assert!(sources.cli.provider.is_none());
        assert!(sources.cli.timezone.is_none());
        assert!(sources.config_file.provider.is_none());
        assert!(sources.config_file.timezone.is_none());
    }

    #[test]
    fn default_source_provides_required_defaults() {
        // CONTRACT: DefaultSource has correct default values
        let defaults = DefaultSource::default();
        
        assert_eq!(defaults.provider, "stormglass");
        assert_eq!(defaults.timezone, "UTC");
        assert_eq!(defaults.days_ahead, 4);
        assert_eq!(defaults.first_day_offset, 0);
    }

    #[test]
    fn config_source_display_is_human_readable() {
        // CONTRACT: ConfigSource Display implementation for error messages
        assert_eq!(ConfigSource::Cli.to_string(), "CLI argument");
        assert_eq!(ConfigSource::ConfigFile.to_string(), "config file");
        assert_eq!(ConfigSource::Default.to_string(), "default value");
    }
}