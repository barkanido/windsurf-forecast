// Core configuration data structures
//
// This module defines the types used throughout the configuration system:
// - ResolvedConfig: Final validated configuration
// - ConfigSources: Raw input sources for precedence resolution
// - ConfigSource: Enum for tracking value origins in error messages

use chrono_tz::Tz;
use std::fmt;

/// Final validated configuration containing all resolved values
///
/// All fields are populated (no Option<T>) and have been validated according to
/// business rules. This is the single source of truth for configuration during
/// application execution.
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

/// Raw input sources before precedence resolution
///
/// Option<T> values enable source tracking for better error messages.
/// Precedence order: CLI > Config File > Default
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
/// All fields except lat/lng have defaults.
/// Coordinates must be provided via CLI or config.
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

/// Indicates which source provided a configuration value
///
/// Used in error messages to help users understand where invalid values originated.
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