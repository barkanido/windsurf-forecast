use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest;
use serde::Deserialize;
use std::env;
use thiserror::Error;

use crate::forecast_provider::{ForecastProvider, WeatherDataPoint};
use crate::provider_registry::ProviderMetadata;

// ============================================================================
// Custom Error Types
// ============================================================================

#[derive(Error, Debug)]
#[error("Storm Glass API Error (HTTP {status_code}): {message}")]
pub struct StormGlassAPIError {
    status_code: u16,
    message: String,
}

impl StormGlassAPIError {
    fn new(status_code: u16, message: String) -> Self {
        Self {
            status_code,
            message,
        }
    }

    fn from_status_code(status_code: u16) -> Self {
        let message = match status_code {
            402 => "Payment Required: You've exceeded the daily request limit for your subscription.\nPlease consider upgrading if this happens frequently, or try again tomorrow.".to_string(),
            403 => "Forbidden: Your API key was not provided or is malformed.\nPlease check that STORMGLASS_API_KEY in your .env file is correct.".to_string(),
            404 => "Not Found: The requested API resource does not exist.\nPlease verify the API endpoint and review the API documentation.".to_string(),
            405 => "Method Not Allowed: The API resource was requested using an unsupported method.\nPlease review the API documentation for correct usage.".to_string(),
            410 => "Gone: You've requested a legacy API resource that is no longer available.\nPlease update your code to use the current API version.".to_string(),
            422 => "Unprocessable Content: Invalid request parameters.\nPlease verify your coordinates, date range, and other parameters are correct.".to_string(),
            503 => "Service Unavailable: Storm Glass is experiencing technical difficulties.\nPlease try again later.".to_string(),
            _ => format!("Unexpected API error (HTTP {}).\nPlease check the API documentation or try again later.", status_code),
        };
        Self::new(status_code, message)
    }
}

// ============================================================================
// StormGlass-Specific Data Structures
// ============================================================================

#[derive(Debug, Deserialize)]
struct SourceData {
    sg: f64,
}

#[derive(Debug, Deserialize)]
struct RawHourlyData {
    time: String,
    #[serde(rename = "airTemperature")]
    air_temperature: Option<SourceData>,
    gust: Option<SourceData>,
    #[serde(rename = "swellDirection")]
    swell_direction: Option<SourceData>,
    #[serde(rename = "swellHeight")]
    swell_height: Option<SourceData>,
    #[serde(rename = "swellPeriod")]
    swell_period: Option<SourceData>,
    #[serde(rename = "waterTemperature")]
    water_temperature: Option<SourceData>,
    #[serde(rename = "windDirection")]
    wind_direction: Option<SourceData>,
    #[serde(rename = "windSpeed")]
    wind_speed: Option<SourceData>,
}

#[derive(Debug, Deserialize)]
struct RawWeatherResponse {
    hours: Vec<RawHourlyData>,
}

// ============================================================================
// StormGlass Provider
// ============================================================================

pub struct StormGlassProvider {
    api_key: String,
}

impl StormGlassProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    /// Convert m/s to knots
    const MS_TO_KNOTS: f64 = 1.94384;

    fn transform_hour(hour: RawHourlyData) -> Result<WeatherDataPoint> {
        let time = DateTime::parse_from_rfc3339(&hour.time)
            .context("Failed to parse timestamp")?
            .with_timezone(&Utc);

        Ok(WeatherDataPoint {
            time,
            air_temperature: hour.air_temperature.map(|s| s.sg),
            wind_speed: hour.wind_speed.map(|s| s.sg * Self::MS_TO_KNOTS),
            wind_direction: hour.wind_direction.map(|s| s.sg),
            gust: hour.gust.map(|s| s.sg * Self::MS_TO_KNOTS),
            swell_height: hour.swell_height.map(|s| s.sg),
            swell_period: hour.swell_period.map(|s| s.sg),
            swell_direction: hour.swell_direction.map(|s| s.sg),
            water_temperature: hour.water_temperature.map(|s| s.sg),
        })
    }
}

#[async_trait]
impl ForecastProvider for StormGlassProvider {
    fn name(&self) -> &str {
        "stormglass"
    }

    fn get_api_key() -> Result<String> {
        env::var("STORMGLASS_API_KEY").context(
            "STORMGLASS_API_KEY not found. Please set it in your .env file or environment.\n\
             See .env.example for the required format.",
        )
    }

    async fn fetch_weather_data(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        lat: f64,
        lng: f64,
    ) -> Result<Vec<WeatherDataPoint>> {
        let params = vec![
            "airTemperature",
            "gust",
            "swellDirection",
            "swellHeight",
            "swellPeriod",
            "waterTemperature",
            "windDirection",
            "windSpeed",
        ];

        println!(
            "Fetching weather data from {} to {} for coordinates ({}, {})",
            start, end, lat, lng
        );

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.stormglass.io/v2/weather/point")
            .query(&[
                ("lat", lat.to_string()),
                ("lng", lng.to_string()),
                ("params", params.join(",")),
                ("start", start.timestamp().to_string()),
                ("end", end.timestamp().to_string()),
                ("source", "sg".to_string()),
            ])
            .header("Authorization", &self.api_key)
            .send()
            .await
            .context("Failed to connect to Storm Glass API")?;

        let status = response.status();

        if !status.is_success() {
            return Err(StormGlassAPIError::from_status_code(status.as_u16()).into());
        }

        let data = response
            .json::<RawWeatherResponse>()
            .await
            .context("Failed to parse API response")?;

        // Transform all hourly data
        let mut weather_points = Vec::with_capacity(data.hours.len());
        for hour in data.hours {
            weather_points.push(Self::transform_hour(hour)?);
        }

        Ok(weather_points)
    }
}

// ============================================================================
// Provider Registry
// ============================================================================

// Register provider with central registry
inventory::submit! {
    ProviderMetadata {
        name: "stormglass",
        description: "StormGlass Marine Weather API",
        api_key_var: "STORMGLASS_API_KEY",
        instantiate: || {
            let api_key = StormGlassProvider::get_api_key()?;
            Ok(Box::new(StormGlassProvider::new(api_key)))
        },
    }
}
