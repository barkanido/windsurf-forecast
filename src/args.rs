use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser, Debug)]
#[command(
    name = "windsurf-forecast",
    about = "Fetch weather forecast data from Storm Glass API",
    long_about = None,
    after_help = "Examples:\n  \
    # Get 4-day forecast starting today (default)\n  \
    windsurf-forecast\n\n  \
    # Get 3-day forecast starting today\n  \
    windsurf-forecast --days-ahead 3\n\n  \
    # Get 2-day forecast starting 3 days from now\n  \
    windsurf-forecast --days-ahead 2 --first-day-offset 3\n\n  \
    # Get 5-day forecast starting tomorrow\n  \
    windsurf-forecast --days-ahead 5 --first-day-offset 1\n\n  \
    # Use a specific timezone\n  \
    windsurf-forecast --timezone America/New_York\n\n  \
    # Use system/local timezone\n  \
    windsurf-forecast --timezone LOCAL\n\n  \
    # Pick timezone interactively\n  \
    windsurf-forecast --pick-timezone\n\n  \
    # Specify custom coordinates\n  \
    windsurf-forecast --lat 40.7128 --lng -74.0060\n\n  \
    # Use custom config file\n  \
    windsurf-forecast --config /path/to/config.toml\n\n\
    Note: days-ahead + first-day-offset must not exceed 7 to ensure reliable forecasts."
)]
pub struct Args {
    /// Number of days to forecast ahead (1-7)
    #[arg(long, default_value_t = 4, value_name = "N")]
    pub days_ahead: i32,

    /// Number of days to offset the start date (0-7, 0 for today)
    #[arg(long, default_value_t = 0, value_name = "N")]
    pub first_day_offset: i32,

    /// Weather forecast provider to use
    #[arg(
        long,
        default_value = "stormglass",
        value_name = "PROVIDER",
        help = get_provider_help()
    )]
    pub provider: String,

    /// Timezone for displaying timestamps (e.g., "UTC", "LOCAL", "America/New_York", "Asia/Jerusalem")
    /// Use "LOCAL" to automatically detect system timezone. Overrides timezone from config file and is persisted.
    #[arg(long, short = 'z', value_name = "TIMEZONE")]
    pub timezone: Option<String>,

    /// Launch interactive timezone picker and save selection to config
    #[arg(long, conflicts_with = "timezone")]
    pub pick_timezone: bool,

    /// List all available weather providers and exit
    #[arg(long)]
    pub list_providers: bool,

    /// Path to custom config file (default: ~/.windsurf-config.toml)
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Latitude for the forecast location
    #[arg(long, value_name = "LAT")]
    pub lat: Option<f64>,

    /// Longitude for the forecast location
    #[arg(long, value_name = "LNG")]
    pub lng: Option<f64>,

    /// Save configuration to file after successful execution
    /// (Applies to provider, timezone, and coordinates)
    #[arg(long)]
    pub save: bool,
}

// ============================================================================
// Validation Functions
// ============================================================================

pub fn validate_args(args: &Args) -> Result<()> {
    validate_days_range(args)?;
    validate_provider(&args.provider)?;

    Ok(())
}

fn validate_days_range(args: &Args) -> Result<()> {
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

    Ok(())
}

pub fn validate_provider(provider_name: &str) -> Result<()> {
    crate::provider_registry::validate_provider_name(provider_name)
}

/// Generate dynamic help text for provider argument
fn get_provider_help() -> String {
    let providers: Vec<_> = crate::provider_registry::all_provider_descriptions()
        .map(|(name, desc)| format!("{}: {}", name, desc))
        .collect();
    
    if providers.is_empty() {
        "Weather forecast provider to use".to_string()
    } else {
        format!(
            "Weather forecast provider to use\nAvailable providers:\n  {}",
            providers.join("\n  ")
        )
    }
}