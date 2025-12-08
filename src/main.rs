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

use args::{Args, validate_args};
use config::{load_config, save_config, validate_timezone_coordinates, pick_timezone_interactive, TimezoneConfig};
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
    // Determine wind speed units based on provider
    let wind_unit = match provider_name {
        "stormglass" => "knots",
        "openweathermap" => "m/s",
        _ => "m/s", // Default to m/s for unknown providers
    };
    
    [
        ("windSpeed", format!("Speed of wind at 10m above ground in {}", wind_unit)),
        ("gust", format!("Wind gust in {}", wind_unit)),
        ("airTemperature", "Air temperature in degrees celsius".to_string()),
        ("swellHeight", "Height of swell waves in meters".to_string()),
        ("swellPeriod", "Period of swell waves in seconds".to_string()),
        ("swellDirection", "Direction of swell waves. 0° indicates swell coming from north".to_string()),
        ("waterTemperature", "Water temperature in degrees celsius".to_string()),
        ("windDirection", "Direction of wind at 10m above ground. 0° indicates wind coming from north".to_string()),
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
    // Load .env file if present
    dotenv::dotenv().ok();

    // Validate provider registry (check for duplicates)
    provider_registry::check_duplicates();

    // Parse command line arguments
    let args = Args::parse();

    // Handle --list-providers flag
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

    // Handle interactive timezone picker
    if args.pick_timezone {
        let selected_tz = pick_timezone_interactive()?;
        
        // Load existing config or create new one
        let mut config = load_config(args.config.as_ref())?;
        config.general.timezone = selected_tz.clone();
        
        // Save config
        save_config(&config, args.config.as_ref())?;
        
        let config_path = if let Some(ref path) = args.config {
            path.display().to_string()
        } else {
            config::get_default_config_path()?.display().to_string()
        };
        
        println!("✓ Timezone '{}' saved to config file: {}", selected_tz, config_path);
        return Ok(());
    }

    // Load configuration file
    let mut config = load_config(args.config.as_ref())?;
    
    // Determine timezone source for display
    let timezone_source = if args.timezone.is_some() {
        "CLI argument"
    } else if config.general.timezone != "UTC" {
        "config file"
    } else {
        "default"
    };

    // Load timezone configuration with precedence: CLI > Config > default UTC
    let timezone_config = TimezoneConfig::load_with_precedence(
        args.timezone.clone(),
        Some(config.general.timezone.clone())
    )?;
    timezone_config.display_default_warning();
    
    let timezone = timezone_config.timezone;
    
    // If timezone was specified via CLI, persist it to config
    if let Some(ref tz_str) = args.timezone {
        config.general.timezone = tz_str.clone();
        save_config(&config, args.config.as_ref())?;
        
        let config_path = if let Some(ref path) = args.config {
            path.display().to_string()
        } else {
            config::get_default_config_path()?.display().to_string()
        };
        eprintln!("✓ Timezone '{}' saved to config file: {}", tz_str, config_path);
    }

    // Validate all arguments
    validate_args(&args)?;

    // Create provider instance using registry
    let provider = provider_registry::create_provider(&args.provider)?;

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

    // Get coordinates from CLI args or config file (CLI takes precedence)
    let (lat, lng) = config::resolve_coordinates(args.lat, args.lng, &config)?;
    
    // Determine coordinate source for display
    let coord_source = if args.lat.is_some() && args.lng.is_some() {
        "CLI arguments"
    } else if args.lat.is_some() || args.lng.is_some() {
        "mixed (CLI and config)"
    } else {
        "config file"
    };

    // Display all arguments and their origins
    eprintln!("\n📋 Configuration:");
    eprintln!("   Provider: {} (CLI argument)", args.provider);
    eprintln!("   Days ahead: {} (CLI argument)", args.days_ahead);
    eprintln!("   First day offset: {} (CLI argument)", args.first_day_offset);
    eprintln!("   Timezone: {} ({})", timezone.name(), timezone_source);
    eprintln!("   Coordinates: ({:.6}, {:.6}) ({})", lat, lng, coord_source);
    
    let config_path_display = if let Some(ref path) = args.config {
        path.display().to_string()
    } else {
        config::get_default_config_path()?.display().to_string()
    };
    eprintln!("   Config file: {}", config_path_display);
    eprintln!();

    // Validate timezone against coordinates
    validate_timezone_coordinates(timezone, lat, lng);

    // Fetch weather data using the provider with target timezone
    let weather_points = provider.fetch_weather_data(start, end, lat, lng, timezone).await?;

    // Create response with weather data - no transformation needed
    // WeatherDataPoint now serializes directly with proper formatting
    let transformed_data = TransformedWeatherResponse {
        hours: weather_points,
        meta: create_meta(lat, lng, start, end, provider.name(), timezone),
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
    // println!("{}", serde_json::to_string_pretty(&transformed_data)?);
    
    println!("Loaded {} hourly data points from file.", transformed_data.hours.len());

    Ok(())
}
