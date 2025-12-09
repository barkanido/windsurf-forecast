use crate::forecast_provider::{
    convert_timezone, ForecastProvider, UtcTimestamp, WeatherDataPoint,
};
use crate::provider_registry::ProviderMetadata;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct RawWeatherResponse {
    hourly: Vec<RawHourlyData>,
}

#[derive(Debug, Deserialize)]
struct RawHourlyData {
    dt: i64,
    #[serde(rename = "feels_like")]
    air_temperature: f64,
    clouds: Option<f64>,
    wind_deg: Option<f64>,
    wind_gust: Option<f64>,
    wind_speed: Option<f64>,
}

pub struct OpenWeatherMapProvider {
    api_key: String,
}

impl OpenWeatherMapProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    fn transform_hour(hour: RawHourlyData, target_tz: Tz) -> Result<WeatherDataPoint> {
        let utc_datetime = DateTime::<Utc>::from_timestamp(hour.dt, 0)
            .ok_or(anyhow::anyhow!("Could not parse timestamp"))?;
        let utc = UtcTimestamp(utc_datetime);

        let local = convert_timezone(utc, target_tz)?;

        Ok(WeatherDataPoint {
            time: local,
            air_temperature: Some(hour.air_temperature),
            wind_speed: hour.wind_speed,
            wind_direction: hour.wind_deg,
            gust: hour.wind_gust,
            swell_height: None,
            swell_period: None,
            swell_direction: None,
            water_temperature: None,
            cloud_cover: hour.clouds,
            precipitation: None, // TODO: Map precipitation if available
        })
    }
}

#[async_trait]
impl ForecastProvider for OpenWeatherMapProvider {
    fn name(&self) -> &str {
        "openweathermap"
    }

    fn get_api_key() -> Result<String>
    where
        Self: Sized,
    {
        env::var("OPEN_WEATHER_MAP_API_KEY").context(
            "OPEN_WEATHER_MAP_API_KEY not found. Please set it in your .env file or environment.\n\
             See .env.example for the required format.",
        )
    }

    async fn fetch_weather_data(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        lat: f64,
        lng: f64,
        target_tz: Tz,
    ) -> Result<Vec<WeatherDataPoint>> {
        println!(
            "Fetching weather data from {} to {} for coordinates ({}, {})",
            start, end, lat, lng
        );
        let client = reqwest::Client::new();
        let req = client
            .get("https://api.openweathermap.org/data/3.0/onecall")
            .query(&[
                ("lat", &lat.to_string()),
                ("lon", &lng.to_string()),
                ("appid", &self.api_key.to_string()),
                ("units", &"metric".to_string()),
                ("mode", &"json".to_string()),
            ]);
        let response = req
            .send()
            .await
            .context("Failed to connect to openweathermap API")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "OpenWeatherMap API returned error status: {}, message: {}",
                status.as_u16(),
                response_text
            ));
        }
        // print!("raw response\n{}", response_text);

        let data: RawWeatherResponse =
            serde_json::from_str(&response_text).context("Failed to parse API response")?;

        let mut weather_points = Vec::with_capacity(data.hourly.len());
        for hour in data.hourly {
            weather_points.push(Self::transform_hour(hour, target_tz)?);
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
        name: "openweathermap",
        description: "OpenWeatherMap Global Weather Data",
        api_key_var: "OPEN_WEATHER_MAP_API_KEY",
        instantiate: || {
            let api_key = OpenWeatherMapProvider::get_api_key()?;
            Ok(Box::new(OpenWeatherMapProvider::new(api_key)))
        },
    }
}
