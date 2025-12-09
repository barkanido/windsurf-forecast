// Configuration resolution and validation
//
// This module implements the precedence resolution logic and validation rules:
// - Generic precedence resolution (CLI > Config > Default)
// - Coordinate validation and resolution
// - Date range validation
// - Main entry point for configuration resolution

use anyhow::{anyhow, Result};
use crate::args::Args;
use super::types::{ResolvedConfig, ConfigSource};
use super::loader::{Config, load_config};
use super::timezone::TimezoneConfig;

/// Resolve a single configuration value using precedence rules
///
/// Precedence order: CLI > Config > Default
/// Works with any type T
pub fn resolve<T>(cli: Option<T>, config: Option<T>, default: T) -> T {
    cli.or(config).unwrap_or(default)
}

/// Resolve a value with source tracking for error messages
///
/// Same precedence as resolve(), but returns tuple: (value, source)
/// Source indicates where value came from (CLI/Config/Default)
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

/// Resolve complete configuration from CLI args and config file
///
/// This is the main entry point for configuration resolution.
/// It loads the config file, applies precedence rules to all parameters,
/// validates all resolved values, and returns a fully validated ResolvedConfig.
pub fn resolve_from_args_and_file(args: &Args) -> Result<ResolvedConfig> {
    // 1. Load config file
    let config = load_config(args.config.as_ref())?;
    
    // 2. Resolve timezone
    let timezone_config = TimezoneConfig::load_with_precedence(
        args.timezone.clone(),
        Some(config.general.timezone.clone()),
    )?;
    
    // Display warning if using default timezone
    timezone_config.display_default_warning();
    
    // 3. Resolve coordinates (must be provided, no defaults)
    let (lat, lng) = resolve_coordinates(
        args.lat,
        args.lng,
        &config,
    )?;
    
    // 4. Resolve provider (args.provider always has a value from clap default)
    let provider = args.provider.clone();
    
    // 5. Resolve days parameters (these already have defaults from Args)
    let days_ahead = args.days_ahead;
    let first_day_offset = args.first_day_offset;
    
    // 6. Validate date range
    validate_date_range(days_ahead, first_day_offset)?;
    
    // 7. Validate provider (against registry)
    crate::args::validate_provider(&provider)?;
    
    // 8. Construct validated config
    Ok(ResolvedConfig {
        provider,
        timezone: timezone_config.timezone,
        lat,
        lng,
        days_ahead,
        first_day_offset,
    })
}