use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Common weather data structure that all providers transform to
#[derive(Debug, Clone)]
pub struct WeatherDataPoint {
    pub time: DateTime<Utc>,
    pub air_temperature: Option<f64>,
    pub wind_speed: Option<f64>,
    pub wind_direction: Option<f64>,
    pub gust: Option<f64>,
    pub swell_height: Option<f64>,
    pub swell_period: Option<f64>,
    pub swell_direction: Option<f64>,
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
