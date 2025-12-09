// ============================================================================
// CLI Argument Validation Tests (User Story 1)
// ============================================================================
//
// Tests for command-line argument parsing and validation logic.
// Covers valid combinations, boundary conditions, and error cases.

use windsurf_forecast::args::{validate_args, validate_provider};
use windsurf_forecast::test_utils::*;

#[test]
fn test_valid_argument_combinations() {
    let args = create_valid_args();
    assert!(validate_args(&args).is_ok());

    let args = create_args_with_coordinates(40.7128, -74.0060);
    assert!(validate_args(&args).is_ok());

    let args = create_args_with_provider("openweathermap");
    assert!(validate_args(&args).is_ok());

    let args = create_args_with_timezone("America/New_York");
    assert!(validate_args(&args).is_ok());
}

#[test]
fn test_boundary_condition_days_ahead_plus_offset_equals_7() {
    // Maximum allowed: days_ahead + first_day_offset = 7
    let args = create_args_with_days(4, 3);
    assert!(validate_args(&args).is_ok(), "4 + 3 = 7 should be valid");

    let args = create_args_with_days(7, 0);
    assert!(validate_args(&args).is_ok(), "7 + 0 = 7 should be valid");

    let args = create_args_with_days(1, 6);
    assert!(validate_args(&args).is_ok(), "1 + 6 = 7 should be valid");
}

#[test]
fn test_constraint_violation_exceeds_7_days() {
    let args = create_args_with_days(5, 3);
    let result = validate_args(&args);
    assert!(result.is_err(), "5 + 3 = 8 should fail");
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("exceeds maximum of 7"));
    assert!(err_msg.contains("5"));
    assert!(err_msg.contains("3"));
    assert!(err_msg.contains("8"));

    let args = create_args_with_days(7, 1);
    let result = validate_args(&args);
    assert!(result.is_err(), "7 + 1 = 8 should fail");
}

#[test]
fn test_days_ahead_zero_returns_error() {
    let args = create_args_with_days(0, 0);
    let result = validate_args(&args);
    assert!(result.is_err(), "days_ahead = 0 should fail");
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("days-ahead must be between 1 and 7"));
    assert!(err_msg.contains("0"));
}

#[test]
fn test_days_ahead_negative_returns_error() {
    let args = create_args_with_days(-1, 0);
    let result = validate_args(&args);
    assert!(result.is_err(), "days_ahead = -1 should fail");
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("days-ahead must be between 1 and 7"));
}

#[test]
fn test_days_ahead_exceeds_7_returns_error() {
    let args = create_args_with_days(8, 0);
    let result = validate_args(&args);
    assert!(result.is_err(), "days_ahead = 8 should fail");
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("days-ahead must be between 1 and 7"));
    assert!(err_msg.contains("8"));
}

#[test]
fn test_first_day_offset_negative_returns_error() {
    let args = create_args_with_days(4, -1);
    let result = validate_args(&args);
    assert!(result.is_err(), "first_day_offset = -1 should fail");
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("first-day-offset must be between 0 and 7"));
}

#[test]
fn test_first_day_offset_exceeds_7_returns_error() {
    let args = create_args_with_days(1, 8);
    let result = validate_args(&args);
    assert!(result.is_err(), "first_day_offset = 8 should fail");
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("first-day-offset must be between 0 and 7"));
}

#[test]
fn test_unknown_provider_returns_error() {
    let result = validate_provider("nonexistent_provider");
    assert!(result.is_err(), "Unknown provider should fail");
    
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Unknown provider") || err_msg.contains("nonexistent_provider"),
        "Error should mention provider name or 'Unknown provider'"
    );
}

#[test]
fn test_known_providers_validate_successfully() {
    assert!(validate_provider("stormglass").is_ok());
    assert!(validate_provider("openweathermap").is_ok());
}

// ============================================================================
// Coordinate Validation Tests
// ============================================================================
// Note: Coordinate validation happens in config layer, not args validation
// These tests document that args layer accepts coordinates without validation

#[test]
fn test_args_accepts_valid_latitude_range() {
    let args = create_args_with_coordinates(90.0, 0.0);
    assert!(validate_args(&args).is_ok());

    let args = create_args_with_coordinates(-90.0, 0.0);
    assert!(validate_args(&args).is_ok());

    let args = create_args_with_coordinates(0.0, 0.0);
    assert!(validate_args(&args).is_ok());
}

#[test]
fn test_args_accepts_valid_longitude_range() {
    let args = create_args_with_coordinates(0.0, 180.0);
    assert!(validate_args(&args).is_ok());

    let args = create_args_with_coordinates(0.0, -180.0);
    assert!(validate_args(&args).is_ok());

    let args = create_args_with_coordinates(0.0, 0.0);
    assert!(validate_args(&args).is_ok());
}

// Note: Out-of-range coordinates are validated in the config layer
// Args layer just stores the values
#[test]
fn test_args_stores_coordinates_without_range_validation() {
    // This is intentional - validation happens in config layer
    let args = create_args_with_coordinates(100.0, 200.0);
    assert!(validate_args(&args).is_ok(), "Args layer accepts any coordinates");
}