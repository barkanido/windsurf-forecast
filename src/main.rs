use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Asia::Jerusalem;
use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

mod args;
mod forecast_provider;
mod providers;

use args::{Args, validate_args};
use forecast_provider::{ForecastProvider, WeatherDataPoint};
use providers::{stormglass::StormGlassProvider, openweathermap::OpenWeatherMapProvider};

// ============================================================================
// Data Structures for Output
// ============================================================================

#[derive(Debug, Serialize)]
struct TransformedMetaData {
    lat: f64,
    lng: f64,
    start: String,
    end: String,
    #[serde(rename = "report_generated_at")]
    report_generated_at: String,
    provider: String,
    units: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct TransformedWeatherResponse {
    hours: Vec<WeatherDataPoint>,
    meta: TransformedMetaData,
}

fn create_provider(provider_name: &str, api_key: String) -> Result<Box<dyn ForecastProvider>> {
    match provider_name {
        "stormglass" => Ok(Box::new(StormGlassProvider::new(api_key))),
        "openweathermap" => Ok(Box::new(OpenWeatherMapProvider::new(api_key))),
        // Future providers can be added here
        _ => unreachable!("Provider validation should have caught this"),
    }
}

// ============================================================================
// Transformation Functions
// ============================================================================

fn create_units_map() -> HashMap<String, String> {
    [
        ("windSpeed", "Speed of wind at 10m above ground in knots"),
        ("gust", "Wind gust in knots"),
        ("airTemperature", "Air temperature in degrees celsius"),
        ("swellHeight", "Height of swell waves in meters"),
        ("swellPeriod", "Period of swell waves in seconds"),
        ("swellDirection", "Direction of swell waves. 0° indicates swell coming from north"),
        ("waterTemperature", "Water temperature in degrees celsius"),
        ("windDirection", "Direction of wind at 10m above ground. 0° indicates wind coming from north"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

fn create_meta(
    lat: f64,
    lng: f64,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    provider_name: &str,
) -> TransformedMetaData {
    let now = Jerusalem.from_utc_datetime(&Utc::now().naive_utc());
    let report_time = now.format("%Y-%m-%d %H:%M").to_string();

    TransformedMetaData {
        lat,
        lng,
        start: start.to_rfc3339(),
        end: end.to_rfc3339(),
        report_generated_at: report_time,
        provider: provider_name.to_string(),
        units: create_units_map(),
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

fn write_weather_json(data: &TransformedWeatherResponse, filename: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    fs::write(filename, json)?;
    Ok(())
}

fn print_error(error_type: &str, message: &str) {
    eprintln!("\n{}", "=".repeat(70));
    eprintln!("{}", error_type);
    eprintln!("{}", "=".repeat(70));
    eprintln!("\n{}", message);
    eprintln!("\n{}", "=".repeat(70));
}

// ============================================================================
// Main Function
// ============================================================================

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        print_error("ERROR", &format!("{:#}", e));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Load .env file if present
    dotenv::dotenv().ok();

    // Parse command line arguments
    let args = Args::parse();

    // Validate all arguments
    validate_args(&args)?;

    // Get API key for the selected provider
    let api_key = match args.provider.as_str() {
        "stormglass" => StormGlassProvider::get_api_key()?,
        "openweathermap" => OpenWeatherMapProvider::get_api_key()?,
        // Future providers can be added here
        _ => unreachable!("Provider validation should have caught this"),
    };

    // Create provider instance
    let provider = create_provider(&args.provider, api_key)?;

    // Calculate start and end dates
    let now = Utc::now();
    let start = (now + chrono::Duration::days(args.first_day_offset as i64))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let start = Utc.from_utc_datetime(&start);

    let end = (now + chrono::Duration::days((args.first_day_offset + args.days_ahead - 1) as i64))
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap();
    let end = Utc.from_utc_datetime(&end);

    // Coordinates for the location (32°29'12.2"N 34°53'19.4"E)
    let lat = 32.486722;
    let lng = 34.888722;

    // Fetch weather data using the provider
    let weather_points = provider.fetch_weather_data(start, end, lat, lng).await?;

    // Create response with weather data - no transformation needed
    // WeatherDataPoint now serializes directly with proper formatting
    let transformed_data = TransformedWeatherResponse {
        hours: weather_points,
        meta: create_meta(lat, lng, start, end, provider.name()),
    };

    // Generate filename
    let filename = format!(
        "weather_data_{}d_{}.json",
        args.days_ahead,
        start.format("%y%m%d")
    );

    // Write to file
    write_weather_json(&transformed_data, &filename)?;
    
    // Print the data
    println!("{}", serde_json::to_string_pretty(&transformed_data)?);
    
    println!("Loaded {} hourly data points from file.", transformed_data.hours.len());

    Ok(())
}
