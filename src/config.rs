use anyhow::{Context, Result};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// Configuration Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
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

/// Create default config file if it doesn't exist
pub fn ensure_default_config() -> Result<PathBuf> {
    let config_path = get_default_config_path()?;
    
    if !config_path.exists() {
        let default_config = Config::default();
        save_config(&default_config, Some(&config_path))?;
    }
    
    Ok(config_path)
}

// ============================================================================
// Timezone Validation
// ============================================================================

/// Parse and validate timezone string
pub fn parse_timezone(tz_string: &str) -> Result<Tz> {
    tz_string.parse::<Tz>()
        .map_err(|_| anyhow::anyhow!("Invalid timezone: '{}'. Use standard IANA timezone names (e.g., 'UTC', 'America/New_York', 'Asia/Jerusalem')", tz_string))
}

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

/// Check if timezone is the default UTC and was not explicitly set
pub fn warn_if_default_timezone(tz_string: &str, was_explicitly_set: bool) {
    if tz_string == "UTC" && !was_explicitly_set {
        eprintln!("\n⚠️  WARNING: Using default timezone 'UTC'");
        eprintln!("   Set a timezone with --timezone <TZ> or run --pick-timezone for interactive selection");
        eprintln!("   The timezone will be saved to your config file for future use.\n");
    }
}

// ============================================================================
// Interactive Timezone Picker
// ============================================================================

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