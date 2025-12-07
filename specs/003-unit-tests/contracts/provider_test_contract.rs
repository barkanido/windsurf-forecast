// ============================================================================
// Provider Transformation Test Contract
// ============================================================================
// This file demonstrates the expected structure for testing provider data
// transformations with mocked HTTP responses. These are example patterns.

use httpmock::prelude::*;
use serde_json::json;
use chrono_tz::Tz;
use windsurf_forecast::providers::stormglass::StormGlassProvider;
use windsurf_forecast::providers::openweathermap::OpenWeatherMapProvider;
use windsurf_forecast::forecast_provider::{ForecastProvider, UtcTimestamp, convert_timezone};

// ============================================================================
// Test Pattern 1: StormGlass Unit Conversion (m/s → knots)
// ============================================================================

#[tokio::test]
async fn test_stormglass_converts_wind_speed_ms_to_knots() {
    // Arrange
    let server = MockServer::start();
    
    // Mock API response with wind speed in m/s
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/weather/point");
        then.status(200)
            .json_body(json!({
                "hours": [{
                    "time": "2025-12-07T12:00:00+00:00",
                    "windSpeed": {
                        "sg": 5.0  // 5.0 m/s
                    }
                }],
                "meta": {
                    "dailyQuota": 50,
                    "requestCount": 10
                }
            }));
    });
    
    let provider = StormGlassProvider::new_with_url(
        "test_api_key".to_string(),
        server.base_url()
    );
    
    let start = Utc::now();
    let end = start + Duration::hours(1);
    let tz: Tz = "UTC".parse().unwrap();
    
    // Act
    let result = provider.fetch_weather_data(start, end, 32.48, 34.89, tz).await;
    
    // Assert
    assert!(result.is_ok());
    let data_points = result.unwrap();
    assert_eq!(data_points.len(), 1);
    
    // Verify conversion: 5.0 m/s × 1.94384 = 9.7192 knots
    let wind_speed = data_points[0].wind_speed.unwrap();
    assert!(
        (wind_speed - 9.7192).abs() < 0.001,
        "Wind speed should be converted from m/s to knots using factor 1.94384"
    );
    
    mock.assert();
}

#[tokio::test]
async fn test_stormglass_handles_missing_optional_fields() {
    // Arrange
    let server = MockServer::start();
    
    // Mock response with only required time field, no optional data
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/weather/point");
        then.status(200)
            .json_body(json!({
                "hours": [{
                    "time": "2025-12-07T12:00:00+00:00"
                    // No windSpeed, no airTemperature, etc.
                }],
                "meta": {
                    "dailyQuota": 50,
                    "requestCount": 10
                }
            }));
    });
    
    let provider = StormGlassProvider::new_with_url(
        "test_api_key".to_string(),
        server.base_url()
    );
    
    let start = Utc::now();
    let end = start + Duration::hours(1);
    let tz: Tz = "UTC".parse().unwrap();
    
    // Act
    let result = provider.fetch_weather_data(start, end, 32.48, 34.89, tz).await;
    
    // Assert
    assert!(result.is_ok());
    let data_points = result.unwrap();
    assert_eq!(data_points.len(), 1);
    
    // All optional fields should be None
    assert!(data_points[0].wind_speed.is_none());
    assert!(data_points[0].air_temperature.is_none());
    assert!(data_points[0].gust.is_none());
    
    mock.assert();
}

// ============================================================================
// Test Pattern 2: OpenWeatherMap No Conversion (m/s stays m/s)
// ============================================================================

#[tokio::test]
async fn test_openweathermap_wind_speed_remains_in_ms() {
    // Arrange
    let server = MockServer::start();
    
    // Mock API response with wind speed in m/s
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/data/2.5/forecast");
        then.status(200)
            .json_body(json!({
                "list": [{
                    "dt": 1702213200,  // Unix timestamp
                    "main": {
                        "temp": 20.5
                    },
                    "wind": {
                        "speed": 5.0,  // 5.0 m/s
                        "deg": 180.0
                    }
                }],
                "city": {
                    "name": "Tel Aviv",
                    "coord": {
                        "lat": 32.48,
                        "lon": 34.89
                    }
                }
            }));
    });
    
    let provider = OpenWeatherMapProvider::new_with_url(
        "test_api_key".to_string(),
        server.base_url()
    );
    
    let start = Utc::now();
    let end = start + Duration::hours(1);
    let tz: Tz = "UTC".parse().unwrap();
    
    // Act
    let result = provider.fetch_weather_data(start, end, 32.48, 34.89, tz).await;
    
    // Assert
    assert!(result.is_ok());
    let data_points = result.unwrap();
    assert_eq!(data_points.len(), 1);
    
    // Verify NO conversion: should remain 5.0 m/s
    let wind_speed = data_points[0].wind_speed.unwrap();
    assert!(
        (wind_speed - 5.0).abs() < 0.001,
        "OpenWeatherMap wind speed should remain in m/s (no conversion)"
    );
    
    mock.assert();
}

// ============================================================================
// Test Pattern 3: Timezone Conversion
// ============================================================================

#[tokio::test]
async fn test_provider_converts_utc_to_target_timezone() {
    // Arrange
    let server = MockServer::start();
    
    let mock = server.mock(|when, then| {
        when.method(GET);
        then.status(200)
            .json_body(json!({
                "hours": [{
                    "time": "2025-12-07T12:00:00+00:00"  // UTC noon
                }],
                "meta": {
                    "dailyQuota": 50,
                    "requestCount": 10
                }
            }));
    });
    
    let provider = StormGlassProvider::new_with_url(
        "test_api_key".to_string(),
        server.base_url()
    );
    
    let start = Utc::now();
    let end = start + Duration::hours(1);
    let tz: Tz = "Asia/Jerusalem".parse().unwrap();  // UTC+2
    
    // Act
    let result = provider.fetch_weather_data(start, end, 32.48, 34.89, tz).await;
    
    // Assert
    assert!(result.is_ok());
    let data_points = result.unwrap();
    
    // Verify timestamp format: "YYYY-MM-DD HH:MM" (not ISO 8601)
    let serialized = serde_json::to_string(&data_points[0]).unwrap();
    
    // Should contain "2025-12-07 14:00" (12:00 UTC + 2 hours = 14:00 Jerusalem)
    assert!(
        serialized.contains("2025-12-07 14:00"),
        "Timestamp should be converted to Asia/Jerusalem timezone and formatted as YYYY-MM-DD HH:MM"
    );
    
    mock.assert();
}

// ============================================================================
// Test Pattern 4: Error Handling
// ============================================================================

#[tokio::test]
async fn test_provider_handles_http_401_unauthorized() {
    // Arrange
    let server = MockServer::start();
    
    let mock = server.mock(|when, then| {
        when.method(GET);
        then.status(401)
            .body("Unauthorized");
    });
    
    let provider = StormGlassProvider::new_with_url(
        "invalid_api_key".to_string(),
        server.base_url()
    );
    
    let start = Utc::now();
    let end = start + Duration::hours(1);
    let tz: Tz = "UTC".parse().unwrap();
    
    // Act
    let result = provider.fetch_weather_data(start, end, 32.48, 34.89, tz).await;
    
    // Assert
    assert!(result.is_err(), "Should return error for 401 status");
    let error = result.unwrap_err();
    let error_msg = format!("{:#}", error);
    
    // Error Transparency: message should mention authentication
    assert!(
        error_msg.contains("401") || error_msg.contains("Unauthorized") || error_msg.contains("API key"),
        "Error message should indicate authentication failure"
    );
    
    mock.assert();
}

#[tokio::test]
async fn test_provider_handles_http_402_quota_exceeded() {
    // Arrange
    let server = MockServer::start();
    
    let mock = server.mock(|when, then| {
        when.method(GET);
        then.status(402)
            .body("Payment Required - Daily quota exceeded");
    });
    
    let provider = StormGlassProvider::new_with_url(
        "test_api_key".to_string(),
        server.base_url()
    );
    
    let start = Utc::now();
    let end = start + Duration::hours(1);
    let tz: Tz = "UTC".parse().unwrap();
    
    // Act
    let result = provider.fetch_weather_data(start, end, 32.48, 34.89, tz).await;
    
    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = format!("{:#}", error);
    
    // Error Transparency: message should explain quota issue
    assert!(
        error_msg.contains("402") || error_msg.contains("quota"),
        "Error message should indicate quota exceeded"
    );
    
    mock.assert();
}

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Helper to create complete mock StormGlass response
fn create_mock_stormglass_response() -> serde_json::Value {
    json!({
        "hours": [
            {
                "time": "2025-12-07T12:00:00+00:00",
                "airTemperature": { "sg": 22.5 },
                "windSpeed": { "sg": 5.0 },
                "windDirection": { "sg": 180.0 },
                "gust": { "sg": 7.5 },
                "swellHeight": { "sg": 1.2 },
                "swellPeriod": { "sg": 8.0 },
                "swellDirection": { "sg": 270.0 },
                "waterTemperature": { "sg": 18.0 }
            }
        ],
        "meta": {
            "dailyQuota": 50,
            "requestCount": 10
        }
    })
}

/// Helper to create mock OpenWeatherMap response
fn create_mock_openweathermap_response() -> serde_json::Value {
    json!({
        "list": [
            {
                "dt": 1702213200,
                "main": { "temp": 22.5 },
                "wind": {
                    "speed": 5.0,
                    "deg": 180.0,
                    "gust": 7.5
                },
                "weather": [
                    { "description": "clear sky" }
                ]
            }
        ],
        "city": {
            "name": "Tel Aviv",
            "coord": { "lat": 32.48, "lon": 34.89 }
        }
    })
}