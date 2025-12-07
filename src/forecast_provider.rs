use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Serialize, Serializer};

// ============================================================================
// Newtype Wrappers for Timezone Safety
// ============================================================================

/// Newtype wrapper for UTC timestamps
///
/// Used internally to represent timestamps as received from weather APIs.
/// This type makes it explicit that a timestamp is in UTC and has not yet
/// been converted to the user's target timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcTimestamp(pub DateTime<Utc>);

impl UtcTimestamp {
    /// Parse from RFC3339 string (common API format)
    pub fn from_rfc3339(s: &str) -> Result<Self> {
        let dt = DateTime::parse_from_rfc3339(s)
            .map_err(|e| anyhow!("Failed to parse UTC timestamp from '{}': {}", s, e))?
            .with_timezone(&Utc);
        Ok(Self(dt))
    }
}

/// Newtype wrapper for timezone-converted timestamps
///
/// Used in output structures after conversion to user's target timezone.
/// This type makes it explicit that a timestamp has been converted and is
/// ready for display/serialization.
#[derive(Debug, Clone)]
pub struct LocalTimestamp {
    inner: DateTime<Tz>,
}

impl LocalTimestamp {
    /// Create a new LocalTimestamp from a DateTime<Tz>
    pub fn new(dt: DateTime<Tz>) -> Self {
        Self { inner: dt }
    }
}

// Custom serialization to maintain "YYYY-MM-DD HH:MM" format
impl Serialize for LocalTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Format: "YYYY-MM-DD HH:MM" (not ISO 8601)
        // This maintains backward compatibility with existing output
        let formatted = self.inner.format("%Y-%m-%d %H:%M").to_string();
        serializer.serialize_str(&formatted)
    }
}

/// Convert a UTC timestamp to the target timezone
///
/// This is the core timezone conversion function that should be called
/// in the provider transform layer (not during serialization).
pub fn convert_timezone(utc: UtcTimestamp, target_tz: Tz) -> Result<LocalTimestamp> {
    let local = target_tz.from_utc_datetime(&utc.0.naive_utc());
    Ok(LocalTimestamp::new(local))
}

// ============================================================================
// WeatherDataPoint Structure
// ============================================================================

/// Common weather data structure that all providers transform to
#[derive(Debug, Clone, Serialize)]
pub struct WeatherDataPoint {
    pub time: LocalTimestamp,
    
    #[serde(rename = "airTemperature", skip_serializing_if = "Option::is_none")]
    pub air_temperature: Option<f64>,
    
    #[serde(rename = "windSpeed", skip_serializing_if = "Option::is_none")]
    pub wind_speed: Option<f64>,
    
    #[serde(rename = "windDirection", skip_serializing_if = "Option::is_none")]
    pub wind_direction: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gust: Option<f64>,
    
    #[serde(rename = "swellHeight", skip_serializing_if = "Option::is_none")]
    pub swell_height: Option<f64>,
    
    #[serde(rename = "swellPeriod", skip_serializing_if = "Option::is_none")]
    pub swell_period: Option<f64>,
    
    #[serde(rename = "swellDirection", skip_serializing_if = "Option::is_none")]
    pub swell_direction: Option<f64>,
    
    #[serde(rename = "waterTemperature", skip_serializing_if = "Option::is_none")]
    pub water_temperature: Option<f64>,
}

/// Trait that all weather forecast providers must implement
#[async_trait]
pub trait ForecastProvider: Send + Sync {
    /// Get the name of this provider (e.g., "stormglass")
    fn name(&self) -> &str;
    
    /// Get the API key from environment variables
    /// Returns the API key value or an error if not found/invalid
    fn get_api_key() -> Result<String>
    where
        Self: Sized;
    
    /// Fetch weather data for the given time range and location
    /// Returns a vector of weather data points
    async fn fetch_weather_data(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        lat: f64,
        lng: f64,
        target_tz: Tz,
    ) -> Result<Vec<WeatherDataPoint>>;
}
