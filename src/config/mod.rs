//! Configuration Module
//!
//! This module provides a unified interface for managing application configuration
//! with clear separation of concerns across four submodules:
//!
//! # Module Organization
//!
//! ## [`types`] - Data Structures
//! - [`ResolvedConfig`]: Final validated configuration ready for application use
//! - `ConfigSources`: Raw input sources (CLI, config file, defaults)
//! - `ConfigSource`: Enum for tracking where values originated (for error messages)
//!
//! ## [`loader`] - File I/O Operations
//! - Load configuration from TOML file (`~/.windsurf-config.toml`)
//! - Save configuration to TOML file (when `--save` flag provided)
//! - Handle default paths and file system operations
//!
//! ## [`resolver`] - Precedence Resolution and Validation
//! - Apply precedence rules: CLI > Config File > Default
//! - Validate all configuration values (coordinates, date ranges, providers)
//! - Provide main entry point: [`resolve_from_args_and_file()`]
//!
//! ## [`timezone`] - Timezone Configuration
//! - Detect system timezone
//! - Validate timezone against coordinates
//! - Interactive timezone picker
//!
//! # Configuration Flow
//!
//! ```text
//! CLI Args + Config File + Defaults
//!           ↓
//!    resolve_from_args_and_file()
//!           ↓
//!     ResolvedConfig (validated)
//!           ↓
//!    Application execution
//!           ↓
//!  (Optional) save_config_from_resolved() if --save flag
//! ```
//!
//! # Precedence Rules
//!
//! All configuration parameters follow the same precedence order:
//! 1. **CLI arguments** (highest priority) - temporary overrides for this execution
//! 2. **Config file** (`~/.windsurf-config.toml`) - persistent user preferences
//! 3. **Defaults** (lowest priority) - fallback values
//!
//! Exception: Coordinates (lat/lng) have NO defaults and must be provided via CLI or config.
//!
//! # Persistence Policy
//!
//! Configuration is **NOT** automatically saved. Users must explicitly provide the
//! `--save` flag to persist CLI arguments to the config file. This prevents surprising
//! behavior where running a command once with different flags permanently changes settings.
//! ```

pub mod types;
pub mod loader;
pub mod resolver;
pub mod timezone;

// Re-export commonly used items for convenience
pub use types::ResolvedConfig;
pub use resolver::resolve_from_args_and_file;
pub use loader::{load_config_from_file, save_config, get_default_config_path};
pub use timezone::{check_timezone_match, pick_timezone_interactive};

use anyhow::Result;
use std::path::PathBuf;

/// Save resolved configuration to file
///
/// Converts ResolvedConfig back to Config structure and persists to TOML file.
/// Only called when user provides --save flag.
pub fn save_config_from_resolved(
    resolved: &ResolvedConfig,
    path: Option<&PathBuf>
) -> Result<()> {
    let config = loader::Config {
        general: loader::GeneralConfig {
            timezone: resolved.timezone.name().to_string(),
            default_provider: resolved.provider.clone(),
            lat: Some(resolved.lat),
            lng: Some(resolved.lng),
        },
    };
    
    loader::save_config(&config, path)?;
    
    let config_path = path
        .cloned()
        .unwrap_or_else(|| loader::get_default_config_path().unwrap());
    eprintln!("✓ Configuration saved to {}", config_path.display());
    
    Ok(())
}