use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize, Serializer};

/// Custom serializer for time field - converts UTC to specified timezone
fn serialize_time_with_tz<S>(time: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // This serializer will be called during JSON serialization
    // We need to get the timezone from context, but serde doesn't support context passing easily
    // So we'll use a thread-local variable set before serialization
    use std::cell::RefCell;
    
    thread_local! {
        static TIMEZONE: RefCell<Tz> = RefCell::new(chrono_tz::UTC);
    }
    
    let formatted = TIMEZONE.with(|tz| {
        let tz = *tz.borrow();
        let local_time = tz.from_utc_datetime(&time.naive_utc());
        local_time.format("%Y-%m-%d %H:%M").to_string()
    });
    
    serializer.serialize_str(&formatted)
}

/// Set the timezone to use for serialization
pub fn set_serialization_timezone(tz: Tz) {
    use std::cell::RefCell;
    
    thread_local! {
        static TIMEZONE: RefCell<Tz> = RefCell::new(chrono_tz::UTC);
    }
    
    TIMEZONE.with(|timezone| {
        *timezone.borrow_mut() = tz;
    });
}

/// Common weather data structure that all providers transform to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherDataPoint {
    #[serde(serialize_with = "serialize_time_with_tz")]
    pub time: DateTime<Utc>,
    
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
    ) -> Result<Vec<WeatherDataPoint>>;
}
