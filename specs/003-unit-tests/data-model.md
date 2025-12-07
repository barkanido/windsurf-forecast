# Data Model: Comprehensive Unit Test Coverage

**Feature**: 003-unit-tests  
**Date**: 2025-12-07  
**Status**: Complete

## Overview

This document defines the test data structures, mock entities, and test organization patterns for the comprehensive unit test coverage feature. Since this is a testing feature, the "data model" describes test fixtures, mock responses, and test case structures rather than production data entities.

## Test Case Organization

### Test Module Structure

```rust
// Pattern for test modules within source files
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_specific_behavior() {
        // Arrange, Act, Assert
    }
}

// Pattern for integration-style tests in tests/ directory
// tests/module_test.rs
use windsurf_forecast::module_name::*;

#[test]
fn test_integration_scenario() {
    // Test public API
}
```

### Test Case Naming Convention

- **Function**: `test_<function_name>_<scenario>_<expected_outcome>`
- **Examples**:
  - `test_validate_args_valid_range_returns_ok`
  - `test_validate_args_exceeds_7_days_returns_error`
  - `test_convert_timezone_utc_to_jerusalem_correct_offset`
  - `test_stormglass_transform_converts_ms_to_knots`

## Mock Data Entities

### 1. Mock API Response Structures

#### StormGlass Mock Response

```rust
/// Mock StormGlass API response structure for testing
#[derive(Debug, Serialize, Deserialize)]
struct MockStormGlassResponse {
    hours: Vec<MockStormGlassHour>,
    meta: MockStormGlassMeta,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockStormGlassHour {
    time: String,  // RFC3339 format: "2025-12-07T12:00:00+00:00"
    #[serde(rename = "airTemperature")]
    air_temperature: Option<MockStormGlassValue>,
    #[serde(rename = "windSpeed")]
    wind_speed: Option<MockStormGlassValue>,
    #[serde(rename = "windDirection")]
    wind_direction: Option<MockStormGlassValue>,
    gust: Option<MockStormGlassValue>,
    #[serde(rename = "swellHeight")]
    swell_height: Option<MockStormGlassValue>,
    #[serde(rename = "swellPeriod")]
    swell_period: Option<MockStormGlassValue>,
    #[serde(rename = "swellDirection")]
    swell_direction: Option<MockStormGlassValue>,
    #[serde(rename = "waterTemperature")]
    water_temperature: Option<MockStormGlassValue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockStormGlassValue {
    sg: f64,  // StormGlass value
}

#[derive(Debug, Serialize, Deserialize)]
struct MockStormGlassMeta {
    #[serde(rename = "dailyQuota")]
    daily_quota: u32,
    #[serde(rename = "requestCount")]
    request_count: u32,
}
```

**Test Fixtures**:
- `mock_stormglass_complete_response()` - All fields populated
- `mock_stormglass_minimal_response()` - Only required fields
- `mock_stormglass_missing_optional_fields()` - Some optional fields are None
- `mock_stormglass_single_hour()` - Single data point for unit tests

#### OpenWeatherMap Mock Response

```rust
/// Mock OpenWeatherMap API response structure for testing
#[derive(Debug, Serialize, Deserialize)]
struct MockOpenWeatherMapResponse {
    list: Vec<MockOpenWeatherMapItem>,
    city: MockOpenWeatherMapCity,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockOpenWeatherMapItem {
    dt: i64,  // Unix timestamp
    main: MockOpenWeatherMapMain,
    wind: MockOpenWeatherMapWind,
    weather: Vec<MockOpenWeatherMapWeather>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockOpenWeatherMapMain {
    temp: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockOpenWeatherMapWind {
    speed: f64,    // m/s
    deg: f64,
    gust: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockOpenWeatherMapWeather {
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockOpenWeatherMapCity {
    name: String,
    coord: MockCoordinates,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockCoordinates {
    lat: f64,
    lon: f64,
}
```

**Test Fixtures**:
- `mock_openweathermap_complete_response()` - All fields populated
- `mock_openweathermap_minimal_response()` - Only required fields
- `mock_openweathermap_no_gust()` - Wind gust not present
- `mock_openweathermap_single_item()` - Single data point

### 2. Test Configuration Entities

#### Mock Config File

```rust
/// Mock configuration for testing config file operations
#[derive(Debug, Serialize, Deserialize)]
struct MockConfig {
    general: MockGeneralConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockGeneralConfig {
    lat: Option<f64>,
    lng: Option<f64>,
    timezone: String,
}
```

**Test Fixtures**:
- `mock_config_complete()` - All fields populated
- `mock_config_no_coordinates()` - Missing lat/lng
- `mock_config_default_timezone()` - timezone = "UTC"
- `mock_config_custom_timezone()` - timezone = "Asia/Jerusalem"

#### Mock CLI Arguments

```rust
/// Mock CLI arguments for testing argument validation
struct MockArgs {
    provider: String,
    days_ahead: u32,
    first_day_offset: u32,
    lat: Option<f64>,
    lng: Option<f64>,
    timezone: Option<String>,
    config: Option<PathBuf>,
    list_providers: bool,
    pick_timezone: bool,
}
```

**Test Cases**:
- Valid combinations: provider + days_ahead + coordinates
- Boundary values: days_ahead + first_day_offset = 7 (max allowed)
- Invalid ranges: days_ahead + first_day_offset > 7
- Missing required: no coordinates (should error if not in config)
- Edge cases: days_ahead = 0, negative values

### 3. Test Assertion Helpers

#### Expected Weather Data Point

```rust
/// Expected output structure for testing transformations
struct ExpectedWeatherDataPoint {
    time: String,  // "YYYY-MM-DD HH:MM" format
    air_temperature: Option<f64>,
    wind_speed: Option<f64>,
    wind_direction: Option<f64>,
    gust: Option<f64>,
    swell_height: Option<f64>,
    swell_period: Option<f64>,
    swell_direction: Option<f64>,
    water_temperature: Option<f64>,
}
```

**Validation Functions**:
```rust
fn assert_weather_point_equals(actual: &WeatherDataPoint, expected: &ExpectedWeatherDataPoint);
fn assert_wind_speed_converted(input_ms: f64, output_knots: f64);
fn assert_timestamp_format(timestamp: &str) -> bool;
```

### 4. Error Test Cases

#### Expected Error Messages

```rust
/// Test cases for error message validation
struct ErrorTestCase {
    scenario: &'static str,
    trigger: Box<dyn Fn() -> Result<()>>,
    expected_message_contains: Vec<&'static str>,
}
```

**Error Scenarios**:
- Missing API key: message includes env var name
- Invalid timezone: message includes example formats
- Invalid coordinates: message includes valid ranges
- Exceeds date range: message explains 7-day limit
- Network failure: message suggests troubleshooting steps

## Test Data Relationships

### Provider → Transformation → Output

```
Mock API Response
    ↓
Provider Transform Logic (under test)
    ↓
WeatherDataPoint (validated output)
    ↓
JSON Serialization (format validation)
```

### CLI Args → Config → Merged Configuration

```
Mock CLI Args (--lat, --lng, --timezone)
    ↓
Mock Config File (lat, lng, timezone)
    ↓
Precedence Resolution (CLI > Config)
    ↓
Final Configuration (validated)
```

### UTC Timestamp → Timezone Conversion → Local Timestamp

```
UtcTimestamp("2025-12-07T12:00:00Z")
    ↓
convert_timezone(utc, Asia/Jerusalem)
    ↓
LocalTimestamp("2025-12-07 14:00")  // +02:00 offset
```

## Test Coverage Mapping

### Module → Test File → Coverage Target

| Source Module | Test File | Coverage Target | Key Test Cases |
|--------------|-----------|-----------------|----------------|
| `args.rs` | `tests/args_test.rs` | >80% lines | Validation rules, boundary conditions |
| `config.rs` | `tests/config_test.rs` | >80% lines | File loading, precedence, validation |
| `provider_registry.rs` | `tests/provider_registry_test.rs` | >80% lines | Discovery, instantiation, duplicates |
| `forecast_provider.rs` | `tests/timezone_test.rs` | >80% lines | Timezone conversion, newtype wrappers |
| `providers/stormglass.rs` | `tests/stormglass_test.rs` | 100% conversions | Unit conversion, field mapping |
| `providers/openweathermap.rs` | `tests/openweathermap_test.rs` | 100% conversions | Field mapping, no conversion |

## Validation Rules

### Field Value Ranges

- **Latitude**: -90.0 to 90.0
- **Longitude**: -180.0 to 180.0
- **Days Ahead**: 1 to 7
- **First Day Offset**: 0 to 6
- **Constraint**: days_ahead + first_day_offset ≤ 7
- **Wind Speed (StormGlass output)**: Should be in knots (original m/s × 1.94384)
- **Wind Speed (OpenWeatherMap output)**: Should remain in m/s (no conversion)
- **Timestamp Format**: "YYYY-MM-DD HH:MM" (not ISO 8601 with 'T' or 'Z')

### Required vs Optional Fields

**Required in WeatherDataPoint**:
- `time` (LocalTimestamp)

**Optional in WeatherDataPoint** (all can be None):
- `air_temperature`
- `wind_speed`
- `wind_direction`
- `gust`
- `swell_height`
- `swell_period`
- `swell_direction`
- `water_temperature`

## Test Isolation Patterns

### Environment Variable Tests

```rust
#[test]
#[serial]  // Ensures sequential execution
fn test_api_key_missing() {
    let original = std::env::var("STORMGLASS_API_KEY").ok();
    std::env::remove_var("STORMGLASS_API_KEY");
    
    let result = StormGlassProvider::get_api_key();
    assert!(result.is_err());
    
    // Restore original state
    if let Some(key) = original {
        std::env::set_var("STORMGLASS_API_KEY", key);
    }
}
```

### Temporary Config Files

```rust
#[test]
fn test_config_loading() {
    use tempfile::NamedTempFile;
    
    let temp_file = NamedTempFile::new().unwrap();
    let config_content = r#"
        [general]
        lat = 32.486722
        lng = 34.888722
        timezone = "Asia/Jerusalem"
    "#;
    std::fs::write(temp_file.path(), config_content).unwrap();
    
    let config = load_config(Some(temp_file.path())).unwrap();
    assert_eq!(config.general.timezone, "Asia/Jerusalem");
    
    // temp_file automatically deleted when dropped
}
```

### Mock HTTP Servers

```rust
#[tokio::test]
async fn test_provider_fetch() {
    use httpmock::prelude::*;
    
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.path("/weather/point")
            .query_param("lat", "32.486722");
        then.status(200)
            .json_body(mock_stormglass_complete_response());
    });
    
    // Test provider with mock server URL
    // ...
    
    mock.assert();  // Verify request was made
}
```

## Success Criteria Validation

Each test validates specific success criteria from the spec:

- **SC-001**: Coverage reports show >80% for core modules
- **SC-002**: Test suite passes on first run (no flakiness)
- **SC-003**: `time cargo test` completes in <10 seconds
- **SC-004**: Tests run with `cargo test` (no manual setup)
- **SC-005**: Test failures show clear diagnostic messages
- **SC-006**: Each test function validates one behavior
- **SC-007**: Tests catch >90% of regression scenarios
- **SC-008**: Provider tests verify 100% of conversions

## References

- Source modules in `src/` directory
- Test organization follows Rust conventions
- Mock data structures mirror actual API responses
- Validation rules enforce constitution principles