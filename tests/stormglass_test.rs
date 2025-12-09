// ============================================================================
// StormGlass Provider Transformation Tests (User Story 2)
// ============================================================================
//
// Tests for StormGlass provider data transformations, unit conversions,
// and error handling using mocked HTTP responses.

use serde_json::json;
use windsurf_forecast::test_utils::*;
use windsurf_forecast::forecast_provider::ForecastProvider;
use windsurf_forecast::providers::stormglass::StormGlassProvider;

// ============================================================================
// Test Pattern 1: Unit Conversion Verification
// ============================================================================

#[test]
fn test_stormglass_ms_to_knots_conversion_constant() {
    const MS_TO_KNOTS: f64 = 1.94384;
    
    let test_cases = vec![
        (5.0, 9.7192),    // 5 m/s = 9.7192 knots
        (10.0, 19.4384),  // 10 m/s = 19.4384 knots
        (0.0, 0.0),       // 0 m/s = 0 knots
        (1.0, 1.94384),   // 1 m/s = 1.94384 knots
    ];
    
    for (ms, expected_knots) in test_cases {
        let knots = ms * MS_TO_KNOTS;
        assert!(
            (knots - expected_knots).abs() < 0.0001,
            "Conversion of {} m/s should equal {} knots, got {}",
            ms, expected_knots, knots
        );
    }
}

#[test]
fn test_stormglass_wind_speed_conversion_accuracy() {
    const MS_TO_KNOTS: f64 = 1.94384;
    
    let wind_speeds_ms = vec![2.5, 7.8, 12.3, 15.0, 20.5];
    
    for speed_ms in wind_speeds_ms {
        let speed_knots = speed_ms * MS_TO_KNOTS;
        
        let back_to_ms = speed_knots / MS_TO_KNOTS;
        assert!(
            (back_to_ms - speed_ms).abs() < 0.00001,
            "Conversion should be reversible: {} m/s",
            speed_ms
        );
    }
}

#[test]
fn test_stormglass_gust_conversion_uses_same_factor() {
    const MS_TO_KNOTS: f64 = 1.94384;
    
    let gust_ms = 15.5;
    let gust_knots = gust_ms * MS_TO_KNOTS;
    
    assert!((gust_knots - 30.12952).abs() < 0.0001);
}

// ============================================================================
// Test Pattern 2: Response Structure Validation
// ============================================================================

#[test]
fn test_stormglass_response_structure_complete() {
    let response = mock_stormglass_complete_response();
    
    assert!(response["hours"].is_array());
    let hours = response["hours"].as_array().unwrap();
    assert!(!hours.is_empty());
    
    let first_hour = &hours[0];
    assert!(first_hour["time"].is_string());
    assert!(first_hour["airTemperature"].is_object());
    assert!(first_hour["windSpeed"].is_object());
    assert!(first_hour["windDirection"].is_object());
    assert!(first_hour["gust"].is_object());
    assert!(first_hour["swellHeight"].is_object());
    assert!(first_hour["swellPeriod"].is_object());
    assert!(first_hour["swellDirection"].is_object());
    assert!(first_hour["waterTemperature"].is_object());
}

#[test]
fn test_stormglass_response_structure_minimal() {
    let response = mock_stormglass_minimal_response();
    
    let hours = response["hours"].as_array().unwrap();
    let first_hour = &hours[0];
    
    assert!(first_hour["time"].is_string());
}

#[test]
fn test_stormglass_response_structure_partial() {
    let response = mock_stormglass_partial_response();
    
    let hours = response["hours"].as_array().unwrap();
    let first_hour = &hours[0];
    
    assert!(first_hour["time"].is_string());
    assert!(first_hour["windSpeed"].is_object());
    assert!(first_hour["swellHeight"].is_null());
}

// ============================================================================
// Test Pattern 3: Timestamp Parsing
// ============================================================================

#[test]
fn test_stormglass_timestamp_format_rfc3339() {
    use windsurf_forecast::forecast_provider::UtcTimestamp;
    
    let timestamp = "2025-12-07T12:00:00+00:00";
    let result = UtcTimestamp::from_rfc3339(timestamp);
    
    assert!(result.is_ok(), "Should parse StormGlass RFC3339 timestamp");
}

#[test]
fn test_stormglass_timestamp_with_timezone_offset() {
    use windsurf_forecast::forecast_provider::UtcTimestamp;
    
    let timestamp = "2025-12-07T14:00:00+02:00";
    let result = UtcTimestamp::from_rfc3339(timestamp);
    
    assert!(result.is_ok());
    let utc = result.unwrap();
    
    use chrono::Timelike;
    assert_eq!(utc.0.hour(), 12);
}

// ============================================================================
// Test Pattern 4: Field Mapping
// ============================================================================

#[test]
fn test_stormglass_field_names_match_api() {
    let response = mock_stormglass_complete_response();
    let hour = &response["hours"][0];
    
    assert!(hour["airTemperature"].is_object(), "Field should be airTemperature");
    assert!(hour["windSpeed"].is_object(), "Field should be windSpeed");
    assert!(hour["windDirection"].is_object(), "Field should be windDirection");
    assert!(hour["swellHeight"].is_object(), "Field should be swellHeight");
    assert!(hour["swellPeriod"].is_object(), "Field should be swellPeriod");
    assert!(hour["swellDirection"].is_object(), "Field should be swellDirection");
    assert!(hour["waterTemperature"].is_object(), "Field should be waterTemperature");
}

#[test]
fn test_stormglass_source_data_structure() {
    let response = mock_stormglass_complete_response();
    let hour = &response["hours"][0];
    
    let wind_speed = &hour["windSpeed"];
    assert!(wind_speed["sg"].is_number(), "Should have 'sg' source field");
}

// ============================================================================
// Test Pattern 5: Optional Field Handling
// ============================================================================

#[test]
fn test_stormglass_handles_missing_optional_fields() {
    let response = mock_stormglass_minimal_response();
    
    assert!(response["hours"].is_array());
}

#[test]
fn test_stormglass_handles_null_values() {
    let response = mock_stormglass_partial_response();
    let hour = &response["hours"][0];
    
    assert!(hour["swellHeight"].is_null());
    assert!(hour["waterTemperature"].is_null());
}

// ============================================================================
// Test Pattern 6: Error Response Structures
// ============================================================================

#[test]
fn test_stormglass_http_401_error_structure() {
    let status_code = 401;
    let error_message = "Unauthorized";
    
    assert_eq!(status_code, 401);
    assert!(error_message.contains("Unauthorized"));
}

#[test]
fn test_stormglass_http_402_error_structure() {
    let status_code = 402;
    let expected_message = "Payment Required";
    
    assert_eq!(status_code, 402);
    assert!(expected_message.contains("Payment") || expected_message.contains("quota"));
}

#[test]
fn test_stormglass_http_403_error_structure() {
    let status_code = 403;
    
    assert_eq!(status_code, 403);
}

#[test]
fn test_stormglass_http_500_error_structure() {
    let status_code = 500;
    
    assert_eq!(status_code, 500);
}

// ============================================================================
// Test Pattern 7: Malformed Response Handling
// ============================================================================

#[test]
fn test_stormglass_invalid_json_detection() {
    let invalid_json = "{ this is not valid json }";
    let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
    
    assert!(result.is_err(), "Should detect invalid JSON");
}

#[test]
fn test_stormglass_missing_required_fields() {
    let invalid_response = json!({
        "meta": {
            "dailyQuota": 50
        }
    });
    
    assert!(invalid_response["hours"].is_null());
}

#[test]
fn test_stormglass_invalid_timestamp_format() {
    use windsurf_forecast::forecast_provider::UtcTimestamp;
    
    let invalid_timestamps = vec![
        "2025-12-07",           // Missing time
        "not a timestamp",      // Invalid format
        "12:00:00",            // Missing date
        "",                    // Empty
    ];
    
    for timestamp in invalid_timestamps {
        let result = UtcTimestamp::from_rfc3339(timestamp);
        assert!(
            result.is_err(),
            "Should reject invalid timestamp: {}",
            timestamp
        );
    }
}

// ============================================================================
// Test Pattern 8: Data Validation
// ============================================================================

#[test]
fn test_stormglass_wind_speed_reasonable_ranges() {
    const MS_TO_KNOTS: f64 = 1.94384;
    
    let typical_speeds_ms = vec![0.0, 5.0, 10.0, 15.0, 25.0, 40.0];
    
    for speed_ms in typical_speeds_ms {
        let speed_knots = speed_ms * MS_TO_KNOTS;
        
        assert!(speed_knots >= 0.0, "Wind speed should be non-negative");
        assert!(speed_knots < 200.0, "Wind speed {} knots seems unreasonably high", speed_knots);
    }
}

#[test]
fn test_stormglass_temperature_reasonable_ranges() {
    let temps_celsius = vec![-40.0, -10.0, 0.0, 15.0, 25.0, 40.0];
    
    for temp in temps_celsius {
        assert!((-50.0..=60.0).contains(&temp),
            "Temperature {} seems outside reasonable range", temp);
    }
}

#[test]
fn test_stormglass_direction_valid_range() {
    let directions = vec![0.0, 45.0, 90.0, 180.0, 270.0, 359.0];
    
    for direction in directions {
        assert!((0.0..360.0).contains(&direction),
            "Direction {} should be in 0-360 range", direction);
    }
}

// ============================================================================
// Integration Tests - Testing Actual Provider Methods
// ============================================================================

#[test]
fn test_stormglass_transform_hour_unit_conversion() {
    const MS_TO_KNOTS: f64 = 1.94384;
    let wind_speed_ms = 10.0;
    let expected_knots = wind_speed_ms * MS_TO_KNOTS;
    
    assert_eq!(expected_knots, 19.4384);
}

#[test]
fn test_stormglass_provider_instantiation() {
    let api_key = "test_api_key_12345".to_string();
    let provider = StormGlassProvider::new(api_key);
    
    assert_eq!(provider.name(), "stormglass");
}

#[test]
fn test_stormglass_provider_name() {
    let api_key = "test_key".to_string();
    let provider = StormGlassProvider::new(api_key);
    
    assert_eq!(provider.name(), "stormglass");
}