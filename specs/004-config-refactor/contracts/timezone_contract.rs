// API Contract: Timezone Configuration
// Feature: 004-config-refactor
// Location: src/config/timezone.rs

use anyhow::{anyhow, Context, Result};
use chrono_tz::Tz;

// ============================================================================
// CONTRACT: TimezoneConfig Structure (MOVED from src/config.rs)
// ============================================================================

/// Configuration for timezone handling
///
/// CONTRACT GUARANTEES:
/// - Tracks whether timezone was explicitly set (vs default)
/// - Used to determine if warning should be displayed
/// - Immutable after creation
///
/// PRECEDENCE:
/// 1. CLI --timezone flag (highest)
/// 2. Config file timezone
/// 3. Default to UTC with warning (lowest)
#[derive(Debug, Clone)]
pub struct TimezoneConfig {
    /// Target timezone for output timestamps
    pub timezone: Tz,
    
    /// Whether timezone was explicitly set by user (vs default)
    pub explicit: bool,
}

impl TimezoneConfig {
    /// Create from explicit user input
    ///
    /// SIGNATURE:
    /// pub fn explicit(tz: Tz) -> Self
    pub fn explicit(tz: Tz) -> Self {
        Self {
            timezone: tz,
            explicit: true,
        }
    }
    
    /// Create default (UTC) with warning flag
    ///
    /// SIGNATURE:
    /// fn default_utc() -> Self
    fn default_utc() -> Self {
        Self {
            timezone: chrono_tz::UTC,
            explicit: false,
        }
    }
    
    /// Parse timezone from string identifier
    ///
    /// CONTRACT GUARANTEES:
    /// - Special value "LOCAL" detects system timezone
    /// - Invalid identifiers return error with examples
    /// - Case-sensitive for "LOCAL"
    ///
    /// SIGNATURE:
    /// fn from_string(s: &str) -> Result<Self>
    ///
    /// ERRORS:
    /// - Invalid identifier: "Invalid timezone identifier: '{s}'\n[list of valid examples]"
    fn from_string(s: &str) -> Result<Self> {
        // Handle special "LOCAL" value
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
    /// CONTRACT GUARANTEES:
    /// - Precedence: CLI > Config > Default
    /// - Default UTC does NOT trigger explicit flag (warning will display)
    /// - Config timezone of "UTC" is treated as not configured
    ///
    /// SIGNATURE:
    /// pub fn load_with_precedence(
    ///     cli_tz: Option<String>,
    ///     config_tz: Option<String>
    /// ) -> Result<Self>
    ///
    /// EXAMPLES:
    /// ```
    /// // CLI provided: explicit, no warning
    /// let config = TimezoneConfig::load_with_precedence(
    ///     Some("America/New_York".to_string()),
    ///     None
    /// )?;
    /// assert!(config.explicit);
    ///
    /// // Config provided (not "UTC"): explicit, no warning
    /// let config = TimezoneConfig::load_with_precedence(
    ///     None,
    ///     Some("Asia/Jerusalem".to_string())
    /// )?;
    /// assert!(config.explicit);
    ///
    /// // No source or config="UTC": default, warning displayed
    /// let config = TimezoneConfig::load_with_precedence(None, None)?;
    /// assert!(!config.explicit);
    /// config.display_default_warning();  // Shows warning
    /// ```
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
    /// CONTRACT GUARANTEES:
    /// - Only displays if explicit == false
    /// - Writes to stderr (not stdout, to avoid polluting JSON output)
    /// - Shows once during application startup
    /// - Provides examples of how to set timezone
    ///
    /// SIGNATURE:
    /// pub fn display_default_warning(&self)
    ///
    /// OUTPUT (stderr):
    /// ```
    /// Warning: No timezone configured. Using UTC as default.
    /// Set timezone via --timezone flag or configure in ~/.windsurf-config.toml
    /// Example: --timezone "America/New_York"
    /// Example: --tz "Europe/London" (short form)
    /// ```
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
// CONTRACT: System Timezone Detection
// ============================================================================

/// Detect the local system timezone
///
/// CONTRACT GUARANTEES:
/// - Uses iana-time-zone crate for detection
/// - Returns valid IANA timezone identifier
/// - Fails with helpful error if detection fails
///
/// SIGNATURE:
/// pub fn detect_system_timezone() -> Result<Tz>
///
/// ERRORS:
/// - Detection failure: "Failed to detect system timezone"
/// - Invalid timezone: "System timezone '{name}' is not a valid IANA timezone identifier"
///
/// EXAMPLES:
/// ```
/// let tz = detect_system_timezone()?;
/// // On macOS: America/Los_Angeles
/// // On Windows: America/New_York (based on system settings)
/// ```
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

// ============================================================================
// CONTRACT: Timezone Validation
// ============================================================================

/// Validate that coordinates are within the timezone's region
///
/// CONTRACT GUARANTEES:
/// - Uses tzf-rs for geographic timezone lookup
/// - Returns true if match, false with warning if mismatch
/// - Warning written to stderr (not stdout)
/// - Non-blocking (returns false but doesn't error)
///
/// SIGNATURE:
/// pub fn validate_timezone_coordinates(tz: Tz, lat: f64, lng: f64) -> bool
///
/// OUTPUT (stderr, if mismatch):
/// ```
/// ⚠️  WARNING: Timezone mismatch detected!
///    Configured timezone: America/New_York
///    Detected timezone at coordinates (40.712800, -74.006000): America/New_York
///    This may cause incorrect timestamp displays.
/// ```
///
/// EXAMPLES:
/// ```
/// // Matching timezone and coordinates
/// let valid = validate_timezone_coordinates(
///     chrono_tz::America::New_York,
///     40.7128,
///     -74.0060
/// );
/// assert!(valid);
///
/// // Mismatched timezone and coordinates (warning displayed)
/// let valid = validate_timezone_coordinates(
///     chrono_tz::Europe::London,
///     40.7128,
///     -74.0060
/// );
/// assert!(!valid);  // Returns false but doesn't error
/// ```
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

// ============================================================================
// CONTRACT: Interactive Timezone Picker
// ============================================================================

/// Launch interactive timezone picker
///
/// CONTRACT GUARANTEES:
/// - Uses dialoguer crate for fuzzy search UI
/// - Returns selected timezone as string identifier
/// - Supports type-to-search, arrow keys, Enter to select
/// - Provides user feedback on selection
///
/// SIGNATURE:
/// pub fn pick_timezone_interactive() -> Result<String>
///
/// ERRORS:
/// - User cancels: "Failed to get timezone selection"
/// - Terminal not interactive: (from dialoguer)
///
/// OUTPUT (stdout):
/// ```
/// 🌍 Select your timezone:
///    (Type to search, use arrow keys to navigate, Enter to select)
///
/// Timezone: [fuzzy searchable list]
///
/// ✓ Selected timezone: America/New_York
/// ```
///
/// EXAMPLES:
/// ```
/// // User types "new york" and selects America/New_York
/// let tz_string = pick_timezone_interactive()?;
/// assert_eq!(tz_string, "America/New_York");
/// ```
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

// ============================================================================
// CONTRACT: Public API
// ============================================================================

/// Required public exports from src/config/timezone.rs module
///
/// BACKWARD COMPATIBILITY:
/// - All functions moved from src/config.rs without changes
/// - TimezoneConfig structure unchanged
pub mod exports {
    pub use super::{
        TimezoneConfig,
        detect_system_timezone,
        validate_timezone_coordinates,
        pick_timezone_interactive,
    };
}

// ============================================================================
// CONTRACT: Testing Requirements
// ============================================================================

#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn explicit_timezone_config_sets_flag() {
        // CONTRACT: Explicit constructor sets explicit=true
        let config = TimezoneConfig::explicit(chrono_tz::UTC);
        assert!(config.explicit);
        assert_eq!(config.timezone, chrono_tz::UTC);
    }

    #[test]
    fn default_utc_config_clears_flag() {
        // CONTRACT: Default constructor sets explicit=false
        let config = TimezoneConfig::default_utc();
        assert!(!config.explicit);
        assert_eq!(config.timezone, chrono_tz::UTC);
    }

    #[test]
    fn from_string_parses_valid_timezone() {
        // CONTRACT: Valid timezone string creates explicit config
        let config = TimezoneConfig::from_string("America/New_York").unwrap();
        assert!(config.explicit);
        assert_eq!(config.timezone.name(), "America/New_York");
    }

    #[test]
    fn from_string_handles_local_keyword() {
        // CONTRACT: "LOCAL" triggers system timezone detection
        let config = TimezoneConfig::from_string("LOCAL").unwrap();
        assert!(config.explicit);
        // Actual timezone depends on system
    }

    #[test]
    fn from_string_rejects_invalid_timezone() {
        // CONTRACT: Invalid timezone returns error with examples
        let result = TimezoneConfig::from_string("Invalid/Timezone");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid timezone identifier"));
        assert!(err_msg.contains("UTC"));
        assert!(err_msg.contains("America/New_York"));
    }

    #[test]
    fn load_with_precedence_prefers_cli() {
        // CONTRACT: CLI overrides config
        let config = TimezoneConfig::load_with_precedence(
            Some("America/New_York".to_string()),
            Some("Europe/London".to_string())
        ).unwrap();
        assert_eq!(config.timezone.name(), "America/New_York");
        assert!(config.explicit);
    }

    #[test]
    fn load_with_precedence_uses_config_if_no_cli() {
        // CONTRACT: Config used if CLI not provided
        let config = TimezoneConfig::load_with_precedence(
            None,
            Some("Europe/London".to_string())
        ).unwrap();
        assert_eq!(config.timezone.name(), "Europe/London");
        assert!(config.explicit);
    }

    #[test]
    fn load_with_precedence_defaults_to_utc() {
        // CONTRACT: Defaults to UTC if neither CLI nor config
        let config = TimezoneConfig::load_with_precedence(None, None).unwrap();
        assert_eq!(config.timezone, chrono_tz::UTC);
        assert!(!config.explicit);
    }

    #[test]
    fn load_with_precedence_ignores_utc_in_config() {
        // CONTRACT: Config timezone "UTC" is treated as not configured
        let config = TimezoneConfig::load_with_precedence(
            None,
            Some("UTC".to_string())
        ).unwrap();
        assert_eq!(config.timezone, chrono_tz::UTC);
        assert!(!config.explicit);  // Should trigger warning
    }

    #[test]
    fn display_default_warning_only_when_not_explicit() {
        // CONTRACT: Warning only displayed if explicit==false
        let explicit_config = TimezoneConfig::explicit(chrono_tz::UTC);
        let default_config = TimezoneConfig::default_utc();
        
        // These should not panic (warnings go to stderr)
        explicit_config.display_default_warning();  // No output
        default_config.display_default_warning();   // Displays warning
    }
}