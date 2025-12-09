// ============================================================================
// Timezone Conversion Tests (User Story 1)
// ============================================================================
//
// Tests for timezone handling, conversion accuracy, and timestamp serialization.

use windsurf_forecast::config::timezone::TimezoneConfig;
use windsurf_forecast::test_utils::*;
use windsurf_forecast::forecast_provider::{convert_timezone, UtcTimestamp};
use chrono::Timelike;
use chrono_tz::Tz;

#[test]
fn test_utc_timestamp_from_rfc3339_valid_format() {
    let timestamp = "2025-12-07T12:00:00Z";
    let result = UtcTimestamp::from_rfc3339(timestamp);
    assert!(result.is_ok(), "Valid RFC3339 should parse successfully");
    
    let utc = result.unwrap();
    assert_eq!(utc.0.hour(), 12);
    assert_eq!(utc.0.minute(), 0);
}

#[test]
fn test_utc_timestamp_from_rfc3339_with_offset_normalizes_to_utc() {
    let timestamp = "2025-12-07T14:00:00+02:00";
    let result = UtcTimestamp::from_rfc3339(timestamp);
    assert!(result.is_ok());
    
    let utc = result.unwrap();
    assert_eq!(utc.0.hour(), 12);
    assert_eq!(utc.0.minute(), 0);
}

#[test]
fn test_utc_timestamp_from_rfc3339_rejects_invalid_format() {
    let invalid_formats = vec![
        "2025-12-07 12:00:00",        // Space instead of T
        "2025-12-07",                  // Date only
        "12:00:00",                    // Time only
        "not-a-timestamp",             // Random string
        "",                             // Empty string
    ];
    
    for invalid in invalid_formats {
        let result = UtcTimestamp::from_rfc3339(invalid);
        assert!(
            result.is_err(),
            "Invalid format '{}' should fail to parse",
            invalid
        );
    }
}

#[test]
fn test_convert_timezone_utc_to_jerusalem() {
    let utc = UtcTimestamp::from_rfc3339("2025-12-07T12:00:00Z").unwrap();
    let target_tz: Tz = "Asia/Jerusalem".parse().unwrap();
    
    let result = convert_timezone(utc, target_tz);
    assert!(result.is_ok());
    
    let local = result.unwrap();
    let serialized = serde_json::to_string(&local).unwrap();
    assert!(serialized.contains("14:00"), "Should show 14:00 in Jerusalem time");
}

#[test]
fn test_convert_timezone_utc_to_new_york() {
    let utc = UtcTimestamp::from_rfc3339("2025-01-15T12:00:00Z").unwrap();
    let target_tz: Tz = "America/New_York".parse().unwrap();
    
    let result = convert_timezone(utc, target_tz);
    assert!(result.is_ok());
    
    let local = result.unwrap();
    let serialized = serde_json::to_string(&local).unwrap();
    assert!(serialized.contains("07:00"), "Should show 07:00 in New York time");
}

#[test]
fn test_convert_timezone_utc_to_utc_preserves_time() {
    let utc = UtcTimestamp::from_rfc3339("2025-12-07T12:00:00Z").unwrap();
    let target_tz: Tz = "UTC".parse().unwrap();
    
    let result = convert_timezone(utc, target_tz);
    assert!(result.is_ok());
    
    let local = result.unwrap();
    let serialized = serde_json::to_string(&local).unwrap();
    assert!(serialized.contains("12:00"), "Time should remain 12:00 in UTC");
}

#[test]
fn test_local_timestamp_serialization_format() {
    let utc = UtcTimestamp::from_rfc3339("2025-12-07T14:30:00Z").unwrap();
    let target_tz: Tz = "UTC".parse().unwrap();
    let local = convert_timezone(utc, target_tz).unwrap();
    
    let serialized = serde_json::to_string(&local).unwrap();
    let timestamp = serialized.trim_matches('"');
    
    assert!(assert_timestamp_format(timestamp),
        "Timestamp should be in 'YYYY-MM-DD HH:MM' format, got: {}", timestamp);
    
    assert!(!timestamp.contains('T'), "Should not contain 'T' separator");
    assert!(!timestamp.contains('Z'), "Should not contain 'Z' timezone marker");
    assert!(!timestamp.contains('+'), "Should not contain '+' offset");
}

#[test]
fn test_weather_data_point_timestamp_serialization() {
    use windsurf_forecast::forecast_provider::WeatherDataPoint;
    
    let utc = UtcTimestamp::from_rfc3339("2025-12-07T12:00:00Z").unwrap();
    let target_tz: Tz = "Asia/Jerusalem".parse().unwrap();
    let local = convert_timezone(utc, target_tz).unwrap();
    
    let data_point = WeatherDataPoint {
        time: local,
        air_temperature: Some(22.5),
        wind_speed: Some(10.0),
        wind_direction: Some(270.0),
        gust: None,
        swell_height: None,
        swell_period: None,
        swell_direction: None,
        water_temperature: None,
    };
    
    let json = serde_json::to_string(&data_point).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    let time_str = parsed["time"].as_str().unwrap();
    assert!(assert_timestamp_format(time_str),
        "WeatherDataPoint timestamp should use correct format");
    assert!(time_str.contains("14:00"), "Should show Jerusalem time (UTC+2)");
}

#[test]
fn test_invalid_timezone_identifier_returns_error() {
    let invalid_timezones = vec![
        "Invalid/Timezone",
        "NotATimezone",
        "PST",  // PST abbreviation not accepted by chrono-tz
        "",
    ];
    
    for tz_str in invalid_timezones {
        let result: Result<Tz, _> = tz_str.parse();
        assert!(result.is_err(), "Invalid timezone '{}' should fail", tz_str);
    }
}

#[test]
fn test_timezone_config_from_string_provides_helpful_error() {
    use windsurf_forecast::config::timezone::TimezoneConfig;
    
    let result = TimezoneConfig::load_with_precedence(
        Some("InvalidTimezone"),
        None
    );
    
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    
    assert!(
        err_msg.contains("UTC") || err_msg.contains("America") || err_msg.contains("Asia"),
        "Error message should include examples of valid timezones"
    );
}

#[test]
fn test_timezone_precedence_cli_over_config() {
    let result = TimezoneConfig::load_with_precedence(
        Some("America/New_York"),
        Some("Asia/Jerusalem")
    );
    
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timezone.name(), "America/New_York");
    assert!(config.explicit, "Should be marked as explicit when from CLI");
}

#[test]
fn test_timezone_precedence_config_over_default() {
    let result = TimezoneConfig::load_with_precedence(
        None,
        Some("Asia/Jerusalem")
    );
    
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timezone.name(), "Asia/Jerusalem");
    assert!(config.explicit, "Should be marked as explicit when from config");
}

#[test]
fn test_timezone_precedence_defaults_to_utc_when_not_specified() {
    let result = TimezoneConfig::load_with_precedence(
        None,
        None
    );
    
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timezone.name(), "UTC");
    assert!(!config.explicit, "Should not be marked as explicit for default");
}

#[test]
fn test_timezone_precedence_ignores_utc_in_config() {
    // When config has default UTC, it should be treated as "not set"
    let result = TimezoneConfig::load_with_precedence(
        None,
        Some("UTC")
    );
    
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timezone.name(), "UTC");
    assert!(!config.explicit);
}

#[test]
fn test_local_timezone_special_value() {
    let result = TimezoneConfig::load_with_precedence(
        Some("LOCAL"),
        None
    );
    
    if let Ok(config) = result {
        assert!(config.explicit, "LOCAL should be marked as explicit choice");
        println!("Detected system timezone: {}", config.timezone.name());
    } else {
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("timezone") || err.contains("detect"),
            "Error should be about timezone detection: {}",
            err
        );
    }
}