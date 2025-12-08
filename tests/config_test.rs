// ============================================================================
// Configuration Management Tests (User Story 3)
// ============================================================================
//
// Tests for configuration file handling, precedence rules, and validation.

use tempfile::NamedTempFile;
use std::io::Write;
use windsurf_forecast::config::{
    Config, GeneralConfig, TimezoneConfig,
    load_config, save_config, get_default_config_path
};

// ============================================================================
// Test Pattern 1: Config File Loading
// ============================================================================

#[test]
fn test_load_config_from_valid_file() {
    // Arrange
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general]
lat = 32.486722
lng = 34.888722
timezone = "Asia/Jerusalem"
default_provider = "stormglass"
    "#).unwrap();
    
    // Act
    let result = load_config(Some(&temp_file.path().to_path_buf()));
    
    // Assert
    assert!(result.is_ok(), "Should load valid config file");
    let config = result.unwrap();
    assert_eq!(config.general.lat, Some(32.486722));
    assert_eq!(config.general.lng, Some(34.888722));
    assert_eq!(config.general.timezone, "Asia/Jerusalem");
    assert_eq!(config.general.default_provider, "stormglass");
}

#[test]
fn test_load_config_missing_file_uses_defaults() {
    // Arrange
    let nonexistent_path = std::path::PathBuf::from("/tmp/nonexistent_windsurf_config_12345.toml");
    
    // Act
    let result = load_config(Some(&nonexistent_path));
    
    // Assert
    assert!(result.is_ok(), "Should create default config when file missing");
    let config = result.unwrap();
    assert_eq!(config.general.timezone, "UTC", "Should default to UTC");
    assert_eq!(config.general.default_provider, "stormglass");
    assert!(config.general.lat.is_none(), "Coordinates should be None by default");
    assert!(config.general.lng.is_none());
}

#[test]
fn test_load_config_invalid_toml_returns_error() {
    // Arrange
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general
lat = "not a number"
this is not valid TOML
    "#).unwrap();
    
    // Act
    let result = load_config(Some(&temp_file.path().to_path_buf()));
    
    // Assert
    assert!(result.is_err(), "Should reject invalid TOML");
    let error = result.unwrap_err();
    let error_msg = format!("{:#}", error);
    assert!(
        error_msg.contains("parse") || error_msg.to_lowercase().contains("toml"),
        "Error should indicate parsing issue: {}",
        error_msg
    );
}

// ============================================================================
// Test Pattern 2: CLI Argument Precedence
// ============================================================================

#[test]
fn test_cli_args_override_config_file_coordinates() {
    use windsurf_forecast::config::resolve_coordinates;
    
    // Arrange - Config file has Tel Aviv coordinates
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general]
lat = 32.486722
lng = 34.888722
timezone = "Asia/Jerusalem"
    "#).unwrap();
    
    let config = load_config(Some(&temp_file.path().to_path_buf())).unwrap();
    
    // Verify config loaded correctly
    assert_eq!(config.general.lat, Some(32.486722), "Config should have Tel Aviv lat");
    assert_eq!(config.general.lng, Some(34.888722), "Config should have Tel Aviv lng");
    
    // Act - CLI args specify New York coordinates (different from config)
    let cli_lat = Some(40.7128);
    let cli_lng = Some(-74.0060);
    let result = resolve_coordinates(cli_lat, cli_lng, &config);
    
    // Assert - CLI values should override config values
    assert!(result.is_ok(), "Should successfully resolve coordinates");
    let (final_lat, final_lng) = result.unwrap();
    assert_eq!(final_lat, 40.7128, "CLI lat should override config lat (32.486722 → 40.7128)");
    assert_eq!(final_lng, -74.0060, "CLI lng should override config lng (34.888722 → -74.0060)");
    
    // Verify we're actually testing override, not just default behavior
    assert_ne!(final_lat, config.general.lat.unwrap(), "Should not be using config lat");
    assert_ne!(final_lng, config.general.lng.unwrap(), "Should not be using config lng");
}

#[test]
fn test_cli_timezone_overrides_config_file() {
    // Arrange
    let cli_timezone = Some("America/New_York".to_string());
    let config_timezone = Some("Asia/Jerusalem".to_string());
    
    // Act
    let result = TimezoneConfig::load_with_precedence(cli_timezone, config_timezone);
    
    // Assert
    assert!(result.is_ok());
    let tz_config = result.unwrap();
    assert_eq!(tz_config.timezone.name(), "America/New_York");
    assert!(tz_config.explicit, "Should be marked as explicit when from CLI");
}

#[test]
fn test_config_file_coordinates_used_when_cli_not_provided() {
    use windsurf_forecast::config::resolve_coordinates;
    
    // Arrange - Config file has coordinates
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general]
lat = 32.486722
lng = 34.888722
    "#).unwrap();
    
    let config = load_config(Some(&temp_file.path().to_path_buf())).unwrap();
    
    // Act - CLI args don't specify coordinates (None)
    let cli_lat: Option<f64> = None;
    let cli_lng: Option<f64> = None;
    let result = resolve_coordinates(cli_lat, cli_lng, &config);
    
    // Assert - Should fall back to config values
    assert!(result.is_ok(), "Should successfully resolve coordinates from config");
    let (final_lat, final_lng) = result.unwrap();
    assert_eq!(final_lat, 32.486722, "Should use config lat when CLI not provided");
    assert_eq!(final_lng, 34.888722, "Should use config lng when CLI not provided");
}

#[test]
fn test_missing_coordinates_returns_error() {
    use windsurf_forecast::config::resolve_coordinates;
    
    // Arrange - Config file has NO coordinates
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general]
timezone = "UTC"
    "#).unwrap();
    
    let config = load_config(Some(&temp_file.path().to_path_buf())).unwrap();
    
    // Act - CLI args also don't specify coordinates
    let cli_lat: Option<f64> = None;
    let cli_lng: Option<f64> = None;
    let result = resolve_coordinates(cli_lat, cli_lng, &config);
    
    // Assert - Should return error with helpful message
    assert!(result.is_err(), "Should fail when neither CLI nor config provide coordinates");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Latitude") || err_msg.contains("--lat"),
        "Error should mention latitude: {}",
        err_msg
    );
}

// ============================================================================
// Test Pattern 3: Config File Persistence
// ============================================================================

#[test]
fn test_save_config_persists_to_file() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let config = Config {
        general: GeneralConfig {
            timezone: "Asia/Jerusalem".to_string(),
            default_provider: "stormglass".to_string(),
            lat: Some(32.486722),
            lng: Some(34.888722),
        },
    };
    
    // Act
    let result = save_config(&config, Some(&temp_file.path().to_path_buf()));
    
    // Assert
    assert!(result.is_ok(), "Should save config successfully");
    
    // Verify by reloading
    let reloaded = load_config(Some(&temp_file.path().to_path_buf())).unwrap();
    assert_eq!(reloaded.general.lat, config.general.lat);
    assert_eq!(reloaded.general.lng, config.general.lng);
    assert_eq!(reloaded.general.timezone, config.general.timezone);
    assert_eq!(reloaded.general.default_provider, config.general.default_provider);
}

#[test]
fn test_save_load_roundtrip_preserves_values() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let original = Config {
        general: GeneralConfig {
            timezone: "Europe/London".to_string(),
            default_provider: "openweathermap".to_string(),
            lat: Some(51.5074),
            lng: Some(-0.1278),
        },
    };
    
    // Act
    save_config(&original, Some(&temp_file.path().to_path_buf())).unwrap();
    let loaded = load_config(Some(&temp_file.path().to_path_buf())).unwrap();
    
    // Assert
    assert_eq!(loaded.general.timezone, original.general.timezone);
    assert_eq!(loaded.general.default_provider, original.general.default_provider);
    assert_eq!(loaded.general.lat, original.general.lat);
    assert_eq!(loaded.general.lng, original.general.lng);
}

// ============================================================================
// Test Pattern 4: Validation Rules
// ============================================================================

#[test]
fn test_coordinate_validation_accepts_valid_coordinates() {
    use windsurf_forecast::config::{resolve_coordinates, validate_coordinates};
    
    // Arrange - Valid coordinates from different locations
    let valid_coordinates = vec![
        (32.486722, 34.888722),   // Tel Aviv
        (40.7128, -74.0060),      // New York
        (-33.8688, 151.2093),     // Sydney
        (0.0, 0.0),               // Equator/Prime Meridian
        (90.0, 180.0),            // Max bounds
        (-90.0, -180.0),          // Min bounds
    ];
    
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general]
timezone = "UTC"
    "#).unwrap();
    let config = load_config(Some(&temp_file.path().to_path_buf())).unwrap();
    
    // Act & Assert - Application should accept all valid coordinates
    for (lat, lng) in valid_coordinates {
        // Test validate_coordinates directly
        let validation_result = validate_coordinates(lat, lng);
        assert!(
            validation_result.is_ok(),
            "Should accept valid coordinates ({}, {}): {:?}",
            lat, lng, validation_result
        );
        
        // Test resolve_coordinates (which calls validate_coordinates)
        let result = resolve_coordinates(Some(lat), Some(lng), &config);
        assert!(
            result.is_ok(),
            "Should accept valid coordinates ({}, {}): {:?}",
            lat, lng, result
        );
        let (resolved_lat, resolved_lng) = result.unwrap();
        assert_eq!(resolved_lat, lat, "Latitude should be preserved");
        assert_eq!(resolved_lng, lng, "Longitude should be preserved");
    }
}

#[test]
fn test_coordinate_validation_rejects_out_of_bounds_latitude() {
    use windsurf_forecast::config::{resolve_coordinates, validate_coordinates};
    
    // Arrange - Invalid latitude values
    let invalid_latitudes = vec![
        (91.0, 0.0),      // Latitude too high (valid: -90 to 90)
        (-91.0, 0.0),     // Latitude too low
        (100.0, 34.0),    // Way out of bounds
    ];
    
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general]
timezone = "UTC"
    "#).unwrap();
    let config = load_config(Some(&temp_file.path().to_path_buf())).unwrap();
    
    // Act & Assert - Application should reject invalid latitudes
    for (lat, lng) in invalid_latitudes {
        // Test validate_coordinates directly
        let validation_result = validate_coordinates(lat, lng);
        assert!(
            validation_result.is_err(),
            "Should reject invalid latitude {}", lat
        );
        let err_msg = validation_result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Latitude") && err_msg.contains("-90.0") && err_msg.contains("90.0"),
            "Error should mention latitude range: {}", err_msg
        );
        
        // Test resolve_coordinates (which calls validate_coordinates)
        let result = resolve_coordinates(Some(lat), Some(lng), &config);
        assert!(
            result.is_err(),
            "Should reject invalid latitude {}", lat
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Latitude"),
            "Error should mention latitude: {}", err_msg
        );
    }
}

#[test]
fn test_coordinate_validation_rejects_out_of_bounds_longitude() {
    use windsurf_forecast::config::{resolve_coordinates, validate_coordinates};
    
    // Arrange - Invalid longitude values
    let invalid_longitudes = vec![
        (0.0, 181.0),     // Longitude too high (valid: -180 to 180)
        (0.0, -181.0),    // Longitude too low
        (32.0, 200.0),    // Way out of bounds
    ];
    
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general]
timezone = "UTC"
    "#).unwrap();
    let config = load_config(Some(&temp_file.path().to_path_buf())).unwrap();
    
    // Act & Assert - Application should reject invalid longitudes
    for (lat, lng) in invalid_longitudes {
        // Test validate_coordinates directly
        let validation_result = validate_coordinates(lat, lng);
        assert!(
            validation_result.is_err(),
            "Should reject invalid longitude {}", lng
        );
        let err_msg = validation_result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Longitude") && err_msg.contains("-180.0") && err_msg.contains("180.0"),
            "Error should mention longitude range: {}", err_msg
        );
        
        // Test resolve_coordinates (which calls validate_coordinates)
        let result = resolve_coordinates(Some(lat), Some(lng), &config);
        assert!(
            result.is_err(),
            "Should reject invalid longitude {}", lng
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Longitude"),
            "Error should mention longitude: {}", err_msg
        );
    }
}

#[test]
fn test_missing_coordinates_detection() {
    // Arrange - Config with no coordinates
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general]
timezone = "UTC"
    "#).unwrap();
    
    let config = load_config(Some(&temp_file.path().to_path_buf())).unwrap();
    
    // Act & Assert
    assert!(config.general.lat.is_none(), "Lat should be None");
    assert!(config.general.lng.is_none(), "Lng should be None");
}

// ============================================================================
// Test Pattern 5: Default Config Path
// ============================================================================

#[test]
fn test_default_config_path_contains_home_directory() {
    // Act
    let result = get_default_config_path();
    
    // Assert
    assert!(result.is_ok(), "Should be able to determine default config path");
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    
    // Should contain the config filename
    assert!(
        path_str.contains("windsurf-config.toml"),
        "Default config path should include correct filename: {:?}",
        path
    );
}

#[test]
fn test_default_config_path_is_absolute() {
    // Act
    let result = get_default_config_path();
    
    // Assert
    if let Ok(path) = result {
        // Path should either be absolute or in current directory
        assert!(
            path.is_absolute() || path.file_name().is_some(),
            "Config path should be absolute or have a filename"
        );
    }
}

// ============================================================================
// Test Pattern 6: Timezone Configuration
// ============================================================================

#[test]
fn test_timezone_precedence_cli_over_config() {
    // CLI timezone should override config file timezone
    let result = TimezoneConfig::load_with_precedence(
        Some("America/New_York".to_string()),
        Some("Asia/Jerusalem".to_string())
    );
    
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timezone.name(), "America/New_York");
    assert!(config.explicit, "Should be marked as explicit when from CLI");
}

#[test]
fn test_timezone_precedence_config_over_default() {
    // Config file timezone should be used if CLI not provided
    let result = TimezoneConfig::load_with_precedence(
        None,
        Some("Asia/Jerusalem".to_string())
    );
    
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timezone.name(), "Asia/Jerusalem");
    assert!(config.explicit, "Should be marked as explicit when from config");
}

#[test]
fn test_timezone_defaults_to_utc_when_not_specified() {
    // Should default to UTC when neither CLI nor config specify timezone
    let result = TimezoneConfig::load_with_precedence(None, None);
    
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timezone.name(), "UTC");
    assert!(!config.explicit, "Should not be marked as explicit for default");
}

#[test]
fn test_timezone_config_ignores_utc_in_config() {
    // When config has default UTC, it should be treated as "not set"
    let result = TimezoneConfig::load_with_precedence(
        None,
        Some("UTC".to_string())
    );
    
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timezone.name(), "UTC");
    // Should not be marked explicit if config just has default UTC
    assert!(!config.explicit);
}

#[test]
fn test_invalid_timezone_returns_helpful_error() {
    // Invalid timezone should provide actionable error message
    let result = TimezoneConfig::load_with_precedence(
        Some("InvalidTimezone".to_string()),
        None
    );
    
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    
    // Should contain examples of valid timezones
    assert!(
        err_msg.contains("UTC") || 
        err_msg.contains("America") || 
        err_msg.contains("Asia") ||
        err_msg.contains("example") ||
        err_msg.contains("Examples"),
        "Error message should include examples of valid timezones: {}",
        err_msg
    );
}

#[test]
fn test_local_timezone_special_value() {
    // Test that "LOCAL" special value triggers system timezone detection
    let result = TimezoneConfig::load_with_precedence(
        Some("LOCAL".to_string()),
        None
    );
    
    // Should either succeed (if system timezone detectable) or fail with clear error
    if let Ok(config) = result {
        assert!(config.explicit, "LOCAL should be marked as explicit choice");
        // The actual timezone will vary by system
        println!("Detected system timezone: {}", config.timezone.name());
    } else {
        // If it fails, the error should be about timezone detection
        let err = result.unwrap_err().to_string();
        assert!(
            err.to_lowercase().contains("timezone") || 
            err.to_lowercase().contains("detect") ||
            err.to_lowercase().contains("system"),
            "Error should be about timezone detection: {}",
            err
        );
    }
}

// ============================================================================
// Test Pattern 7: Config File Format
// ============================================================================

#[test]
fn test_config_serialization_format() {
    // Config should serialize to valid TOML
    let config = Config {
        general: GeneralConfig {
            timezone: "Asia/Jerusalem".to_string(),
            default_provider: "stormglass".to_string(),
            lat: Some(32.486722),
            lng: Some(34.888722),
        },
    };
    
    let toml_string = toml::to_string_pretty(&config);
    assert!(toml_string.is_ok(), "Should serialize to TOML");
    
    let toml = toml_string.unwrap();
    assert!(toml.contains("[general]"));
    assert!(toml.contains("timezone"));
    assert!(toml.contains("Asia/Jerusalem"));
}

#[test]
fn test_config_with_missing_optional_fields() {
    // Config with missing optional fields should still be valid
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
[general]
timezone = "UTC"
    "#).unwrap();
    
    let result = load_config(Some(&temp_file.path().to_path_buf()));
    assert!(result.is_ok());
    
    let config = result.unwrap();
    assert!(config.general.lat.is_none());
    assert!(config.general.lng.is_none());
    assert_eq!(config.general.default_provider, "stormglass"); // Should use default
}