use anyhow::{anyhow, Context, Result};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// System Timezone Detection
// ============================================================================

/// Detect the local system timezone
pub fn detect_system_timezone() -> Result<Tz> {
    // Try to get timezone from system
    let tz_name = iana_time_zone::get_timezone()
        .context("Failed to detect system timezone")?;
    
    let tz = tz_name.parse::<Tz>()
        .map_err(|_| anyhow!(
            "System timezone '{}' is not a valid IANA timezone identifier",
            tz_name
        ))?;
    
    Ok(tz)
}

// ============================================================================
// Timezone Configuration
// ============================================================================

/// Configuration for timezone handling
///
/// Manages user's timezone preferences with precedence:
/// 1. CLI --timezone flag (highest)
/// 2. Environment variable FORECAST_TIMEZONE
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
    /// 2. Config file timezone
    /// 3. Default to UTC with warning
    pub fn load_with_precedence(cli_tz: Option<String>, config_tz: Option<String>) -> Result<Self> {
        // 1. Try CLI argument
        if let Some(tz_str) = cli_tz {
            return Self::from_string(&tz_str);
        }
        
        // 2. Try config file timezone
        if let Some(tz_str) = config_tz {
            // Only use config if it's not the default "UTC"
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

// ============================================================================
// Configuration Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Timezone for displaying timestamps (e.g., "UTC", "Asia/Jerusalem")
    #[serde(default = "default_timezone")]
    pub timezone: String,
    
    /// Default weather provider
    #[serde(default = "default_provider")]
    pub default_provider: String,
    
    /// Latitude for forecast location (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    
    /// Longitude for forecast location (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
}

// ============================================================================
// Default Values
// ============================================================================

fn default_timezone() -> String {
    "UTC".to_string()
}

fn default_provider() -> String {
    "stormglass".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            timezone: default_timezone(),
            default_provider: default_provider(),
            lat: None,
            lng: None,
        }
    }
}

// ============================================================================
// Configuration File Management
// ============================================================================

/// Get the default config file path
pub fn get_default_config_path() -> Result<PathBuf> {
    // Try home directory first, fallback to current directory
    if let Some(home) = dirs::home_dir() {
        Ok(home.join(".windsurf-config.toml"))
    } else {
        Ok(PathBuf::from("windsurf-config.toml"))
    }
}

/// Load configuration from file
pub fn load_config(path: Option<&PathBuf>) -> Result<Config> {
    let config_path = if let Some(p) = path {
        p.clone()
    } else {
        get_default_config_path()?
    };

    if !config_path.exists() {
        // Return default config if file doesn't exist
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&config_path)
        .context(format!("Failed to read config file: {}", config_path.display()))?;
    
    let config: Config = toml::from_str(&contents)
        .context(format!("Failed to parse config file: {}", config_path.display()))?;
    
    Ok(config)
}

/// Save configuration to file
pub fn save_config(config: &Config, path: Option<&PathBuf>) -> Result<()> {
    let config_path = if let Some(p) = path {
        p.clone()
    } else {
        get_default_config_path()?
    };

    let toml_string = toml::to_string_pretty(config)
        .context("Failed to serialize config to TOML")?;
    
    fs::write(&config_path, toml_string)
        .context(format!("Failed to write config file: {}", config_path.display()))?;
    
    Ok(())
}

// ============================================================================
// Coordinate Validation & Precedence
// ============================================================================

/// Validate coordinate ranges
///
/// # Arguments
/// * `lat` - Latitude value to validate
/// * `lng` - Longitude value to validate
///
/// # Returns
/// * `Ok(())` - If coordinates are within valid ranges
/// * `Err` - If coordinates are out of bounds
pub fn validate_coordinates(lat: f64, lng: f64) -> Result<()> {
    if !(-90.0..=90.0).contains(&lat) {
        anyhow::bail!(
            "Latitude {} is out of valid range. Must be between -90.0 and 90.0 degrees.",
            lat
        );
    }
    
    if !(-180.0..=180.0).contains(&lng) {
        anyhow::bail!(
            "Longitude {} is out of valid range. Must be between -180.0 and 180.0 degrees.",
            lng
        );
    }
    
    Ok(())
}

/// Apply coordinate precedence: CLI args override config file values
///
/// This function implements the precedence logic used in main.rs where
/// CLI-provided coordinates take precedence over config file coordinates.
/// Also validates that coordinates are within valid ranges.
///
/// # Arguments
/// * `cli_lat` - Latitude from CLI argument (--lat)
/// * `cli_lng` - Longitude from CLI argument (--lng)
/// * `config` - Config loaded from file
///
/// # Returns
/// * `Ok((lat, lng))` - Final validated coordinates to use
/// * `Err` - If neither CLI nor config provide coordinates, or if coordinates are invalid
pub fn resolve_coordinates(
    cli_lat: Option<f64>,
    cli_lng: Option<f64>,
    config: &Config,
) -> Result<(f64, f64)> {
    let lat = cli_lat
        .or(config.general.lat)
        .ok_or_else(|| anyhow!(
            "Latitude not specified. Provide via --lat argument or configure in config file."
        ))?;
    
    let lng = cli_lng
        .or(config.general.lng)
        .ok_or_else(|| anyhow!(
            "Longitude not specified. Provide via --lng argument or configure in config file."
        ))?;
    
    // Validate coordinate ranges
    validate_coordinates(lat, lng)?;
    
    Ok((lat, lng))
}

// ============================================================================
// Timezone Validation & Interactive Selection
// ============================================================================

/// Validate that coordinates are within the timezone's region
/// Returns true if valid, false with warning if mismatch
pub fn validate_timezone_coordinates(tz: Tz, lat: f64, lng: f64) -> bool {
    // Use tzf-rs to find timezone at coordinates
    let finder = tzf_rs::DefaultFinder::new();
    let detected_tz = finder.get_tz_name(lng, lat);
    
    // Compare timezone names
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
    
    // Get all available timezones
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