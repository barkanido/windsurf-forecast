use anyhow::Result;
use clap::Parser;

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
pub struct Args {
    /// Number of days to forecast ahead (1-7)
    #[arg(long, default_value_t = 4, value_name = "N")]
    pub days_ahead: i32,

    /// Number of days to offset the start date (0-7, 0 for today)
    #[arg(long, default_value_t = 0, value_name = "N")]
    pub first_day_offset: i32,

    /// Weather forecast provider to use
    #[arg(long, default_value = "stormglass", value_name = "PROVIDER")]
    pub provider: String,
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
    match provider_name {
        "stormglass" => Ok(()),
        // Future providers can be added here
        _ => anyhow::bail!(
            "Unknown provider '{}'. Available providers: stormglass",
            provider_name
        ),
    }
}