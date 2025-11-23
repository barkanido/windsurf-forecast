use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Asia::Jerusalem;
use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

mod forecast_provider;
mod providers;

use forecast_provider::{ForecastProvider, WeatherDataPoint};
use providers::stormglass::StormGlassProvider;

// ============================================================================
// Data Structures for Output
// ============================================================================

#[derive(Debug, Serialize)]
struct TransformedHourlyData {
    time: String,
    #[serde(rename = "airTemperature")]
    air_temperature: f64,
    gust: f64,
    #[serde(rename = "swellDirection")]
    swell_direction: f64,
    #[serde(rename = "swellHeight")]
    swell_height: f64,
    #[serde(rename = "swellPeriod")]
    swell_period: f64,
    #[serde(rename = "waterTemperature")]
    water_temperature: f64,
    #[serde(rename = "windDirection")]
    wind_direction: f64,
    #[serde(rename = "windSpeed")]
    wind_speed: f64,
}

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
    hours: Vec<TransformedHourlyData>,
    meta: TransformedMetaData,
}

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser, Debug)]
#[command(
    name = "stromglass-windsurf-forecast",
    about = "Fetch weather forecast data from Storm Glass API",
    long_about = None,
    after_help = "Examples:\n  \
    # Get 4-day forecast starting today (default)\n  \
    stromglass-windsurf-forecast\n\n  \
    # Get 3-day forecast starting today\n  \
    stromglass-windsurf-forecast --days-ahead 3\n\n  \
    # Get 2-day forecast starting 3 days from now\n  \
    stromglass-windsurf-forecast --days-ahead 2 --first-day-offset 3\n\n  \
    # Get 5-day forecast starting tomorrow\n  \
    stromglass-windsurf-forecast --days-ahead 5 --first-day-offset 1\n\n\
    Note: days-ahead + first-day-offset must not exceed 7 to ensure reliable forecasts."
)]
struct Args {
    /// Number of days to forecast ahead (1-7)
    #[arg(long, default_value_t = 4, value_name = "N")]
    days_ahead: i32,

    /// Number of days to offset the start date (0-7, 0 for today)
    #[arg(long, default_value_t = 0, value_name = "N")]
    first_day_offset: i32,

    /// Weather forecast provider to use
    #[arg(long, default_value = "stormglass", value_name = "PROVIDER")]
    provider: String,
}

// ============================================================================
// Validation Functions
// ============================================================================

fn validate_args(args: &Args) -> Result<()> {
    validate_days_range(args);
    validate_provider(&args.provider)?;

    Ok(())
}

fn validate_days_range(args: &Args) {
    // Validate days_ahead range
    if args.days_ahead < 1 || args.days_ahead > 7 {
        anyhow::bail!("days-ahead must be between 1 and 7 (got {})", args.days_ahead);
    }

    // Validate first_day_offset range
    if args.first_day_offset < 0 || args.first_day_offset > 7 {
        anyhow::bail!(
            "first-day-offset must be between 0 and 7 (got {})",
            args.first_day_offset
        );
    }

    // Validate total days doesn't exceed maximum
    let total_days = args.days_ahead + args.first_day_offset;
    if total_days > 7 {
        anyhow::bail!(
            "days-ahead ({}) + first-day-offset ({}) = {} exceeds maximum of 7 days for reliable forecasts",
            args.days_ahead,
            args.first_day_offset,
            total_days
        );
    }
}

fn validate_provider(provider_name: &str) -> Result<()> {
    match provider_name {
        "stormglass" => Ok(()),
        // Future providers can be added here
        _ => anyhow::bail!(
            "Unknown provider '{}'. Available providers: stormglass",
            provider_name
        ),
    }
}

fn create_provider(provider_name: &str, api_key: String) -> Result<Box<dyn ForecastProvider>> {
    match provider_name {
        "stormglass" => Ok(Box::new(StormGlassProvider::new(api_key))),
        // Future providers can be added here
        _ => unreachable!("Provider validation should have caught this"),
    }
}

// ============================================================================
// Transformation Functions
// ============================================================================

fn transform_weather_point(point: WeatherDataPoint) -> TransformedHourlyData {
    // Convert to Asia/Jerusalem timezone
    let local_time = Jerusalem.from_utc_datetime(&point.time.naive_utc());
    let formatted_time = local_time.format("%Y-%m-%d %H:%M").to_string();

    TransformedHourlyData {
        time: formatted_time,
        air_temperature: point.air_temperature.unwrap_or(0.0),
        gust: point.gust.unwrap_or(0.0),
        swell_direction: point.swell_direction.unwrap_or(0.0),
        swell_height: point.swell_height.unwrap_or(0.0),
        swell_period: point.swell_period.unwrap_or(0.0),
        water_temperature: point.water_temperature.unwrap_or(0.0),
        wind_direction: point.wind_direction.unwrap_or(0.0),
        wind_speed: point.wind_speed.unwrap_or(0.0),
    }
}

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

    // Transform data to output format
    let transformed_hours: Vec<TransformedHourlyData> = weather_points
        .into_iter()
        .map(transform_weather_point)
        .collect();

    let transformed_data = TransformedWeatherResponse {
        hours: transformed_hours,
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
