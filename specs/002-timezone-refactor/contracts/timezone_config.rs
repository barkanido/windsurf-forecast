// Contract: Timezone configuration structures
// Feature: 002-timezone-refactor
// 
// This file defines configuration structures for timezone handling,
// including parsing from CLI arguments and environment variables.

use anyhow::{anyhow, Result};
use chrono_tz::Tz;
use std::env;

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
    /// 
    /// This should trigger a warning message to stderr recommending
    /// the user configure an explicit timezone.
    pub fn default_utc() -> Self {
        Self {
            timezone: chrono_tz::UTC,
            explicit: false,
        }
    }
    
    /// Parse timezone from string identifier
    /// 
    /// # Arguments
    /// * `s` - IANA timezone identifier (e.g., "America/New_York", "UTC")
    /// 
    /// # Returns
    /// TimezoneConfig with explicit=true if parsing succeeds
    /// 
    /// # Errors
    /// Returns actionable error message if timezone identifier is invalid,
    /// including examples of valid identifiers.
    /// 
    /// # Example
    /// ```rust
    /// let config = TimezoneConfig::from_string("America/New_York")?;
    /// assert_eq!(config.timezone, chrono_tz::America::New_York);
    /// assert!(config.explicit);
    /// ```
    pub fn from_string(s: &str) -> Result<Self> {
        let tz = s.parse::<Tz>()
            .map_err(|_| anyhow!(
                "Invalid timezone identifier: '{}'\n\
                 \n\
                 Examples of valid identifiers:\n\
                 - UTC\n\
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
    /// 2. Environment variable FORECAST_TIMEZONE (if set)
    /// 3. Default to UTC with warning
    /// 
    /// # Arguments
    /// * `cli_tz` - Timezone from CLI --timezone flag (Option<String>)
    /// 
    /// # Returns
    /// TimezoneConfig with appropriate timezone and explicit flag
    /// 
    /// # Example
    /// ```rust
    /// // From CLI argument
    /// let config = TimezoneConfig::load_with_precedence(Some("America/New_York".to_string()))?;
    /// 
    /// // From environment variable
    /// env::set_var("FORECAST_TIMEZONE", "Europe/London");
    /// let config = TimezoneConfig::load_with_precedence(None)?;
    /// 
    /// // Default (UTC)
    /// env::remove_var("FORECAST_TIMEZONE");
    /// let config = TimezoneConfig::load_with_precedence(None)?;
    /// assert_eq!(config.timezone, chrono_tz::UTC);
    /// assert!(!config.explicit); // Triggers warning
    /// ```
    pub fn load_with_precedence(cli_tz: Option<String>) -> Result<Self> {
        // 1. Try CLI argument first
        if let Some(tz_str) = cli_tz {
            return Self::from_string(&tz_str);
        }
        
        // 2. Try environment variable
        if let Ok(tz_str) = env::var("FORECAST_TIMEZONE") {
            return Self::from_string(&tz_str);
        }
        
        // 3. Default to UTC (will trigger warning)
        Ok(Self::default_utc())
    }
    
    /// Display warning if using default timezone
    /// 
    /// Should be called once during application startup if config.explicit == false.
    /// Writes to stderr to avoid polluting JSON output on stdout.
    /// 
    /// # Example
    /// ```rust
    /// let config = TimezoneConfig::load_with_precedence(None)?;
    /// if !config.explicit {
    ///     config.display_default_warning();
    /// }
    /// ```
    pub fn display_default_warning(&self) {
        if !self.explicit {
            eprintln!("Warning: No timezone configured. Using UTC as default.");
            eprintln!("Set timezone via --timezone flag or FORECAST_TIMEZONE environment variable.");
            eprintln!("Example: --timezone \"America/New_York\"");
            eprintln!("Example: --tz \"Europe/London\" (short form)");
            eprintln!();
        }
    }
}

// ============================================================================
// Integration with CLI Arguments
// ============================================================================

/// Example integration with clap Args structure
/// 
/// Add these fields to your Args struct:
/// ```rust
/// use clap::Parser;
/// 
/// #[derive(Parser)]
/// pub struct Args {
///     /// Timezone for displaying timestamps (e.g., "UTC", "America/New_York")
///     /// Overrides FORECAST_TIMEZONE environment variable
///     #[arg(long, short = 'z', value_name = "TIMEZONE")]
///     pub timezone: Option<String>,
///     
///     // ... other fields
/// }
/// ```
/// 
/// Then load configuration:
/// ```rust
/// let args = Args::parse();
/// let tz_config = TimezoneConfig::load_with_precedence(args.timezone)?;
/// tz_config.display_default_warning();
/// ```

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_explicit_timezone() {
        let tz: Tz = "America/New_York".parse().unwrap();
        let config = TimezoneConfig::explicit(tz);
        
        assert_eq!(config.timezone, tz);
        assert!(config.explicit);
    }
    
    #[test]
    fn test_default_utc() {
        let config = TimezoneConfig::default_utc();
        
        assert_eq!(config.timezone, chrono_tz::UTC);
        assert!(!config.explicit);
    }
    
    #[test]
    fn test_from_string_valid() {
        let config = TimezoneConfig::from_string("America/New_York").unwrap();
        
        assert_eq!(config.timezone, chrono_tz::America::New_York);
        assert!(config.explicit);
    }
    
    #[test]
    fn test_from_string_invalid() {
        let result = TimezoneConfig::from_string("Invalid/Timezone");
        
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid timezone identifier"));
        assert!(err_msg.contains("Examples of valid identifiers"));
    }
    
    #[test]
    fn test_precedence_cli_wins() {
        env::set_var("FORECAST_TIMEZONE", "Europe/London");
        
        let config = TimezoneConfig::load_with_precedence(
            Some("America/New_York".to_string())
        ).unwrap();
        
        // CLI should win over environment variable
        assert_eq!(config.timezone, chrono_tz::America::New_York);
        assert!(config.explicit);
        
        env::remove_var("FORECAST_TIMEZONE");
    }
    
    #[test]
    fn test_precedence_env_second() {
        env::set_var("FORECAST_TIMEZONE", "Europe/London");
        
        let config = TimezoneConfig::load_with_precedence(None).unwrap();
        
        assert_eq!(config.timezone, chrono_tz::Europe::London);
        assert!(config.explicit);
        
        env::remove_var("FORECAST_TIMEZONE");
    }
    
    #[test]
    fn test_precedence_default_last() {
        env::remove_var("FORECAST_TIMEZONE");
        
        let config = TimezoneConfig::load_with_precedence(None).unwrap();
        
        assert_eq!(config.timezone, chrono_tz::UTC);
        assert!(!config.explicit);
    }
    
    #[test]
    fn test_utc_parsing() {
        let config = TimezoneConfig::from_string("UTC").unwrap();
        assert_eq!(config.timezone, chrono_tz::UTC);
    }
    
    #[test]
    fn test_various_timezones() {
        let timezones = vec![
            ("America/Los_Angeles", chrono_tz::America::Los_Angeles),
            ("Asia/Jerusalem", chrono_tz::Asia::Jerusalem),
            ("Asia/Tokyo", chrono_tz::Asia::Tokyo),
            ("Australia/Sydney", chrono_tz::Australia::Sydney),
            ("Europe/Paris", chrono_tz::Europe::Paris),
        ];
        
        for (tz_str, expected_tz) in timezones {
            let config = TimezoneConfig::from_string(tz_str).unwrap();
            assert_eq!(config.timezone, expected_tz, "Failed for {}", tz_str);
        }
    }
}