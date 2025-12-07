// Contract: Updated ForecastProvider trait with timezone parameter
// Feature: 002-timezone-refactor
// 
// This file defines the updated provider trait signature that all weather
// providers must implement. The key change is adding a timezone parameter
// to the fetch_weather_data() method.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

// Re-export types from other contract files for convenience
use super::weather_data_point::WeatherDataPoint;

/// Trait that all weather forecast providers must implement
/// 
/// # Changes from Original
/// - Added `target_tz: Tz` parameter to `fetch_weather_data()`
/// - Provider implementations must convert timestamps to target timezone
/// - Returns WeatherDataPoint with LocalTimestamp (not DateTime<Utc>)
/// 
/// # Implementation Contract
/// 1. Parse API response timestamps to UtcTimestamp
/// 2. Convert to LocalTimestamp using provided target_tz
/// 3. Construct WeatherDataPoint with LocalTimestamp
/// 4. No timezone logic should remain in serialization layer
#[async_trait]
pub trait ForecastProvider: Send + Sync {
    /// Get the name of this provider (e.g., "stormglass", "openweathermap")
    fn name(&self) -> &str;
    
    /// Get the API key from environment variables
    /// 
    /// Returns the API key value or an error if not found/invalid.
    /// Each provider should read from its own environment variable
    /// (e.g., STORMGLASS_API_KEY, OPEN_WEATHER_MAP_API_KEY).
    fn get_api_key() -> Result<String>
    where
        Self: Sized;
    
    /// Fetch weather data for the given time range and location
    /// 
    /// # Arguments
    /// * `start` - Start of forecast period (UTC)
    /// * `end` - End of forecast period (UTC)
    /// * `lat` - Latitude of location
    /// * `lng` - Longitude of location
    /// * `target_tz` - **NEW**: Target timezone for output timestamps
    /// 
    /// # Returns
    /// Vector of weather data points with timestamps converted to target_tz
    /// 
    /// # Implementation Notes
    /// - API requests should still use UTC timestamps for start/end
    /// - Parse API response timestamps as UTC
    /// - Convert to target_tz in the transform layer (NOT during serialization)
    /// - Return WeatherDataPoint with LocalTimestamp fields
    /// 
    /// # Example Implementation Pattern
    /// ```rust
    /// async fn fetch_weather_data(
    ///     &self,
    ///     start: DateTime<Utc>,
    ///     end: DateTime<Utc>,
    ///     lat: f64,
    ///     lng: f64,
    ///     target_tz: Tz,
    /// ) -> Result<Vec<WeatherDataPoint>> {
    ///     // 1. Make API request (start/end are UTC)
    ///     let response = self.api_client.fetch(...).await?;
    ///     
    ///     // 2. Parse response, extract UTC timestamps
    ///     let raw_data = parse_response(&response)?;
    ///     
    ///     // 3. Transform to WeatherDataPoint with timezone conversion
    ///     let data_points = raw_data.into_iter()
    ///         .map(|item| {
    ///             // Parse as UTC
    ///             let utc_time = UtcTimestamp::from_rfc3339(&item.time)?;
    ///             
    ///             // Convert to target timezone HERE (not in serialization)
    ///             let local_time = convert_timezone(utc_time, target_tz)?;
    ///             
    ///             Ok(WeatherDataPoint {
    ///                 time: local_time,  // LocalTimestamp, not DateTime<Utc>
    ///                 air_temperature: item.temp,
    ///                 wind_speed: item.wind,
    ///                 // ... other fields
    ///             })
    ///         })
    ///         .collect::<Result<Vec<_>>>()?;
    ///     
    ///     Ok(data_points)
    /// }
    /// ```
    async fn fetch_weather_data(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        lat: f64,
        lng: f64,
        target_tz: Tz,  // NEW PARAMETER
    ) -> Result<Vec<WeatherDataPoint>>;
}

// ============================================================================
// Migration Guide for Existing Providers
// ============================================================================

/// # Migration Checklist for Provider Implementations
/// 
/// When updating an existing provider to support the new timezone parameter:
/// 
/// ## Step 1: Update Function Signature
/// ```rust
/// // OLD:
/// async fn fetch_weather_data(
///     &self,
///     start: DateTime<Utc>,
///     end: DateTime<Utc>,
///     lat: f64,
///     lng: f64,
/// ) -> Result<Vec<WeatherDataPoint>>
/// 
/// // NEW:
/// async fn fetch_weather_data(
///     &self,
///     start: DateTime<Utc>,
///     end: DateTime<Utc>,
///     lat: f64,
///     lng: f64,
///     target_tz: Tz,  // ADD THIS
/// ) -> Result<Vec<WeatherDataPoint>>
/// ```
/// 
/// ## Step 2: Update Transform Logic
/// ```rust
/// // OLD: Direct assignment of DateTime<Utc>
/// WeatherDataPoint {
///     time: parsed_utc_time,  // DateTime<Utc>
///     // ...
/// }
/// 
/// // NEW: Convert to LocalTimestamp
/// use super::newtype_wrappers::{UtcTimestamp, convert_timezone};
/// 
/// let utc = UtcTimestamp::new(parsed_utc_time);
/// let local = convert_timezone(utc, target_tz)?;
/// 
/// WeatherDataPoint {
///     time: local,  // LocalTimestamp
///     // ...
/// }
/// ```
/// 
/// ## Step 3: Update Tests
/// ```rust
/// // OLD: No timezone parameter
/// let data = provider.fetch_weather_data(start, end, lat, lng).await?;
/// 
/// // NEW: Pass timezone explicitly
/// let tz: Tz = "UTC".parse()?;
/// let data = provider.fetch_weather_data(start, end, lat, lng, tz).await?;
/// ```
/// 
/// ## Step 4: Update Call Sites
/// All locations calling fetch_weather_data() must pass target_tz:
/// ```rust
/// // In main.rs or similar
/// let timezone_config = TimezoneConfig::from_args(&args)?;
/// let data = provider.fetch_weather_data(
///     start,
///     end,
///     lat,
///     lng,
///     timezone_config.timezone,  // Pass timezone here
/// ).await?;
/// ```
/// 
/// ## Affected Files
/// - src/providers/stormglass.rs
/// - src/providers/openweathermap.rs
/// - src/main.rs (call site)
/// - Tests for each provider

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    
    /// Mock provider for testing trait usage
    struct MockProvider;
    
    #[async_trait]
    impl ForecastProvider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }
        
        fn get_api_key() -> Result<String> {
            Ok("test_key".to_string())
        }
        
        async fn fetch_weather_data(
            &self,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
            _lat: f64,
            _lng: f64,
            target_tz: Tz,
        ) -> Result<Vec<WeatherDataPoint>> {
            // Demonstrate proper timezone conversion pattern
            use super::super::newtype_wrappers::{UtcTimestamp, convert_timezone};
            
            let utc_time = Utc.ymd(2024, 1, 15).and_hms(12, 0, 0);
            let utc = UtcTimestamp::new(utc_time);
            let local = convert_timezone(utc, target_tz)?;
            
            Ok(vec![WeatherDataPoint {
                time: local,
                air_temperature: Some(20.0),
                wind_speed: Some(10.0),
                wind_direction: Some(180.0),
                gust: None,
                swell_height: None,
                swell_period: None,
                swell_direction: None,
                water_temperature: None,
            }])
        }
    }
    
    #[tokio::test]
    async fn test_provider_with_timezone() {
        let provider = MockProvider;
        let start = Utc.ymd(2024, 1, 15).and_hms(0, 0, 0);
        let end = Utc.ymd(2024, 1, 16).and_hms(0, 0, 0);
        let tz: Tz = "America/New_York".parse().unwrap();
        
        let result = provider.fetch_weather_data(start, end, 40.7, -74.0, tz).await;
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 1);
        // Verify timezone was applied (12:00 UTC = 07:00 EST)
        assert_eq!(data[0].time.inner().hour(), 7);
    }
}