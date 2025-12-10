// Contract: Updated WeatherDataPoint structure
// Feature: 002-timezone-refactor
// 
// This file defines the updated WeatherDataPoint structure that uses
// LocalTimestamp instead of DateTime<Utc> for the time field.

use serde::{Deserialize, Serialize};

// Import from other contract files
use super::newtype_wrappers::LocalTimestamp;

/// Common weather data structure that all providers transform to
/// 
/// # Changes from Original
/// - `time` field changed from `DateTime<Utc>` to `LocalTimestamp`
/// - Removed `#[serde(serialize_with = "serialize_time_with_tz")]` attribute
/// - Serialization now handled by LocalTimestamp's Serialize implementation
/// 
/// # Backward Compatibility
/// - JSON output format remains "YYYY-MM-DD HH:MM" (not ISO 8601)
/// - Field names unchanged (camelCase in JSON via serde rename)
/// - All measurement fields remain optional
/// 
/// # Example JSON Output
/// ```json
/// {
///   "time": "2024-01-15 14:30",
///   "airTemperature": 20.5,
///   "windSpeed": 15.2,
///   "windDirection": 180.0,
///   "gust": 20.1,
///   "swellHeight": 1.5,
///   "swellPeriod": 8.0,
///   "swellDirection": 200.0,
///   "waterTemperature": 18.0
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherDataPoint {
    /// Forecast timestamp in user's target timezone
    /// 
    /// CHANGED: Now uses LocalTimestamp (was DateTime<Utc>)
    /// Serializes to "YYYY-MM-DD HH:MM" format
    pub time: LocalTimestamp,
    
    /// Air temperature in degrees Celsius
    /// Optional because not all providers return this metric
    #[serde(rename = "airTemperature", skip_serializing_if = "Option::is_none")]
    pub air_temperature: Option<f64>,
    
    /// Wind speed
    /// Unit varies by provider:
    /// - StormGlass: knots (converted from m/s in provider)
    /// - OpenWeatherMap: m/s (no conversion)
    #[serde(rename = "windSpeed", skip_serializing_if = "Option::is_none")]
    pub wind_speed: Option<f64>,
    
    /// Wind direction in degrees (0-360, where 0/360 is North)
    #[serde(rename = "windDirection", skip_serializing_if = "Option::is_none")]
    pub wind_direction: Option<f64>,
    
    /// Wind gust speed (same units as wind_speed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gust: Option<f64>,
    
    /// Swell height in meters
    #[serde(rename = "swellHeight", skip_serializing_if = "Option::is_none")]
    pub swell_height: Option<f64>,
    
    /// Swell period in seconds
    #[serde(rename = "swellPeriod", skip_serializing_if = "Option::is_none")]
    pub swell_period: Option<f64>,
    
    /// Swell direction in degrees (0-360, where 0/360 is North)
    #[serde(rename = "swellDirection", skip_serializing_if = "Option::is_none")]
    pub swell_direction: Option<f64>,
    
    /// Water temperature in degrees Celsius
    #[serde(rename = "waterTemperature", skip_serializing_if = "Option::is_none")]
    pub water_temperature: Option<f64>,
}

impl WeatherDataPoint {
    /// Create a new WeatherDataPoint with all fields optional except time
    /// 
    /// # Example
    /// ```rust
    /// use chrono_tz::America::New_York;
    /// 
    /// let local_time = LocalTimestamp::new(
    ///     New_York.ymd(2024, 1, 15).and_hms(14, 30, 0)
    /// );
    /// 
    /// let point = WeatherDataPoint::new(local_time)
    ///     .with_air_temperature(20.5)
    ///     .with_wind_speed(15.2)
    ///     .with_wind_direction(180.0);
    /// ```
    pub fn new(time: LocalTimestamp) -> Self {
        Self {
            time,
            air_temperature: None,
            wind_speed: None,
            wind_direction: None,
            gust: None,
            swell_height: None,
            swell_period: None,
            swell_direction: None,
            water_temperature: None,
        }
    }
    
    /// Builder method for air_temperature
    pub fn with_air_temperature(mut self, temp: f64) -> Self {
        self.air_temperature = Some(temp);
        self
    }
    
    /// Builder method for wind_speed
    pub fn with_wind_speed(mut self, speed: f64) -> Self {
        self.wind_speed = Some(speed);
        self
    }
    
    /// Builder method for wind_direction
    pub fn with_wind_direction(mut self, direction: f64) -> Self {
        self.wind_direction = Some(direction);
        self
    }
    
    /// Builder method for gust
    pub fn with_gust(mut self, gust: f64) -> Self {
        self.gust = Some(gust);
        self
    }
    
    /// Builder method for swell_height
    pub fn with_swell_height(mut self, height: f64) -> Self {
        self.swell_height = Some(height);
        self
    }
    
    /// Builder method for swell_period
    pub fn with_swell_period(mut self, period: f64) -> Self {
        self.swell_period = Some(period);
        self
    }
    
    /// Builder method for swell_direction
    pub fn with_swell_direction(mut self, direction: f64) -> Self {
        self.swell_direction = Some(direction);
        self
    }
    
    /// Builder method for water_temperature
    pub fn with_water_temperature(mut self, temp: f64) -> Self {
        self.water_temperature = Some(temp);
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use chrono_tz::Tz;
    use super::super::newtype_wrappers::{UtcTimestamp, convert_timezone};
    
    #[test]
    fn test_weather_data_point_creation() {
        let utc = UtcTimestamp::new(Utc.ymd(2024, 1, 15).and_hms(12, 0, 0));
        let tz: Tz = "UTC".parse().unwrap();
        let local = convert_timezone(utc, tz).unwrap();
        
        let point = WeatherDataPoint::new(local)
            .with_air_temperature(20.5)
            .with_wind_speed(15.2);
        
        assert_eq!(point.air_temperature, Some(20.5));
        assert_eq!(point.wind_speed, Some(15.2));
        assert!(point.wind_direction.is_none());
    }
    
    #[test]
    fn test_json_serialization() {
        let utc = UtcTimestamp::new(Utc.ymd(2024, 1, 15).and_hms(12, 30, 0));
        let tz: Tz = "UTC".parse().unwrap();
        let local = convert_timezone(utc, tz).unwrap();
        
        let point = WeatherDataPoint::new(local)
            .with_air_temperature(20.5)
            .with_wind_speed(15.2)
            .with_wind_direction(180.0);
        
        let json = serde_json::to_string(&point).unwrap();
        
        // Verify time format
        assert!(json.contains("\"time\":\"2024-01-15 12:30\""));
        
        // Verify camelCase field names
        assert!(json.contains("\"airTemperature\":20.5"));
        assert!(json.contains("\"windSpeed\":15.2"));
        assert!(json.contains("\"windDirection\":180"));
        
        // Verify optional fields not serialized when None
        assert!(!json.contains("gust"));
        assert!(!json.contains("swellHeight"));
    }
    
    #[test]
    fn test_json_deserialization() {
        let json = r#"{
            "time": "2024-01-15 12:30",
            "airTemperature": 20.5,
            "windSpeed": 15.2,
            "windDirection": 180.0
        }"#;
        
        // Note: Deserialization of LocalTimestamp would need custom deserializer
        // This test demonstrates the expected structure
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        
        assert_eq!(value["time"], "2024-01-15 12:30");
        assert_eq!(value["airTemperature"], 20.5);
        assert_eq!(value["windSpeed"], 15.2);
    }
    
    #[test]
    fn test_timezone_conversion_in_point() {
        // Test that timezone conversion works correctly when creating point
        let utc = UtcTimestamp::new(Utc.ymd(2024, 1, 15).and_hms(17, 0, 0));
        let tz: Tz = "America/New_York".parse().unwrap();
        let local = convert_timezone(utc, tz).unwrap();
        
        let point = WeatherDataPoint::new(local);
        
        // 17:00 UTC = 12:00 EST (UTC-5 in winter)
        assert_eq!(point.time.inner().hour(), 12);
    }
}