// ============================================================================
// Common Test Helpers and Mock Data Builders
// ============================================================================
//
// This module provides test utilities and mock data structures for testing
// the windsurf-forecast application. It includes:
// - Helper functions for creating valid Args structures
// - Mock API response builders for StormGlass and OpenWeatherMap
// - Temporary config file helpers
// - Common test data and constants

use serde_json::json;
use windsurf_forecast::args::Args;

// ============================================================================
// Args Test Helpers
// ============================================================================

/// Create a valid Args structure with default values for testing
pub fn create_valid_args() -> Args {
    Args {
        days_ahead: 4,
        first_day_offset: 0,
        provider: "stormglass".to_string(),
        timezone: Some("UTC".to_string()),
        pick_timezone: false,
        list_providers: false,
        config: None,
        lat: Some(32.486722),
        lng: Some(34.888722),
    }
}

/// Create Args with custom days_ahead and first_day_offset
pub fn create_args_with_days(days_ahead: i32, first_day_offset: i32) -> Args {
    Args {
        days_ahead,
        first_day_offset,
        ..create_valid_args()
    }
}

/// Create Args with custom provider
pub fn create_args_with_provider(provider: &str) -> Args {
    Args {
        provider: provider.to_string(),
        ..create_valid_args()
    }
}

/// Create Args with custom coordinates
pub fn create_args_with_coordinates(lat: f64, lng: f64) -> Args {
    Args {
        lat: Some(lat),
        lng: Some(lng),
        ..create_valid_args()
    }
}

/// Create Args with custom timezone
pub fn create_args_with_timezone(timezone: &str) -> Args {
    Args {
        timezone: Some(timezone.to_string()),
        ..create_valid_args()
    }
}

// ============================================================================
// StormGlass Mock Response Builders
// ============================================================================

/// Create a complete StormGlass API response with all fields populated
pub fn mock_stormglass_complete_response() -> serde_json::Value {
    json!({
        "hours": [
            {
                "time": "2025-12-07T12:00:00+00:00",
                "airTemperature": { "sg": 22.5 },
                "windSpeed": { "sg": 5.2 },
                "windDirection": { "sg": 270.0 },
                "gust": { "sg": 7.8 },
                "swellHeight": { "sg": 1.2 },
                "swellPeriod": { "sg": 8.5 },
                "swellDirection": { "sg": 310.0 },
                "waterTemperature": { "sg": 20.1 }
            },
            {
                "time": "2025-12-07T13:00:00+00:00",
                "airTemperature": { "sg": 23.0 },
                "windSpeed": { "sg": 6.1 },
                "windDirection": { "sg": 280.0 },
                "gust": { "sg": 8.2 },
                "swellHeight": { "sg": 1.3 },
                "swellPeriod": { "sg": 8.7 },
                "swellDirection": { "sg": 315.0 },
                "waterTemperature": { "sg": 20.2 }
            }
        ]
    })
}

/// Create a minimal StormGlass API response with only required fields
pub fn mock_stormglass_minimal_response() -> serde_json::Value {
    json!({
        "hours": [
            {
                "time": "2025-12-07T12:00:00+00:00"
            }
        ]
    })
}

/// Create a StormGlass response with some optional fields missing (None)
pub fn mock_stormglass_partial_response() -> serde_json::Value {
    json!({
        "hours": [
            {
                "time": "2025-12-07T12:00:00+00:00",
                "airTemperature": { "sg": 22.5 },
                "windSpeed": { "sg": 5.2 },
                "windDirection": { "sg": 270.0 }
                // Note: Missing gust, swell data, and water temperature
            }
        ]
    })
}

// ============================================================================
// OpenWeatherMap Mock Response Builders
// ============================================================================

/// Create a complete OpenWeatherMap API response with all fields populated
pub fn mock_openweathermap_complete_response() -> serde_json::Value {
    json!({
        "list": [
            {
                "dt": 1701950400, // 2023-12-07 12:00:00 UTC
                "main": {
                    "temp": 22.5
                },
                "wind": {
                    "speed": 5.2,
                    "deg": 270.0,
                    "gust": 7.8
                },
                "timezone": 0
            },
            {
                "dt": 1701954000, // 2023-12-07 13:00:00 UTC
                "main": {
                    "temp": 23.0
                },
                "wind": {
                    "speed": 6.1,
                    "deg": 280.0,
                    "gust": 8.2
                },
                "timezone": 0
            }
        ],
        "city": {
            "name": "Tel Aviv",
            "coord": {
                "lat": 32.486722,
                "lon": 34.888722
            }
        }
    })
}

/// Create a minimal OpenWeatherMap API response with only required fields
pub fn mock_openweathermap_minimal_response() -> serde_json::Value {
    json!({
        "list": [
            {
                "dt": 1701950400,
                "main": {
                    "temp": 22.5
                },
                "wind": {
                    "speed": 5.2,
                    "deg": 270.0
                },
                "timezone": 0
            }
        ]
    })
}

/// Create an OpenWeatherMap response without gust field
pub fn mock_openweathermap_no_gust() -> serde_json::Value {
    json!({
        "list": [
            {
                "dt": 1701950400,
                "main": {
                    "temp": 22.5
                },
                "wind": {
                    "speed": 5.2,
                    "deg": 270.0
                    // Note: No gust field
                },
                "timezone": 0
            }
        ]
    })
}

// ============================================================================
// Test Constants
// ============================================================================

/// Wind speed conversion constant (m/s to knots) - StormGlass specific
pub const MS_TO_KNOTS: f64 = 1.94384;

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that a wind speed has been correctly converted from m/s to knots
pub fn assert_wind_speed_converted(input_ms: f64, output_knots: f64) {
    let expected = input_ms * MS_TO_KNOTS;
    let tolerance = 0.0001;
    assert!(
        (output_knots - expected).abs() < tolerance,
        "Wind speed conversion failed: input {} m/s should convert to {} knots, got {}",
        input_ms,
        expected,
        output_knots
    );
}

/// Assert that a timestamp string matches the expected format "YYYY-MM-DD HH:MM"
pub fn assert_timestamp_format(timestamp: &str) -> bool {
    // Expected format: "YYYY-MM-DD HH:MM" (19 characters)
    if timestamp.len() != 16 {
        return false;
    }
    
    // Check structure: YYYY-MM-DD HH:MM
    let parts: Vec<&str> = timestamp.split(' ').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();
    
    date_parts.len() == 3 && time_parts.len() == 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_args() {
        let args = create_valid_args();
        assert_eq!(args.days_ahead, 4);
        assert_eq!(args.first_day_offset, 0);
        assert_eq!(args.provider, "stormglass");
        assert_eq!(args.lat, Some(32.486722));
        assert_eq!(args.lng, Some(34.888722));
    }

    #[test]
    fn test_wind_speed_conversion_helper() {
        assert_wind_speed_converted(5.0, 5.0 * MS_TO_KNOTS);
        assert_wind_speed_converted(10.0, 10.0 * MS_TO_KNOTS);
    }

    #[test]
    fn test_timestamp_format_validation() {
        assert!(assert_timestamp_format("2025-12-07 14:30"));
        assert!(!assert_timestamp_format("2025-12-07T14:30:00Z"));
        assert!(!assert_timestamp_format("2025-12-07"));
    }
}