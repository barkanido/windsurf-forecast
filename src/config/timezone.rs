//! Timezone Configuration and Utilities
//!
//! This module provides timezone-specific configuration handling, including
//! detection, validation, and interactive selection.
//!
//! # Core Concepts
//!
//! ## [`TimezoneConfig`] - Explicit vs Default Tracking
//!
//! The [`TimezoneConfig`] structure tracks whether a timezone was explicitly
//! set by the user or defaulted to UTC. This enables conditional warning
//! messages without repeated prompts.
//!
//!
//! # Special Values
//!
//! ## "LOCAL" - System Timezone Detection
//!
//! The special string `"LOCAL"` (case-sensitive) triggers automatic detection
//! of the system's configured timezone:
//!
//! ```bash
//! windsurf-forecast --timezone LOCAL
//! ```
//!
//! Uses `iana-time-zone` crate for cross-platform timezone detection.
//!
//! # Precedence Rules
//!
//! Timezone resolution follows these rules:
//! 1. **CLI argument** (`--timezone` or `-z`) - highest priority
//! 2. **Config file** (`~/.windsurf-config.toml`) - if not "UTC"
//! 3. **Default to UTC** - with warning message
//!
//! **Note**: Config file "UTC" is ignored (treated as unset) to allow
//! distinguishing between explicit UTC choice and default fallback.
//!
//! # Functions
//!
//! - [`detect_system_timezone()`]: Auto-detect local timezone from OS
//! - [`check_timezone_match()`]: Check timezone matches coordinates
//! - [`pick_timezone_interactive()`]: Launch interactive picker UI
//!
//! # Coordinate Validation
//!
//! The [`check_timezone_match()`] function warns users when the
//! configured timezone doesn't match the location coordinates. This prevents
//! confusing timestamp displays.
//!
//! ```text
//! ⚠️  WARNING: Timezone mismatch detected!
//!    Configured timezone: America/New_York
//!    Detected timezone at coordinates (51.5074, -0.1278): Europe/London
//!    This may cause incorrect timestamp displays.
//! ```
//!
//! # Interactive Picker
//!
//! The [`pick_timezone_interactive()`] function provides a searchable,
//! fuzzy-matching interface for selecting from all 600+ IANA timezones:
//!
//! ```bash
//! windsurf-forecast --pick-timezone
//! ```
//!
//! Features:
//! - Fuzzy search by typing
//! - Arrow key navigation
//! - Automatic persistence to config file
//!
//! # Error Messages
//!
//! Invalid timezone identifiers provide helpful examples:
//! ```text
//! Invalid timezone identifier: 'EST'
//!
//! Examples of valid identifiers:
//! - UTC
//! - LOCAL (use system timezone)
//! - America/New_York
//! - Europe/London
//! - Asia/Jerusalem
//!
//! For a complete list, see:
//! https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
//! ```

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
    
    fn from_string(s: &str) -> Result<Self> {
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
    
    pub fn load_with_precedence(
        cli_tz: Option<&str>,
        config_tz: Option<&str>
    ) -> Result<Self> {
        if let Some(tz_str) = cli_tz {
            return Self::from_string(tz_str);
        }
        
        if let Some(tz_str) = config_tz {
            if tz_str != "UTC" {
                return Self::from_string(tz_str);
            }
        }
        
        Ok(Self::default_utc())
    }
    
    pub fn display_timezone_warning_if_default(&self) {
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

pub fn check_timezone_match(tz: Tz, lat: f64, lng: f64) -> bool {
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