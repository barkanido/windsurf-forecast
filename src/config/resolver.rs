//! Configuration Resolution and Validation
//!
//! This module implements the core logic for resolving configuration values
//! from multiple sources and validating them according to business rules.
//!
//! # Precedence Resolution
//!
//! All configuration parameters follow the same precedence order:
//! **CLI > Config File > Default**
//!
//! ## Generic Resolution Functions
//!
//! ### [`resolve()`] - Simple Precedence
//! ```rust
//! use windsurf_forecast::config::resolver::resolve;
//!
//! let value = resolve(
//!     Some("cli_value"),      // CLI argument (highest priority)
//!     Some("config_value"),   // Config file value
//!     "default_value"         // Default (lowest priority)
//! );
//! assert_eq!(value, "cli_value");
//! ```
//!
//! ### [`resolve_with_source()`] - Precedence with Source Tracking
//! Same as `resolve()` but returns a tuple `(value, source)` indicating where
//! the value originated. Used for enhanced error messages.
//!
//! # Validation Functions
//!
//! ## Coordinate Validation
//!
//! - [`validate_coordinates()`]: Validates lat/lng ranges
//!   - Latitude: -90.0 to 90.0 (inclusive)
//!   - Longitude: -180.0 to 180.0 (inclusive)
//!
//! - [`resolve_coordinates()`]: Applies precedence then validates
//!   - Returns error if neither CLI nor config provides both lat AND lng
//!
//! ## Date Range Validation
//!
//! - [`validate_date_range()`]: Validates forecast parameters
//!   - `days_ahead`: 1-7 (inclusive)
//!   - `first_day_offset`: 0-7 (inclusive)
//!   - Business rule: `days_ahead + first_day_offset ≤ 7`
//!
//! # Main Entry Point
//!
//! [`resolve_from_args_and_file()`] is the primary function that:
//! 1. Loads config file
//! 2. Applies precedence to all parameters
//! 3. Validates all resolved values
//! 4. Returns [`ResolvedConfig`] ready for use
//!
//! # Error Messages
//!
//! Validation errors include:
//! - Parameter name
//! - Invalid value
//! - Validation rule violated
//! - Suggested fix
//!
//! Example:
//! ```text
//! Latitude 95.0 is out of valid range.
//! Must be between -90.0 and 90.0 degrees.
//! ```
//!
//! # Example Flow
//!
//! ```text
//! Args { lat: Some(40.7), lng: Some(-74.0), days_ahead: 3, ... }
//!           ↓
//! load_config() → Config { lat: Some(32.0), lng: Some(34.0), ... }
//!           ↓
//! resolve_coordinates() → (40.7, -74.0)  [CLI wins]
//!           ↓
//! validate_coordinates(40.7, -74.0) → ✓ OK
//!           ↓
//! validate_date_range(3, 0) → ✓ OK
//!           ↓
//! ResolvedConfig { lat: 40.7, lng: -74.0, days_ahead: 3, ... }
//! ```

use anyhow::{anyhow, Result};
use crate::args::Args;
use super::types::{ResolvedConfig, ConfigSource};
use super::loader::{Config, load_config};
use super::timezone::TimezoneConfig;

/// Resolve a single configuration value using precedence rules
///
/// Precedence order: CLI > Config > Default
/// Works with any type T
#[allow(dead_code)] // Part of contract, will be used in future enhancements
pub fn resolve<T>(cli: Option<T>, config: Option<T>, default: T) -> T {
    cli.or(config).unwrap_or(default)
}

/// Resolve a value with source tracking for error messages
///
/// Same precedence as resolve(), but returns tuple: (value, source)
/// Source indicates where value came from (CLI/Config/Default)
#[allow(dead_code)] // Part of contract, will be used in future enhancements
pub fn resolve_with_source<T: Clone>(
    cli: Option<T>,
    config: Option<T>,
    default: T
) -> (T, ConfigSource) {
    if let Some(val) = cli {
        (val, ConfigSource::Cli)
    } else if let Some(val) = config {
        (val, ConfigSource::ConfigFile)
    } else {
        (default, ConfigSource::Default)
    }
}

/// Validate coordinate ranges
///
/// Latitude range: -90.0 to 90.0 (inclusive)
/// Longitude range: -180.0 to 180.0 (inclusive)
pub fn validate_coordinates(lat: f64, lng: f64) -> Result<()> {
    if !(-90.0..=90.0).contains(&lat) {
        anyhow::bail!(
            "Latitude {} is out of valid range. Must be between -90.0 and 90.0 degrees.",
            lat
        );
    }
    
    if !(-180.0..=180.0).contains(&lng) {
        anyhow::bail!(
            "Longitude {} is out of valid range. Must be between -180.0 and 180.0 degrees.",
            lng
        );
    }
    
    Ok(())
}

/// Resolve coordinates with precedence and validation
///
/// CLI coordinates override config file coordinates.
/// At least one source must provide both lat AND lng.
/// Coordinates are validated after resolution.
pub fn resolve_coordinates(
    cli_lat: Option<f64>,
    cli_lng: Option<f64>,
    config: &Config,
) -> Result<(f64, f64)> {
    let lat = cli_lat
        .or(config.general.lat)
        .ok_or_else(|| anyhow!(
            "Latitude not specified. Provide via --lat argument or configure in config file."
        ))?;
    
    let lng = cli_lng
        .or(config.general.lng)
        .ok_or_else(|| anyhow!(
            "Longitude not specified. Provide via --lng argument or configure in config file."
        ))?;
    
    validate_coordinates(lat, lng)?;
    
    Ok((lat, lng))
}

/// Validate date range parameters
///
/// - days_ahead range: 1 to 7 (inclusive)
/// - first_day_offset range: 0 to 7 (inclusive)
/// - Business rule: days_ahead + first_day_offset <= 7
pub fn validate_date_range(days_ahead: i32, first_day_offset: i32) -> Result<()> {
    if !(1..=7).contains(&days_ahead) {
        anyhow::bail!("days-ahead must be between 1 and 7 (got {})", days_ahead);
    }

    if !(0..=7).contains(&first_day_offset) {
        anyhow::bail!(
            "first-day-offset must be between 0 and 7 (got {})",
            first_day_offset
        );
    }

    let total_days = days_ahead + first_day_offset;
    if total_days > 7 {
        anyhow::bail!(
            "days-ahead ({}) + first-day-offset ({}) = {} exceeds maximum of 7 days for reliable forecasts",
            days_ahead,
            first_day_offset,
            total_days
        );
    }

    Ok(())
}

pub fn resolve_from_args_and_file(args: &Args) -> Result<ResolvedConfig> {
    let config = load_config(args.config.as_ref())?;
    
    let timezone_config = TimezoneConfig::load_with_precedence(
        args.timezone.clone(),
        Some(config.general.timezone.clone()),
    )?;
    
    timezone_config.display_timezone_warning_if_default();
    
    let (lat, lng) = resolve_coordinates(
        args.lat,
        args.lng,
        &config,
    )?;
    
    let provider = args.provider.clone();
    
    let days_ahead = args.days_ahead;
    let first_day_offset = args.first_day_offset;
    
    validate_date_range(days_ahead, first_day_offset)?;
    
    crate::args::validate_provider(&provider)?;
    
    Ok(ResolvedConfig {
        provider,
        timezone: timezone_config.timezone,
        lat,
        lng,
        days_ahead,
        first_day_offset,
    })
}