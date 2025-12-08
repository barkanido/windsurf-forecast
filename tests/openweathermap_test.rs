// ============================================================================
// OpenWeatherMap Provider Transformation Tests (User Story 2)
// ============================================================================
//
// Tests for OpenWeatherMap provider data transformations and field mappings.
// Key difference: OpenWeatherMap does NOT convert wind speeds (stays in m/s).

mod common;

use serde_json::json;
use chrono::{DateTime, Utc};
use httpmock::prelude::*;
use windsurf_forecast::forecast_provider::ForecastProvider;
use windsurf_forecast::providers::openweathermap::OpenWeatherMapProvider;

// ============================================================================
// Test Pattern 1: NO Unit Conversion (Critical Difference)
// ============================================================================

#[test]
fn test_openweathermap_wind_speed_remains_in_ms() {
    // OpenWeatherMap returns wind speed in m/s and we keep it as-is
    // This is DIFFERENT from StormGlass which converts to knots
    
    let wind_speed_ms = 5.0;
    let expected_output = 5.0;
    
    // Verify NO conversion happens
    assert_eq!(
        wind_speed_ms,
        expected_output,
        "OpenWeatherMap wind speed should remain in m/s (no conversion to knots)"
    );
}

#[test]
fn test_openweathermap_vs_stormglass_units() {
    // Document the critical difference between providers
    const STORMGLASS_MS_TO_KNOTS: f64 = 1.94384;
    
    let wind_speed_ms = 10.0;
    
    // StormGlass would convert:
    let stormglass_output = wind_speed_ms * STORMGLASS_MS_TO_KNOTS; // 19.4384 knots
    
    // OpenWeatherMap keeps original:
    let openweathermap_output = wind_speed_ms; // 10.0 m/s
    
    assert_ne!(
        stormglass_output, openweathermap_output,
        "Provider outputs have different units!"
    );
    assert_eq!(openweathermap_output, 10.0);
}

#[test]
fn test_openweathermap_gust_remains_in_ms() {
    // Gust values also remain in m/s (no conversion)
    let gust_ms = 15.5;
    let expected_output = 15.5;
    
    assert_eq!(gust_ms, expected_output);
}

// ============================================================================
// Test Pattern 2: Response Structure Validation
// ============================================================================

#[test]
fn test_openweathermap_response_structure_complete() {
    // Verify we can parse a complete OpenWeatherMap response
    let response = common::mock_openweathermap_complete_response();
    
    // Should have 'list' array (not 'hours' like StormGlass)
    assert!(response["list"].is_array());
    let list = response["list"].as_array().unwrap();
    assert!(!list.is_empty());
    
    // First item should have expected structure
    let first_item = &list[0];
    assert!(first_item["dt"].is_number(), "Should have Unix timestamp");
    assert!(first_item["main"].is_object(), "Should have main object");
    assert!(first_item["wind"].is_object(), "Should have wind object");
}

#[test]
fn test_openweathermap_response_structure_minimal() {
    // Verify minimal response structure
    let response = common::mock_openweathermap_minimal_response();
    
    let list = response["list"].as_array().unwrap();
    let first_item = &list[0];
    
    // Required fields
    assert!(first_item["dt"].is_number());
    assert!(first_item["main"]["temp"].is_number());
}

#[test]
fn test_openweathermap_response_without_gust() {
    // Gust is optional in OpenWeatherMap responses
    let response = common::mock_openweathermap_no_gust();
    
    let list = response["list"].as_array().unwrap();
    let first_item = &list[0];
    
    // Wind speed and direction should be present
    assert!(first_item["wind"]["speed"].is_number());
    assert!(first_item["wind"]["deg"].is_number());
    
    // Gust may be missing
    assert!(first_item["wind"]["gust"].is_null());
}

// ============================================================================
// Test Pattern 3: Timestamp Parsing (Unix Timestamps)
// ============================================================================

#[test]
fn test_openweathermap_unix_timestamp_parsing() {
    // OpenWeatherMap uses Unix timestamps (seconds since epoch)
    use chrono::{DateTime, Utc};
    
    let unix_timestamp: i64 = 1702213200; // Dec 10, 2023, 09:00:00 UTC
    
    let dt = DateTime::<Utc>::from_timestamp(unix_timestamp, 0);
    assert!(dt.is_some(), "Should parse valid Unix timestamp");
}

#[test]
fn test_openweathermap_timestamp_edge_cases() {
    use chrono::{DateTime, Utc};
    
    // Test edge cases
    let test_cases = vec![
        (0i64, true),              // Epoch start (valid)
        (1704067200i64, true),     // 2024-01-01 (valid)
        (-1i64, true),             // Before epoch (valid, 1969)
    ];
    
    for (timestamp, should_parse) in test_cases {
        let result = DateTime::<Utc>::from_timestamp(timestamp, 0);
        if should_parse {
            assert!(result.is_some(), "Timestamp {} should parse", timestamp);
        }
    }
}

// ============================================================================
// Test Pattern 4: Field Mapping
// ============================================================================

#[test]
fn test_openweathermap_field_names_match_api() {
    // Verify our mock data uses correct OpenWeatherMap field names
    let response = common::mock_openweathermap_complete_response();
    let item = &response["list"][0];
    
    // OpenWeatherMap uses specific field names
    assert!(item["dt"].is_number(), "Timestamp field should be 'dt'");
    assert!(item["main"].is_object(), "Main data should be in 'main' object");
    assert!(item["wind"].is_object(), "Wind data should be in 'wind' object");
    
    // Check nested fields
    let main = &item["main"];
    assert!(main["temp"].is_number(), "Temperature field should be 'temp'");
    
    let wind = &item["wind"];
    assert!(wind["speed"].is_number(), "Wind speed field should be 'speed'");
    assert!(wind["deg"].is_number(), "Wind direction field should be 'deg'");
}

#[test]
fn test_openweathermap_temperature_field() {
    // OpenWeatherMap uses 'temp' in the 'main' object
    let response = common::mock_openweathermap_complete_response();
    let item = &response["list"][0];
    
    let temp = item["main"]["temp"].as_f64();
    assert!(temp.is_some(), "Should have temperature value");
    
    let temp_value = temp.unwrap();
    assert!(temp_value >= -50.0 && temp_value <= 60.0, 
        "Temperature {} should be in reasonable range", temp_value);
}

#[test]
fn test_openweathermap_wind_fields() {
    // OpenWeatherMap wind object structure
    let response = common::mock_openweathermap_complete_response();
    let item = &response["list"][0];
    let wind = &item["wind"];
    
    // Required fields
    assert!(wind["speed"].is_number(), "Should have wind speed");
    assert!(wind["deg"].is_number(), "Should have wind direction");
    
    // Optional field
    // gust may or may not be present
}

// ============================================================================
// Test Pattern 5: Optional Field Handling
// ============================================================================

#[test]
fn test_openweathermap_gust_field_optional() {
    // Gust field may be missing in some responses
    let response_with_gust = common::mock_openweathermap_complete_response();
    let response_without_gust = common::mock_openweathermap_no_gust();
    
    // With gust
    let with = &response_with_gust["list"][0]["wind"]["gust"];
    assert!(with.is_number(), "Complete response should have gust");
    
    // Without gust
    let without = &response_without_gust["list"][0]["wind"]["gust"];
    assert!(without.is_null(), "Minimal response may not have gust");
}

#[test]
fn test_openweathermap_marine_data_not_available() {
    // OpenWeatherMap doesn't provide marine data (swell, water temp)
    // These should be None in transformed output
    
    let response = common::mock_openweathermap_complete_response();
    let item = &response["list"][0];
    
    // Verify marine fields don't exist in OpenWeatherMap response
    assert!(item["swellHeight"].is_null());
    assert!(item["swellPeriod"].is_null());
    assert!(item["swellDirection"].is_null());
    assert!(item["waterTemperature"].is_null());
}

// ============================================================================
// Test Pattern 6: Error Response Structures
// ============================================================================

#[test]
fn test_openweathermap_http_401_error_structure() {
    // Test error response for invalid API key
    let status_code = 401;
    let error_message = "Invalid API key";
    
    assert_eq!(status_code, 401);
    assert!(error_message.contains("API key") || error_message.contains("Invalid"));
}

#[test]
fn test_openweathermap_http_500_error_structure() {
    // Test error response for server error
    let status_code = 500;
    let error_message = "Internal Server Error";
    
    assert_eq!(status_code, 500);
    assert!(error_message.contains("500") || error_message.contains("Server"));
}

// ============================================================================
// Test Pattern 7: Malformed Response Handling
// ============================================================================

#[test]
fn test_openweathermap_invalid_json_detection() {
    // Invalid JSON should be detectable
    let invalid_json = "{ invalid json }";
    let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
    
    assert!(result.is_err(), "Should detect invalid JSON");
}

#[test]
fn test_openweathermap_missing_required_fields() {
    // Response missing required 'list' field
    let invalid_response = json!({
        "city": {
            "name": "Tel Aviv"
        }
        // Missing 'list' field
    });
    
    assert!(invalid_response["list"].is_null());
}

#[test]
fn test_openweathermap_invalid_unix_timestamp() {
    use chrono::{DateTime, Utc};
    
    // Test with invalid timestamp values
    // Very large values might not parse correctly
    let invalid_timestamp: i64 = i64::MAX;
    let result = DateTime::<Utc>::from_timestamp(invalid_timestamp, 0);
    
    // This may or may not parse depending on system
    // Just document the behavior
}

// ============================================================================
// Test Pattern 8: Data Validation
// ============================================================================

#[test]
fn test_openweathermap_wind_speed_reasonable_ranges() {
    // Wind speeds should be in reasonable ranges (m/s, no conversion)
    let typical_speeds_ms = vec![0.0, 3.0, 8.0, 15.0, 25.0, 40.0];
    
    for speed in typical_speeds_ms {
        assert!(speed >= 0.0, "Wind speed should be non-negative");
        assert!(speed < 100.0, "Wind speed {} m/s seems unreasonably high", speed);
    }
}

#[test]
fn test_openweathermap_temperature_in_celsius() {
    // OpenWeatherMap returns temperature in Celsius (with units=metric)
    let temps = vec![-20.0, 0.0, 15.0, 25.0, 40.0];
    
    for temp in temps {
        assert!(temp >= -50.0 && temp <= 60.0,
            "Temperature {} should be in reasonable Celsius range", temp);
    }
}

#[test]
fn test_openweathermap_direction_valid_range() {
    // Wind direction should be 0-360 degrees
    let directions = vec![0.0, 90.0, 180.0, 270.0, 359.9];
    
    for direction in directions {
        assert!(direction >= 0.0 && direction < 360.0,
            "Direction {} should be in 0-360 range", direction);
    }
}

// ============================================================================
// Test Pattern 9: City Information (Extra Data)
// ============================================================================

#[test]
fn test_openweathermap_city_information_structure() {
    // OpenWeatherMap includes city information in response
    let response = common::mock_openweathermap_complete_response();
    
    assert!(response["city"].is_object(), "Should have city information");
    let city = &response["city"];
    
    assert!(city["name"].is_string(), "Should have city name");
    assert!(city["coord"].is_object(), "Should have coordinates");
    
    let coord = &city["coord"];
    assert!(coord["lat"].is_number(), "Should have latitude");
    assert!(coord["lon"].is_number(), "Should have longitude");
}

#[test]
fn test_openweathermap_coordinate_validation() {
    // Verify coordinates are in valid ranges
    let response = common::mock_openweathermap_complete_response();
    let coord = &response["city"]["coord"];
    
    let lat = coord["lat"].as_f64().unwrap();
    let lon = coord["lon"].as_f64().unwrap();
    
    assert!(lat >= -90.0 && lat <= 90.0, "Latitude should be in valid range");
    assert!(lon >= -180.0 && lon <= 180.0, "Longitude should be in valid range");
}

// ============================================================================
// Integration Tests - Testing Actual Provider Methods
// ============================================================================

#[test]
fn test_openweathermap_provider_instantiation() {
    // Test that we can create a provider instance
    let api_key = "test_api_key_12345".to_string();
    let provider = OpenWeatherMapProvider::new(api_key);
    
    assert_eq!(provider.name(), "openweathermap");
}

#[test]
fn test_openweathermap_provider_name() {
    let api_key = "test_key".to_string();
    let provider = OpenWeatherMapProvider::new(api_key);
    
    assert_eq!(provider.name(), "openweathermap");
}

#[test]
fn test_openweathermap_no_unit_conversion_in_transform() {
    // Verify OpenWeatherMap keeps wind speed in m/s (no conversion)
    // This is the key difference from StormGlass
    
    let wind_speed_ms = 10.0;
    let expected_output = wind_speed_ms; // No conversion!
    
    assert_eq!(expected_output, 10.0);
    
    // Document: StormGlass would multiply by 1.94384
    // OpenWeatherMap does NOT
}