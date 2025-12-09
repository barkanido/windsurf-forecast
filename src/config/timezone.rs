// Timezone-specific configuration and utilities
//
// This module handles timezone detection, validation, and interactive selection:
// - TimezoneConfig: Tracks explicit vs default timezone
// - System timezone detection
// - Coordinate-timezone validation
// - Interactive timezone picker

use anyhow::{anyhow, Context, Result};
use chrono_tz::Tz;

/// Configuration for timezone handling
///
/// Manages user's timezone preferences with precedence:
/// 1. CLI --timezone flag (highest)
/// 2. Config file timezone
/// 3. Default to UTC with warning (lowest)
#[derive(Debug, Clone)]
pub struct TimezoneConfig {
    /// Target timezone for output timestamps
    pub timezone: Tz,
    
    /// Whether timezone was explicitly set by user (vs default)
    /// Used to determine if warning should be displayed
    pub explicit: bool,
}

impl TimezoneConfig {
    /// Create from explicit user input
    pub fn explicit(tz: Tz) -> Self {
        Self {
            timezone: tz,
            explicit: true,
        }
    }
    
    /// Create default (UTC) with warning flag
    fn default_utc() -> Self {
        Self {
            timezone: chrono_tz::UTC,
            explicit: false,
        }
    }
    
    /// Parse timezone from string identifier
    /// Special value "LOCAL" detects system timezone automatically
    fn from_string(s: &str) -> Result<Self> {
        // Handle special "LOCAL" value (case-sensitive)
        if s == "LOCAL" {
            let tz = detect_system_timezone()?;
            return Ok(Self::explicit(tz));
        }
        
        let tz = s.parse::<Tz>()
            .map_err(|_| anyhow!(
                "Invalid timezone identifier: '{}'\n\
                 \n\
                 Examples of valid identifiers:\n\
                 - UTC\n\
                 - LOCAL (use system timezone)\n\
                 - America/New_York\n\
                 - America/Los_Angeles\n\
                 - Europe/London\n\
                 - Europe/Paris\n\
                 - Asia/Jerusalem\n\
                 - Asia/Tokyo\n\
                 - Australia/Sydney\n\
                 \n\
                 For a complete list, see:\n\
                 https://en.wikipedia.org/wiki/List_of_tz_database_time_zones",
                s
            ))?;
        Ok(Self::explicit(tz))
    }
    
    /// Load timezone configuration with standard precedence rules
    ///
    /// Precedence (highest to lowest):
    /// 1. CLI argument (if provided)
    /// 2. Config file timezone (if not "UTC")
    /// 3. Default to UTC with warning
    pub fn load_with_precedence(
        cli_tz: Option<String>,
        config_tz: Option<String>
    ) -> Result<Self> {
        // 1. Try CLI argument
        if let Some(tz_str) = cli_tz {
            return Self::from_string(&tz_str);
        }
        
        // 2. Try config file timezone (ignore "UTC" default)
        if let Some(tz_str) = config_tz {
            if tz_str != "UTC" {
                return Self::from_string(&tz_str);
            }
        }
        
        // 3. Default to UTC (will trigger warning)
        Ok(Self::default_utc())
    }
    
    /// Display warning if using default timezone
    ///
    /// Should be called once during application startup if config.explicit == false.
    /// Writes to stderr to avoid polluting JSON output on stdout.
    pub fn display_default_warning(&self) {
        if !self.explicit {
            eprintln!("Warning: No timezone configured. Using UTC as default.");
            eprintln!("Set timezone via --timezone flag or configure in ~/.windsurf-config.toml");
            eprintln!("Example: --timezone \"America/New_York\"");
            eprintln!("Example: --tz \"Europe/London\" (short form)");
            eprintln!();
        }
    }
}

/// Detect the local system timezone
pub fn detect_system_timezone() -> Result<Tz> {
    let tz_name = iana_time_zone::get_timezone()
        .context("Failed to detect system timezone")?;
    
    let tz = tz_name.parse::<Tz>()
        .map_err(|_| anyhow!(
            "System timezone '{}' is not a valid IANA timezone identifier",
            tz_name
        ))?;
    
    Ok(tz)
}

/// Validate that coordinates are within the timezone's region
/// Returns true if valid, false with warning if mismatch
pub fn validate_timezone_coordinates(tz: Tz, lat: f64, lng: f64) -> bool {
    let finder = tzf_rs::DefaultFinder::new();
    let detected_tz = finder.get_tz_name(lng, lat);
    
    let tz_name = tz.name();
    if detected_tz != tz_name {
        eprintln!("\n⚠️  WARNING: Timezone mismatch detected!");
        eprintln!("   Configured timezone: {}", tz_name);
        eprintln!("   Detected timezone at coordinates ({:.6}, {:.6}): {}", lat, lng, detected_tz);
        eprintln!("   This may cause incorrect timestamp displays.\n");
        return false;
    }
    true
}

/// Launch interactive timezone picker
pub fn pick_timezone_interactive() -> Result<String> {
    use dialoguer::{theme::ColorfulTheme, FuzzySelect};
    
    let timezones: Vec<String> = chrono_tz::TZ_VARIANTS
        .iter()
        .map(|tz| tz.name().to_string())
        .collect();
    
    println!("\n🌍 Select your timezone:");
    println!("   (Type to search, use arrow keys to navigate, Enter to select)\n");
    
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Timezone")
        .default(0)
        .items(&timezones)
        .interact()
        .context("Failed to get timezone selection")?;
    
    let selected_tz = timezones[selection].clone();
    
    println!("\n✓ Selected timezone: {}\n", selected_tz);
    
    Ok(selected_tz)
}