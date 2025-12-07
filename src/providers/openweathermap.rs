use crate::forecast_provider::{ForecastProvider, WeatherDataPoint, UtcTimestamp, convert_timezone};
use crate::provider_registry::ProviderMetadata;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawWeatherResponse {
    list: Vec<RawHourlyData>,
}

#[derive(Debug, Deserialize)]
struct RawHourlyData {
    dt: i64, //unix timestamp, UTC
    main: MainData,
    wind: WindData,
    #[allow(dead_code)]  // Part of API response but not used
    timezone: i64,
}

#[derive(Debug, Deserialize)]
struct MainData {
    #[serde(rename = "temp")]
    air_temperature: f64, //celsius
}

#[derive(Debug, Deserialize)]
struct WindData {
    #[serde(rename = "speed")]
    wind_speed: Option<f64>, //m/s
    #[serde(rename = "deg")]
    wind_direction: Option<f64>,
    gust: Option<f64>, //m/s
}
pub struct OpenWeatherMapProvider {
    api_key: String,
}

impl OpenWeatherMapProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    fn transform_hour(hour: RawHourlyData, target_tz: Tz) -> Result<WeatherDataPoint> {
        // Parse Unix timestamp as UTC
        let utc_datetime = DateTime::<Utc>::from_timestamp(hour.dt, 0)
            .ok_or(anyhow::anyhow!("Could not parse timestamp"))?;
        let utc = UtcTimestamp(utc_datetime);
        
        // Convert to target timezone
        let local = convert_timezone(utc, target_tz)?;

        Ok(WeatherDataPoint {
            time: local,
            air_temperature: Some(hour.main.air_temperature),
            wind_speed: hour.wind.wind_speed,
            wind_direction: hour.wind.wind_direction,
            gust: hour.wind.gust,
            swell_height: None,
            swell_period: None,
            swell_direction: None,
            water_temperature: None,
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
        dotenv::var("OPEN_WEATHER_MAP_API_KEY").context("OPEN_WEATHER_MAP_API_KEY not set in .env")
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
            .get("https://pro.openweathermap.org/data/2.5/forecast/hourly")
            .query(&[
                ("lat", &lat.to_string()),
                ("lon", &lng.to_string()),
                ("appid", &self.api_key.to_string()),
                ("units", &"metric".to_string()),
                ("mode", &"json".to_string()),
            ]);
        println!("{:#?}", req);
        let response = req
            .send()
            .await
            .context("Failed to connect to openweathermap API")?;

        let status = response.status();

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "OpenWeatherMap API returned error status: {}, message: {}",
                status.as_u16(),
                response.text().await.unwrap_or_default()
            ));
        }

        let data = response
            .json::<RawWeatherResponse>()
            .await
            .context("Failed to parse API response")?;

        // Transform all hourly data
        let mut weather_points = Vec::with_capacity(data.list.len());
        for hour in data.list {
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
