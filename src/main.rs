use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

mod args;
mod config;
mod forecast_provider;
mod provider_registry;
mod providers;

use args::{validate_args, Args};
use config::{check_timezone_match, pick_timezone_interactive};
use forecast_provider::WeatherDataPoint;

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

// ============================================================================
// Transformation Functions
// ============================================================================

fn create_units_map(provider_name: &str) -> HashMap<String, String> {
    let wind_unit = match provider_name {
        "stormglass" => "knots",
        "openweathermap" => "m/s",
        _ => "m/s",
    };

    [
        (
            "windSpeed",
            format!("Speed of wind at 10m above ground in {}", wind_unit),
        ),
        ("gust", format!("Wind gust in {}", wind_unit)),
        (
            "airTemperature",
            "Air temperature in degrees celsius".to_string(),
        ),
        ("swellHeight", "Height of swell waves in meters".to_string()),
        (
            "swellPeriod",
            "Period of swell waves in seconds".to_string(),
        ),
        (
            "swellDirection",
            "Direction of swell waves. 0° indicates swell coming from north".to_string(),
        ),
        (
            "waterTemperature",
            "Water temperature in degrees celsius".to_string(),
        ),
        (
            "windDirection",
            "Direction of wind at 10m above ground. 0° indicates wind coming from north"
                .to_string(),
        ),
        ("cloudCover", "Total cloud coverage in percent".to_string()),
        (
            "precipitation",
            "TMean precipitation in kg/m²/h = mm/h".to_string(),
        ),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.clone()))
    .collect()
}

fn create_meta(
    lat: f64,
    lng: f64,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    provider_name: &str,
    tz: Tz,
) -> TransformedMetaData {
    let now = tz.from_utc_datetime(&Utc::now().naive_utc());
    let report_time = now.format("%Y-%m-%d %H:%M").to_string();

    TransformedMetaData {
        lat,
        lng,
        start: start.to_rfc3339(),
        end: end.to_rfc3339(),
        report_generated_at: report_time,
        provider: provider_name.to_string(),
        units: create_units_map(provider_name),
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

fn write_weather_json(data: &TransformedWeatherResponse, filename: &str) -> Result<()> {
    println!("Writing weather data to file: {}", filename);
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
    dotenv::dotenv().ok();
    provider_registry::check_duplicates();
    let args = Args::parse();

    if args.list_providers {
        println!("Available weather providers:\n");
        for (name, description) in provider_registry::all_provider_descriptions() {
            let metadata = provider_registry::get_provider_metadata(name).unwrap();
            println!("  {} - {}", name, description);
            println!("    API Key: {}", metadata.api_key_var);
            println!();
        }
        return Ok(());
    }

    if args.pick_timezone {
        let selected_tz = pick_timezone_interactive()?;

        let mut config = config::load_config_from_file(args.config_file_path.as_ref())?;
        config.general.timezone = selected_tz.clone();
        config::save_config(&config, args.config_file_path.as_ref())?;

        let config_path = args.config_path_display().unwrap_or_else(|| {
            config::get_default_config_path()
                .unwrap()
                .display()
                .to_string()
        });
        println!(
            "✓ Timezone '{}' saved to config file: {}",
            selected_tz, config_path
        );
        return Ok(());
    }

    validate_args(&args)?;

    let resolved_config = config::resolve_from_args_and_file(&args)?;

    eprintln!("\n{}", resolved_config);

    let config_path = args.config_path_display().unwrap_or_else(|| {
        config::get_default_config_path()
            .unwrap()
            .display()
            .to_string()
    });
    eprintln!("   Config file: {}", config_path);

    check_timezone_match(
        resolved_config.timezone,
        resolved_config.lat,
        resolved_config.lng,
    );

    let provider = provider_registry::create_provider(&resolved_config.provider)?;

    let now = Utc::now();
    let start = day_start_utc(now, resolved_config.first_day_offset as i64);
    let end = day_end_utc(
        now,
        (resolved_config.first_day_offset + resolved_config.days_ahead - 1) as i64,
    );

    let weather_points = provider
        .fetch_weather_data(
            start,
            end,
            resolved_config.lat,
            resolved_config.lng,
            resolved_config.timezone,
        )
        .await?;

    let transformed_data = TransformedWeatherResponse {
        hours: weather_points,
        meta: create_meta(
            resolved_config.lat,
            resolved_config.lng,
            start,
            end,
            provider.name(),
            resolved_config.timezone,
        ),
    };

    let filename = format!(
        "weather_data_{}d_{}.json",
        resolved_config.days_ahead,
        start.format("%y%m%d")
    );

    write_weather_json(&transformed_data, &filename)?;
    println!(
        "Loaded {} hourly data points from file.",
        transformed_data.hours.len()
    );

    if args.save {
        config::save_config_from_resolved(&resolved_config, args.config_file_path.as_ref())?;
    }

    Ok(())
}

/// Creates a UTC datetime at the start of day (00:00:00) for a date offset from now
fn day_start_utc(now: DateTime<Utc>, day_offset: i64) -> DateTime<Utc> {
    let date = (now + chrono::Duration::days(day_offset))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    Utc.from_utc_datetime(&date)
}

/// Creates a UTC datetime at the end of day (23:59:59) for a date offset from now
fn day_end_utc(now: DateTime<Utc>, day_offset: i64) -> DateTime<Utc> {
    let date = (now + chrono::Duration::days(day_offset))
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap();
    Utc.from_utc_datetime(&date)
}
