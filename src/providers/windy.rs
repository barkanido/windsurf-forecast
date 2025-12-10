use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

use crate::forecast_provider::{
    convert_timezone, CloudDatapointSection, ForecastProvider, LocalTimestamp, UtcTimestamp,
    WaveDatapointSection, WeatherData, WeatherDataPoint, WindDatapoinSection,
};
use crate::provider_registry::ProviderMetadata;

// ============================================================================
// Custom Error Types
// ============================================================================

#[derive(Error, Debug)]
#[error("Windy Glass API Error (HTTP {status_code}): {message}")]
pub struct WindyAPIError {
    status_code: u16,
    message: String,
}

impl WindyAPIError {
    fn new(status_code: u16, message: String) -> Self {
        Self {
            status_code,
            message,
        }
    }

    fn from_status_code_and_body(status_code: u16, body: &str) -> Self {
        let message = match status_code {
            204 => "the selected model does not feature any of the requested parameters.".to_string(),
            400 => format!("invalid request, error: {}.", body),
            500 => "unexpected error.".to_string(),
            _ => format!("Unexpected API error (HTTP {}).\nPlease check the API documentation or try again later.", status_code),
        };
        Self::new(status_code, message)
    }
}

// ============================================================================
// Windy-Specific Data Structures
// ============================================================================

#[derive(Debug, Deserialize)]
struct SourceData {
    sg: f64,
}

#[derive(Debug, Deserialize)]
enum Unit {
    #[serde(rename = "m")]
    Meters,
    #[serde(rename = "s")]
    Seconds,
    #[serde(rename = "deg")]
    Degrees,
    #[serde(rename = "K")]
    Kelvins,
    #[serde(rename = "m*s-1")]
    MetersPerSecond,
    #[serde(rename = "%")]
    Percentage,
}

#[derive(Debug, Deserialize)]
struct GfsUnitsRsponse {
    #[serde(rename = "waves_height-surface")]
    waves_hight: Option<Unit>,
    #[serde(rename = "waves_period-surface")]
    waves_period: Option<Unit>,
    #[serde(rename = "waves_direction-surface")]
    waves_direction: Option<Unit>,
    #[serde(rename = "wwaves_height-surface")]
    wind_waves_hight: Option<Unit>,
    #[serde(rename = "wwaves_period-surface")]
    wind_waves_period: Option<Unit>,
    #[serde(rename = "wwaves_direction-surface")]
    wind_waves_direction: Option<Unit>,
    #[serde(rename = "swell1_direction-surface")]
    swell1_direction: Option<Unit>,
    #[serde(rename = "swell1_height-surface")]
    swell1_hight: Option<Unit>,
    #[serde(rename = "swell1_period-surface")]
    swell1_period: Option<Unit>,
}

#[derive(Debug, Deserialize)]
struct GfsWaveUnitsRsponse {
    #[serde(rename = "wind_u-surface")]
    wind_west: Option<Unit>,
    #[serde(rename = "wind_v-surface")]
    wind_south: Option<Unit>,
    #[serde(rename = "gust-surface")]
    gust: Option<Unit>,
    #[serde(rename = "temp-surface")]
    temprature: Option<Unit>,
}

#[derive(Debug, Deserialize)]
struct GfsRawWeatherResponse {
    #[serde(rename = "ts")]
    local_epoch_ts: Vec<i64>,
    units: GfsUnitsRsponse,
    #[serde(rename = "temp-surface")]
    air_temperature: Option<Vec<f64>>,
    #[serde(rename = "wind_u-surface")]
    wind_west: Option<Vec<f64>>,
    #[serde(rename = "wind_v-surface")]
    wind_south: Option<Vec<f64>>,
    #[serde(rename = "gust-surface")]
    gust: Option<Vec<f64>>,
    #[serde(rename = "lclouds-surface")]
    low_cloud_cover: Option<Vec<f64>>,
    #[serde(rename = "mclouds-surface")]
    medium_cloud_cover: Option<Vec<f64>>,
    #[serde(rename = "hclouds-surface")]
    high_cloud_cover: Option<Vec<f64>>,
    #[serde(rename = "past3hprecip-surface")]
    precipitation: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize)]
struct GfsWaveRawWeatherResponse {
    #[serde(rename = "ts")]
    local_epoch_ts: Vec<i64>,
    units: GfsWaveUnitsRsponse,
    #[serde(rename = "wwaves_height-surface")]
    wind_waves_hight: Option<Vec<Option<f64>>>,
    #[serde(rename = "wwaves_period-surface")]
    wind_waves_period: Option<Vec<Option<f64>>>,
    #[serde(rename = "wwaves_direction-surface")]
    wind_waves_direction: Option<Vec<Option<f64>>>,
    #[serde(rename = "swell1_height-surface")]
    swell1_hight: Option<Vec<f64>>,
    #[serde(rename = "swell1_period-surface")]
    swell1_period: Option<Vec<f64>>,
    #[serde(rename = "swell1_direction-surface")]
    swell1_direction: Option<Vec<f64>>,
}

// ============================================================================
// Windy Provider
// ============================================================================

pub struct WindyProvider {
    api_key: String,
    api_url: String,
}

impl WindyProvider {
    pub fn new(api_key: String, api_url: String) -> Self {
        Self { api_key, api_url }
    }
}

#[derive(Debug, Serialize)]
struct WindyRequestBody {
    lat: f64,
    lon: f64,
    model: String,
    parameters: Vec<String>,
    key: String,
    levels: Vec<String>,
}

#[async_trait]
impl ForecastProvider for WindyProvider {
    fn name(&self) -> &str {
        "windy"
    }

    fn get_api_key() -> Result<String> {
        env::var("WINDY_API_KEY").context(
            "WINDY_API_KEY not found. Please set it in your .env file or environment.\n\
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
        let gfs_wave_body = WindyRequestBody {
            lat,
            lon: lng,
            model: "gfsWave".to_string(),
            parameters: vec![
                "swell1".to_string(),
                "waves".to_string(),
                "windWaves".to_string(),
            ],
            levels: vec!["surface".to_string()],
            key: self.api_key.clone(),
        };
        let gfs_body = WindyRequestBody {
            lat,
            lon: lng,
            model: "gfs".to_string(),
            parameters: vec![
                "temp".to_string(),
                "precip".to_string(),
                "wind".to_string(),
                "windGust".to_string(),
            ],
            levels: vec!["surface".to_string()],
            key: self.api_key.clone(),
        };

        println!(
            "Fetching weather data from {} to {} for coordinates ({}, {})",
            start, end, lat, lng
        );

        let client = reqwest::Client::new();
        // Execute both requests in parallel
        let (gfs_wave_response, gfs_response) = tokio::join!(
            client
                .post(self.api_url.as_str())
                .json(&gfs_wave_body)
                .send(),
            client.post(self.api_url.as_str()).json(&gfs_body).send()
        );

        // Handle first response
        let gfs_wave_response =
            gfs_wave_response.context("Failed to connect to Windy API (gfsWave)")?;

        let gfs_wave_status = gfs_wave_response.status();

        let gfs_wave_body = gfs_wave_response
            .text()
            .await
            .context("Failed to read gfsWave response body")?;

        if !gfs_wave_status.is_success() || gfs_wave_status == reqwest::StatusCode::NO_CONTENT {
            return Err(WindyAPIError::from_status_code_and_body(
                gfs_wave_status.as_u16(),
                &gfs_wave_body,
            )
            .into());
        }

        // Handle second response
        let gfs_response = gfs_response.context("Failed to connect to Windy API (gfs)")?;

        let gfs_status = gfs_response.status();

        let gfs_body = gfs_response
            .text()
            .await
            .context("Failed to read gfsWave response body")?;

        if !gfs_status.is_success() || gfs_status == reqwest::StatusCode::NO_CONTENT {
            return Err(
                WindyAPIError::from_status_code_and_body(gfs_status.as_u16(), &gfs_body).into(),
            );
        }

        println!("=== DEBUG: gfsWave Response ===");
        println!("{}", &gfs_wave_body);
        println!("=== DEBUG: gfs Response ===");
        println!("{}", &gfs_body);
        println!("=================================");
        let gfs_wave_data: GfsWaveRawWeatherResponse =
            serde_json::from_str(&gfs_wave_body).context("Failed to parse gfsWave API response")?;
        let gfs_data: GfsRawWeatherResponse =
            serde_json::from_str(&gfs_body).context("Failed to parse gfs API response")?;

        let mut data_points = Vec::with_capacity(gfs_wave_data.local_epoch_ts.len());
        Self::populate_weather_points(&mut data_points, gfs_wave_data, gfs_data, target_tz)?;
        Ok(WeatherData {
            data_points,
            alerts: None,
        })
    }
}

impl WindyProvider {
    fn populate_weather_points(
        weather_points: &mut Vec<WeatherDataPoint>,
        gfs_wave_data: GfsWaveRawWeatherResponse,
        gfs_data: GfsRawWeatherResponse,
        target_tz: Tz,
    ) -> Result<()> {
        // Assuming both responses have the same length and timestamps
        // we iterate over the two responses simultaneously, in each response we iterate over
        // all the fields simultaneously. All the fields are vectors of the same length.
        for i in 0..gfs_wave_data.local_epoch_ts.len() {
            let data_point: WeatherDataPoint = WeatherDataPoint {
                time: Self::convert_timestamp(gfs_wave_data.local_epoch_ts[i], target_tz)?,
                air_temperature: gfs_data
                    .air_temperature
                    .as_ref()
                    .map(|v| Self::kelvin_to_celcius(v[i])),
                wind: WindDatapoinSection {
                    wind_speed: gfs_data
                        .wind_west
                        .as_ref()
                        .zip(gfs_data.wind_south.as_ref())
                        .map(|(west, south)| Self::calc_wind_speed(west[i], south[i])),
                    wind_direction: gfs_data
                        .wind_west
                        .as_ref()
                        .zip(gfs_data.wind_south.as_ref())
                        .map(|(west, south)| Self::calc_wind_direction(west[i], south[i])),
                    gust: gfs_data.gust.as_ref().map(|v| v[i]),
                },
                waves: WaveDatapointSection {
                    swell_height: gfs_wave_data.swell1_hight.as_ref().map(|v| v[i]),
                    swell_period: gfs_wave_data.swell1_period.as_ref().map(|v| v[i]),
                    swell_direction: gfs_wave_data.swell1_direction.as_ref().map(|v| v[i]),
                    wind_wave_height: gfs_wave_data.wind_waves_hight.as_ref().and_then(|v| v[i]),
                    wind_wave_period: gfs_wave_data.wind_waves_period.as_ref().and_then(|v| v[i]),
                    wind_wave_direction: gfs_wave_data.wind_waves_direction.as_ref().and_then(|v| v[i]),
                },

                water_temperature: None,
                clouds: CloudDatapointSection {
                    cloud_cover: None,
                    low_cloud_cover: gfs_data.low_cloud_cover.as_ref().map(|v| v[i]),
                    medium_cloud_cover: gfs_data.medium_cloud_cover.as_ref().map(|v| v[i]),
                    high_cloud_cover: gfs_data.high_cloud_cover.as_ref().map(|v| v[i]),
                },

                precipitation: gfs_data.precipitation.as_ref().map(|v| v[i]),
            };
            weather_points.push(data_point);
        }
        Ok(())
    }

    fn convert_timestamp(ts_ms: i64, target_tz: Tz) -> Result<LocalTimestamp> {
        let utc_datetime = DateTime::<Utc>::from_timestamp_millis(ts_ms)
            .ok_or(anyhow::anyhow!("Invalid timestamp: {}", ts_ms))?;
        let utc = UtcTimestamp(utc_datetime);
        convert_timezone(utc, target_tz)
    }

    fn calc_wind_speed(wind_west: f64, wind_south: f64) -> f64 {
        (wind_west.powi(2) + wind_south.powi(2)).sqrt()
    }

    fn calc_wind_direction(wind_west: f64, wind_south: f64) -> f64 {
        let angle_deg = wind_south.atan2(wind_west).to_degrees();
        // north = 0, east = 90, south = 180, west = 270
        (270.0 - angle_deg) % 360.0
    }

    fn kelvin_to_celcius(kelvin: f64) -> f64 {
        kelvin - 273.15
    }
}

// ============================================================================
// Provider Registry
// ============================================================================

// Register provider with central registry
inventory::submit! {
    ProviderMetadata {
        name: "windy",
        description: "Windy.com Weather API",
        api_key_var: "WINDY_API_KEY",
        instantiate: || {
            let api_key = WindyProvider::get_api_key()?;
            Ok(Box::new(
                WindyProvider::new(
                    api_key,
                    "https://api.windy.com/api/point-forecast/v2".to_string())))
        },
    }
}

// tests
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_wind_direction() {
        let test_cases = vec![
            // (wind_west, wind_south, expected_direction, description)
            (4.0, 4.0, 225.0, "SW wind (blowing toward NE)"),
            (0.0, 4.0, 180.0, "S wind (blowing toward N)"),
            (4.0, 0.0, 270.0, "W wind (blowing toward E)"),
            (0.0, -4.0, 0.0, "N wind (blowing toward S)"),
            (-4.0, 0.0, 90.0, "E wind (blowing toward W)"),
            (-4.0, -4.0, 45.0, "NE wind (blowing toward SW)"),
            (4.0, -4.0, 315.0, "NW wind (blowing toward SE)"),
            (-4.0, 4.0, 135.0, "SE wind (blowing toward NW)"),
        ];

        for (wind_west, wind_south, expected, description) in test_cases {
            let direction = WindyProvider::calc_wind_direction(wind_west, wind_south);
            assert!(
                (direction - expected).abs() < 0.01,
                "Failed for {}: expected {:.1}°, got {:.1}° (wind_west={}, wind_south={})",
                description,
                expected,
                direction,
                wind_west,
                wind_south
            );
        }
    }
}
