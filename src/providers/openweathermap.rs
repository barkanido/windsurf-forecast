use crate::forecast_provider::{
    CloudDatapointSection, ForecastProvider, UtcTimestamp, WaveDatapointSection, WeatherData, WeatherDataPoint, WindDatapoinSection, convert_timezone
};
use crate::provider_registry::ProviderMetadata;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use std::env;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
struct RawWeatherResponse {
    hourly: Vec<RawHourlyData>,
    alerts: Option<Vec<RawAlert>>,
}

#[derive(Debug, Deserialize)]
struct RawAlert {
    sender_name: String,
    event: String,
    start: i64,
    end: i64,
    description: String,
}

impl Display for RawAlert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Alert[{}]: {}\nFrom: {} To: {}\nDescription: {}",
            self.sender_name, self.event, self.start, self.end, self.description
        )
    }
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
    name: String,
    short_name: String,
}

impl OpenWeatherMapProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key, name: "openweathermap".to_string(), short_name: "owm".to_string() }
    }

    fn build_weather_data_point(hour: RawHourlyData, target_tz: Tz) -> Result<WeatherDataPoint> {
        let utc_datetime = DateTime::<Utc>::from_timestamp(hour.dt, 0)
            .ok_or(anyhow::anyhow!("Could not parse timestamp"))?;
        let utc = UtcTimestamp(utc_datetime);

        let local = convert_timezone(utc, target_tz)?;

        Ok(WeatherDataPoint {
            time: local,
            air_temperature: Some(hour.air_temperature),
            wind: WindDatapoinSection {
                wind_speed: hour.wind_speed,
                wind_direction: hour.wind_deg,
                gust: hour.wind_gust,
            },
            waves: WaveDatapointSection {
                swell_height: None,
                swell_period: None,
                swell_direction: None,
                wind_wave_height: None,
                wind_wave_period: None,
                wind_wave_direction: None,
            },
            water_temperature: None,
            clouds: CloudDatapointSection {
                cloud_cover: hour.clouds,
                low_cloud_cover: None,
                medium_cloud_cover: None,
                high_cloud_cover: None,
            },
            precipitation: None, // TODO: Map precipitation if available
        })
    }
}

#[async_trait]
impl ForecastProvider for OpenWeatherMapProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn short_name(&self) -> &str {
        &self.short_name
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
    ) -> Result<WeatherData> {
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

        let alerts_formatted = data.alerts.map(|alerts| {
            alerts
                .into_iter()
                // filter_map to convert and skip any invalid timestamps
                .filter_map(|alert| {
                    let utc_start = DateTime::<Utc>::from_timestamp(alert.start, 0)?;
                    let utc_end = DateTime::<Utc>::from_timestamp(alert.end, 0)?;

                    let start_local = target_tz.from_utc_datetime(&utc_start.naive_utc());
                    let end_local = target_tz.from_utc_datetime(&utc_end.naive_utc());

                    Some(format!(
                        "Alert[{}]: {}\nFrom: {} To: {}\nDescription: {}",
                        alert.sender_name,
                        alert.event,
                        start_local.format("%Y-%m-%d %H:%M"),
                        end_local.format("%Y-%m-%d %H:%M"),
                        alert.description
                    ))
                })
                .collect::<Vec<String>>()
        });

        let mut data_points = Vec::with_capacity(data.hourly.len());
        for hour in data.hourly {
            data_points.push(Self::build_weather_data_point(hour, target_tz)?);
        }

        Ok(WeatherData{data_points, alerts: alerts_formatted})
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
