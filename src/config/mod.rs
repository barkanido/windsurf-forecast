// Configuration module
//
// This module consolidates all configuration-related logic:
// - types: Core data structures (ResolvedConfig, ConfigSources)
// - loader: File I/O operations (load/save TOML)
// - resolver: Precedence resolution and validation
// - timezone: Timezone-specific configuration

pub mod types;
pub mod loader;
pub mod resolver;
pub mod timezone;

// Re-export commonly used items for convenience
pub use types::ResolvedConfig;
pub use resolver::resolve_from_args_and_file;
pub use loader::{load_config, save_config, get_default_config_path, Config, GeneralConfig};
pub use timezone::{TimezoneConfig, validate_timezone_coordinates, pick_timezone_interactive};
pub use resolver::{resolve_coordinates, validate_coordinates};

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