// API Contract: Configuration Resolution and Validation
// Feature: 004-config-refactor
// Location: src/config/resolver.rs

use anyhow::{anyhow, Result};
use chrono_tz::Tz;
use std::path::PathBuf;

// Import from other config modules
use super::types_contract::{ResolvedConfig, ConfigSources, ConfigSource};
use super::loader_contract::{Config, load_config};
use crate::args::Args;

// ============================================================================
// CONTRACT: Generic Precedence Resolution
// ============================================================================

/// Resolve a single configuration value using precedence rules
///
/// CONTRACT GUARANTEES:
/// - Precedence order: CLI > Config > Default
/// - Works with any type T
/// - Always returns a value (unwrap_or on default)
///
/// SIGNATURE:
/// pub fn resolve<T>(cli: Option<T>, config: Option<T>, default: T) -> T
///
/// EXAMPLES:
/// ```
/// let provider = resolve(
///     args.provider.clone(),
///     Some(config.general.default_provider.clone()),
///     "stormglass".to_string()
/// );
/// ```
pub fn resolve<T>(cli: Option<T>, config: Option<T>, default: T) -> T {
    cli.or(config).unwrap_or(default)
}

/// Resolve a value with source tracking for error messages
///
/// CONTRACT GUARANTEES:
/// - Same precedence as resolve()
/// - Returns tuple: (value, source)
/// - Source indicates where value came from (CLI/Config/Default)
///
/// SIGNATURE:
/// pub fn resolve_with_source<T: Clone>(
///     cli: Option<T>,
///     config: Option<T>,
///     default: T
/// ) -> (T, ConfigSource)
///
/// EXAMPLES:
/// ```
/// let (lat, source) = resolve_with_source(
///     args.lat,
///     config.general.lat,
///     32.486722
/// );
/// // If validation fails, error message includes: "Invalid latitude from {source}"
/// ```
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

// ============================================================================
// CONTRACT: Coordinate Validation
// ============================================================================

/// Validate coordinate ranges
///
/// CONTRACT GUARANTEES:
/// - Latitude range: -90.0 to 90.0 (inclusive)
/// - Longitude range: -180.0 to 180.0 (inclusive)
/// - Error messages include the invalid value and range
///
/// SIGNATURE:
/// pub fn validate_coordinates(lat: f64, lng: f64) -> Result<()>
///
/// ERRORS:
/// - Out of range latitude: "Latitude {lat} is out of valid range. Must be between -90.0 and 90.0 degrees."
/// - Out of range longitude: "Longitude {lng} is out of valid range. Must be between -180.0 and 180.0 degrees."
///
/// EXAMPLES:
/// ```
/// validate_coordinates(32.486722, 34.888722)?; // OK
/// validate_coordinates(91.0, 34.888722)?; // Error: latitude out of range
/// ```
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
/// CONTRACT GUARANTEES:
/// - CLI coordinates override config file coordinates
/// - At least one source must provide both lat AND lng
/// - Coordinates are validated after resolution
/// - Error message indicates if coordinates are missing or invalid
///
/// SIGNATURE:
/// pub fn resolve_coordinates(
///     cli_lat: Option<f64>,
///     cli_lng: Option<f64>,
///     config: &Config,
/// ) -> Result<(f64, f64)>
///
/// ERRORS:
/// - Missing latitude: "Latitude not specified. Provide via --lat argument or configure in config file."
/// - Missing longitude: "Longitude not specified. Provide via --lng argument or configure in config file."
/// - Invalid range: (from validate_coordinates)
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

// ============================================================================
// CONTRACT: Date Range Validation
// ============================================================================

/// Validate date range parameters
///
/// CONTRACT GUARANTEES:
/// - days_ahead range: 1 to 7 (inclusive)
/// - first_day_offset range: 0 to 7 (inclusive)
/// - Business rule: days_ahead + first_day_offset <= 7
/// - Error messages explain the constraint and actual values
///
/// SIGNATURE:
/// pub fn validate_date_range(days_ahead: i32, first_day_offset: i32) -> Result<()>
///
/// ERRORS:
/// - days_ahead out of range: "days-ahead must be between 1 and 7 (got {days_ahead})"
/// - first_day_offset out of range: "first-day-offset must be between 0 and 7 (got {first_day_offset})"
/// - Total exceeds 7: "days-ahead ({days_ahead}) + first-day-offset ({first_day_offset}) = {total} exceeds maximum of 7 days for reliable forecasts"
///
/// EXAMPLES:
/// ```
/// validate_date_range(4, 0)?; // OK
/// validate_date_range(5, 3)?; // Error: 5 + 3 = 8 > 7
/// ```
pub fn validate_date_range(days_ahead: i32, first_day_offset: i32) -> Result<()> {
    if days_ahead < 1 || days_ahead > 7 {
        anyhow::bail!("days-ahead must be between 1 and 7 (got {})", days_ahead);
    }

    if first_day_offset < 0 || first_day_offset > 7 {
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

// ============================================================================
// CONTRACT: Main Resolution Entry Point
// ============================================================================

/// Resolve complete configuration from CLI args and config file
///
/// CONTRACT GUARANTEES:
/// - Loads config file (or default if missing)
/// - Applies precedence rules to all parameters
/// - Validates all resolved values
/// - Returns ResolvedConfig with all fields populated
/// - Errors include parameter name, value, source, and fix suggestion
///
/// SIGNATURE:
/// pub fn resolve_from_args_and_file(args: &Args) -> Result<ResolvedConfig>
///
/// PRECEDENCE RULES:
/// - provider: CLI > Config > Default ("stormglass")
/// - timezone: CLI > Config > Default ("UTC") [handled by TimezoneConfig]
/// - coordinates: CLI > Config > Error (no defaults)
/// - days_ahead: CLI > Default (4)
/// - first_day_offset: CLI > Default (0)
///
/// VALIDATION ORDER:
/// 1. Load config file (may fail on I/O or parse error)
/// 2. Resolve timezone (may fail on invalid timezone identifier)
/// 3. Resolve coordinates (may fail if missing or out of range)
/// 4. Resolve days parameters (always succeed, use defaults)
/// 5. Validate date range (may fail on business rule)
/// 6. Validate provider name (may fail if provider not registered)
///
/// EXAMPLES:
/// ```
/// let args = Args::parse();
/// let config = resolve_from_args_and_file(&args)?;
/// 
/// // config is now ready to use:
/// let provider = create_provider(&config.provider)?;
/// let data = provider.fetch_forecast(
///     config.lat,
///     config.lng,
///     config.days_ahead,
///     config.first_day_offset,
///     config.timezone,
/// ).await?;
/// ```
pub fn resolve_from_args_and_file(args: &Args) -> Result<ResolvedConfig> {
    // 1. Load config file
    let config = load_config(args.config.as_ref())?;
    
    // 2. Resolve timezone
    let timezone_config = crate::config::timezone::TimezoneConfig::load_with_precedence(
        args.timezone.clone(),
        Some(config.general.timezone.clone()),
    )?;
    
    // 3. Resolve coordinates (must be provided, no defaults)
    let (lat, lng) = resolve_coordinates(
        args.lat,
        args.lng,
        &config,
    )?;
    
    // 4. Resolve provider
    let provider = resolve(
        args.provider.clone(),
        Some(config.general.default_provider.clone()),
        "stormglass".to_string(),
    );
    
    // 5. Resolve days parameters
    let days_ahead = args.days_ahead;  // Always has CLI default
    let first_day_offset = args.first_day_offset;  // Always has CLI default
    
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

// ============================================================================
// CONTRACT: Public API
// ============================================================================

/// Required public exports from src/config/resolver.rs module
pub mod exports {
    pub use super::{
        resolve,
        resolve_with_source,
        validate_coordinates,
        resolve_coordinates,
        validate_date_range,
        resolve_from_args_and_file,
    };
}

// ============================================================================
// CONTRACT: Testing Requirements
// ============================================================================

#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn resolve_prefers_cli_over_config() {
        // CONTRACT: CLI takes precedence over config
        let result = resolve(
            Some("cli_value"),
            Some("config_value"),
            "default_value"
        );
        assert_eq!(result, "cli_value");
    }

    #[test]
    fn resolve_prefers_config_over_default() {
        // CONTRACT: Config takes precedence over default
        let result = resolve(
            None::<&str>,
            Some("config_value"),
            "default_value"
        );
        assert_eq!(result, "config_value");
    }

    #[test]
    fn resolve_uses_default_when_others_none() {
        // CONTRACT: Default used when CLI and config are None
        let result = resolve(
            None::<&str>,
            None::<&str>,
            "default_value"
        );
        assert_eq!(result, "default_value");
    }

    #[test]
    fn resolve_with_source_tracks_cli() {
        // CONTRACT: Source tracking identifies CLI origin
        let (val, source) = resolve_with_source(
            Some(42),
            Some(100),
            0
        );
        assert_eq!(val, 42);
        assert_eq!(source, ConfigSource::Cli);
    }

    #[test]
    fn validate_coordinates_accepts_valid_ranges() {
        // CONTRACT: Valid coordinates pass validation
        assert!(validate_coordinates(32.486722, 34.888722).is_ok());
        assert!(validate_coordinates(-90.0, -180.0).is_ok());
        assert!(validate_coordinates(90.0, 180.0).is_ok());
    }

    #[test]
    fn validate_coordinates_rejects_invalid_latitude() {
        // CONTRACT: Out of range latitude fails validation
        assert!(validate_coordinates(91.0, 34.888722).is_err());
        assert!(validate_coordinates(-91.0, 34.888722).is_err());
    }

    #[test]
    fn validate_coordinates_rejects_invalid_longitude() {
        // CONTRACT: Out of range longitude fails validation
        assert!(validate_coordinates(32.486722, 181.0).is_err());
        assert!(validate_coordinates(32.486722, -181.0).is_err());
    }

    #[test]
    fn validate_date_range_accepts_valid_values() {
        // CONTRACT: Valid date ranges pass validation
        assert!(validate_date_range(4, 0).is_ok());
        assert!(validate_date_range(1, 6).is_ok());
        assert!(validate_date_range(7, 0).is_ok());
    }

    #[test]
    fn validate_date_range_enforces_business_rule() {
        // CONTRACT: days_ahead + first_day_offset <= 7
        assert!(validate_date_range(5, 3).is_err());  // 5 + 3 = 8 > 7
        assert!(validate_date_range(4, 4).is_err());  // 4 + 4 = 8 > 7
    }
}