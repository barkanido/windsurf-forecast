//! Configuration Data Structures
//!
//! This module defines the core types used throughout the configuration system.
//!
//! # Type Overview
//!
//! ## [`ResolvedConfig`] - Final Validated Configuration
//!
//! The [`ResolvedConfig`] structure represents the final, validated configuration
//! ready for application use. All fields are concrete values (no `Option<T>`),
//! guaranteeing that:
//! - All precedence rules have been applied
//! - All validation rules have passed
//! - All required values are present
//!
//! This is the **single source of truth** for configuration during execution.
//!
//! ## [`ConfigSources`] - Raw Input Tracking
//!
//! The [`ConfigSources`] structure tracks raw inputs from three sources:
//! - [`CliSource`]: CLI arguments (highest priority)
//! - [`FileSource`]: Config file values (medium priority)
//! - [`DefaultSource`]: Default values (lowest priority)
//!
//! All fields use `Option<T>` to distinguish "not provided" from "provided with value".
//! This enables:
//! - Accurate precedence resolution
//! - Source tracking for error messages
//! - Detection of missing required values
//!
//! ## [`ConfigSource`] - Error Message Enhancement
//!
//! The [`ConfigSource`] enum identifies where a configuration value originated.
//! Used in validation errors to help users fix issues at the source:
//! ```text
//! Invalid latitude -95.0 from CLI argument
//! Rule: Latitude must be between -90.0 and 90.0
//! Fix: Provide valid --lat argument
//! ```
//!
//! # Design Principles
//!
//! 1. **Type Safety**: `ResolvedConfig` uses concrete types, not `Option<T>`
//! 2. **Source Tracking**: `Option<T>` in sources enables better error messages
//! 3. **No Partial States**: `ResolvedConfig` cannot exist without all required fields
//! 4. **Validation Boundary**: Raw sources → validation → resolved config
//!
//! # Example Flow
//!
//! ```text
//! CliSource { lat: Some(40.7), lng: Some(-74.0), ... }
//!      ↓
//! FileSource { lat: Some(32.0), lng: Some(34.0), ... }
//!      ↓
//! DefaultSource { provider: "stormglass", ... }
//!      ↓
//! Precedence Resolution (CLI > File > Default)
//!      ↓
//! Validation (ranges, business rules)
//!      ↓
//! ResolvedConfig { lat: 40.7, lng: -74.0, ... }
//! ```

use chrono_tz::Tz;
use std::fmt;

/// Final validated configuration containing all resolved values
///
/// All fields are populated (no `Option<T>`) and have been validated according to
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

impl fmt::Display for ResolvedConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "📋 Configuration:")?;
        writeln!(f, "   Provider: {}", self.provider)?;
        writeln!(f, "   Days ahead: {}", self.days_ahead)?;
        writeln!(f, "   First day offset: {}", self.first_day_offset)?;
        writeln!(f, "   Timezone: {}", self.timezone.name())?;
        write!(f, "   Coordinates: ({:.6}, {:.6})", self.lat, self.lng)
    }
}

/// Raw input sources before precedence resolution
///
/// `Option<T>` values enable source tracking for better error messages.
/// Precedence order: CLI > Config File > Default
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Part of contract, will be used in future enhancements
pub struct ConfigSources {
    pub cli: CliSource,
    pub config_file: FileSource,
    pub defaults: DefaultSource,
}

/// CLI argument source
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Part of contract, will be used in future enhancements
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
#[allow(dead_code)] // Part of contract, will be used in future enhancements
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
#[allow(dead_code)] // Part of contract, will be used in future enhancements
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
#[allow(dead_code)] // Part of contract, will be used in future enhancements
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