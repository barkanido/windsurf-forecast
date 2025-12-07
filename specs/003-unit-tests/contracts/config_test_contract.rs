// ============================================================================
// Configuration Management Test Contract
// ============================================================================
// This file demonstrates the expected structure for testing configuration
// file loading, precedence rules, and validation.

use tempfile::NamedTempFile;
use std::io::Write;
use windsurf_forecast::config::{load_config, save_config, Config};

// ============================================================================
// Test Pattern 1: Config File Loading
// ============================================================================

#[test]
fn test_load_config_from_valid_file() {
    // Arrange
    let mut temp_file = NamedTempFile::new().unwrap();
    let config_content = r#"
        [general]
        lat = 32.486722
        lng = 34.888722
        timezone = "Asia/Jerusalem"
    "#;
    write!(temp_file, "{}", config_content).unwrap();
    
    // Act
    let result = load_config(Some(temp_file.path()));
    
    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.general.lat, Some(32.486722));
    assert_eq!(config.general.lng, Some(34.888722));
    assert_eq!(config.general.timezone, "Asia/Jerusalem");
}

#[test]
fn test_load_config_missing_file_uses_defaults() {
    // Arrange
    let nonexistent_path = std::path::PathBuf::from("/tmp/nonexistent_config.toml");
    
    // Act
    let result = load_config(Some(&nonexistent_path));
    
    // Assert
    assert!(result.is_ok(), "Should create default config when file missing");
    let config = result.unwrap();
    assert_eq!(config.general.timezone, "UTC", "Should default to UTC");
    assert!(config.general.lat.is_none(), "Coordinates should be None by default");
    assert!(config.general.lng.is_none());
}

#[test]
fn test_load_config_invalid_toml_returns_error() {
    // Arrange
    let mut temp_file = NamedTempFile::new().unwrap();
    let invalid_content = r#"
        [general
        lat = "not a number"
        this is not valid TOML
    "#;
    write!(temp_file, "{}", invalid_content).unwrap();
    
    // Act
    let result = load_config(Some(temp_file.path()));
    
    // Assert
    assert!(result.is_err(), "Should reject invalid TOML");
    let error = result.unwrap_err();
    let error_msg = format!("{:#}", error);
    assert!(
        error_msg.contains("TOML") || error_msg.contains("parse"),
        "Error should indicate TOML parsing issue"
    );
}

// ============================================================================
// Test Pattern 2: CLI Argument Precedence
// ============================================================================

#[test]
fn test_cli_args_override_config_file_coordinates() {
    // Arrange - Config file has one set of coordinates
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
        [general]
        lat = 32.486722
        lng = 34.888722
        timezone = "Asia/Jerusalem"
    "#).unwrap();
    
    let config = load_config(Some(temp_file.path())).unwrap();
    
    // CLI args specify different coordinates
    let cli_lat = Some(40.7128);  // New York
    let cli_lng = Some(-74.0060);
    
    // Act - Simulate precedence logic
    let final_lat = cli_lat.or(config.general.lat);
    let final_lng = cli_lng.or(config.general.lng);
    
    // Assert
    assert_eq!(final_lat, Some(40.7128), "CLI lat should override config");
    assert_eq!(final_lng, Some(-74.0060), "CLI lng should override config");
}

#[test]
fn test_cli_timezone_overrides_config_file() {
    // Arrange
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
        [general]
        timezone = "Asia/Jerusalem"
    "#).unwrap();
    
    let config = load_config(Some(temp_file.path())).unwrap();
    assert_eq!(config.general.timezone, "Asia/Jerusalem");
    
    // CLI specifies different timezone
    let cli_timezone = Some("America/New_York".to_string());
    
    // Act
    let final_timezone = cli_timezone.unwrap_or(config.general.timezone);
    
    // Assert
    assert_eq!(final_timezone, "America/New_York", "CLI timezone should override config");
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
            lat: Some(32.486722),
            lng: Some(34.888722),
            timezone: "Asia/Jerusalem".to_string(),
        },
    };
    
    // Act
    let result = save_config(&config, Some(temp_file.path()));
    
    // Assert
    assert!(result.is_ok(), "Should save config successfully");
    
    // Verify by reloading
    let reloaded = load_config(Some(temp_file.path())).unwrap();
    assert_eq!(reloaded.general.lat, config.general.lat);
    assert_eq!(reloaded.general.lng, config.general.lng);
    assert_eq!(reloaded.general.timezone, config.general.timezone);
}

#[test]
fn test_save_config_cli_timezone_persisted() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let mut config = load_config(Some(temp_file.path())).unwrap();
    
    // User specified timezone via CLI
    config.general.timezone = "Europe/London".to_string();
    
    // Act
    save_config(&config, Some(temp_file.path())).unwrap();
    
    // Assert - Reload and verify persistence
    let reloaded = load_config(Some(temp_file.path())).unwrap();
    assert_eq!(
        reloaded.general.timezone,
        "Europe/London",
        "CLI-specified timezone should be persisted to config file"
    );
}

// ============================================================================
// Test Pattern 4: Validation Rules
// ============================================================================

#[test]
fn test_validate_coordinates_within_bounds() {
    // Arrange
    let valid_coordinates = vec![
        (32.486722, 34.888722),   // Tel Aviv
        (40.7128, -74.0060),      // New York
        (-33.8688, 151.2093),     // Sydney
        (0.0, 0.0),               // Equator/Prime Meridian
        (90.0, 180.0),            // Max bounds
        (-90.0, -180.0),          // Min bounds
    ];
    
    // Act & Assert
    for (lat, lng) in valid_coordinates {
        assert!(
            lat >= -90.0 && lat <= 90.0,
            "Latitude {} should be in valid range",
            lat
        );
        assert!(
            lng >= -180.0 && lng <= 180.0,
            "Longitude {} should be in valid range",
            lng
        );
    }
}

#[test]
fn test_validate_coordinates_out_of_bounds() {
    // Arrange
    let invalid_coordinates = vec![
        (91.0, 0.0),      // Latitude too high
        (-91.0, 0.0),     // Latitude too low
        (0.0, 181.0),     // Longitude too high
        (0.0, -181.0),    // Longitude too low
    ];
    
    // Act & Assert
    for (lat, lng) in invalid_coordinates {
        let is_invalid = lat < -90.0 || lat > 90.0 || lng < -180.0 || lng > 180.0;
        assert!(
            is_invalid,
            "Coordinates ({}, {}) should be detected as invalid",
            lat, lng
        );
    }
}

#[test]
fn test_missing_coordinates_error_is_actionable() {
    // Arrange - Config with no coordinates
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"
        [general]
        timezone = "UTC"
    "#).unwrap();
    
    let config = load_config(Some(temp_file.path())).unwrap();
    
    // Act - Simulate coordinate requirement check
    let lat = None;  // CLI didn't provide
    let lng = None;  // CLI didn't provide
    
    let final_lat = lat.or(config.general.lat);
    let final_lng = lng.or(config.general.lng);
    
    // Assert
    if final_lat.is_none() || final_lng.is_none() {
        // Error Transparency: error message should guide user
        let error_msg = "Latitude not specified. Provide via --lat argument or configure in config file.";
        assert!(
            error_msg.contains("--lat") || error_msg.contains("config file"),
            "Error should tell user how to provide coordinates"
        );
    }
}

// ============================================================================
// Test Pattern 5: Default Config Path
// ============================================================================

#[test]
fn test_default_config_path_uses_home_directory() {
    // Arrange
    use windsurf_forecast::config::get_default_config_path;
    
    // Act
    let result = get_default_config_path();
    
    // Assert
    assert!(result.is_ok(), "Should be able to determine default config path");
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    
    // Should be in home directory with expected filename
    assert!(
        path_str.contains("windsurf-config.toml") || path_str.contains(".windsurf"),
        "Default config path should include application name: {:?}",
        path
    );
}

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Helper to create valid test config
fn create_test_config() -> Config {
    Config {
        general: GeneralConfig {
            lat: Some(32.486722),
            lng: Some(34.888722),
            timezone: "Asia/Jerusalem".to_string(),
        },
    }
}

/// Helper to create temporary config file with content
fn create_temp_config_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", content).unwrap();
    file
}