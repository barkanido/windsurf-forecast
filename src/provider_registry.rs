//! Centralized provider registry using inventory pattern
//! 
//! This module provides automatic provider discovery and registration.
//! Providers self-register using the `inventory::submit!()` macro.

use anyhow::{anyhow, Result};
use crate::forecast_provider::ForecastProvider;
use std::collections::HashMap;


pub struct ProviderMetadata {
    /// Unique identifier for the provider (e.g., "stormglass", "openweathermap")
    /// Must be lowercase alphanumeric with optional hyphens/underscores
    pub name: &'static str,
    
    /// Human-readable description for CLI help text
    /// Should be concise (< 100 chars recommended)
    pub description: &'static str,
    
    /// Environment variable name for the provider's API key
    /// (e.g., "STORMGLASS_API_KEY", "OPEN_WEATHER_MAP_API_KEY")
    pub api_key_var: &'static str,
    
    /// Factory function that creates an instance of this provider
    /// 
    /// The function is responsible for:
    /// - Retrieving its own API key from environment
    /// - Constructing the provider instance
    /// - Returning appropriate errors if configuration is missing
    /// 
    /// # Errors
    /// Returns error if API key is not set or provider initialization fails
    pub instantiate: fn() -> Result<Box<dyn ForecastProvider>>,
}

// Enable inventory collection of ProviderMetadata
inventory::collect!(ProviderMetadata);


pub fn get_provider_metadata(name: &str) -> Option<&'static ProviderMetadata> {
    inventory::iter::<ProviderMetadata>()
        .find(|meta| meta.name == name)
}


pub fn all_provider_names() -> impl Iterator<Item = &'static str> {
    inventory::iter::<ProviderMetadata>()
        .map(|meta| meta.name)
}


pub fn all_provider_descriptions() -> impl Iterator<Item = (&'static str, &'static str)> {
    inventory::iter::<ProviderMetadata>()
        .map(|meta| (meta.name, meta.description))
}


pub fn create_provider(name: &str) -> Result<Box<dyn ForecastProvider>> {
    match get_provider_metadata(name) {
        Some(meta) => (meta.instantiate)(),
        None => {
            let available: Vec<_> = all_provider_names().collect();
            Err(anyhow!(
                "Unknown provider '{}'. Available providers: {}",
                name,
                available.join(", ")
            ))
        }
    }
}


pub fn validate_provider_name(name: &str) -> Result<()> {
    if get_provider_metadata(name).is_some() {
        Ok(())
    } else {
        let available: Vec<_> = all_provider_names().collect();
        Err(anyhow!(
            "Unknown provider '{}'. Available providers: {}",
            name,
            available.join(", ")
        ))
    }
}

/// Check for duplicate provider names in the registry
/// 
/// This function should be called early in main() to detect programming errors
/// where two providers registered with the same name.
/// 
/// # Panics
/// Panics if duplicate provider names are detected, with message listing:
/// - The duplicate provider name
/// - Module paths of both conflicting registrations
/// 
/// This is intentional - duplicates indicate a programming error that must be
/// fixed before the application can run.
/// ```
pub fn check_duplicates() {
    let mut seen = HashMap::new();
    for meta in inventory::iter::<ProviderMetadata>() {
        if let Some(existing) = seen.insert(meta.name, meta) {
            panic!(
                "FATAL: Duplicate provider name '{}' detected in registry!\n\
                This indicates a programming error where two provider modules \
                registered with the same name.\n\
                First registration: {} ({})\n\
                Second registration: {} ({})",
                meta.name,
                existing.description,
                existing.api_key_var,
                meta.description,
                meta.api_key_var
            );
        }
    }
}