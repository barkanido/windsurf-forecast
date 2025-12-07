// API Contract: Provider Registry Module
// 
// This file defines the public interface contract for the provider registry module.
// These are the functions and types that will be exposed by src/provider_registry.rs

use anyhow::Result;
use crate::forecast_provider::ForecastProvider;

/// Metadata describing a registered weather provider
/// 
/// Providers submit instances of this struct via `inventory::submit!()` macro
/// to register themselves with the central registry.
/// 
/// # Example Registration
/// ```rust
/// inventory::submit! {
///     ProviderMetadata {
///         name: "stormglass",
///         description: "StormGlass Marine Weather API",
///         api_key_var: "STORMGLASS_API_KEY",
///         instantiate: || {
///             let api_key = StormGlassProvider::get_api_key()?;
///             Ok(Box::new(StormGlassProvider::new(api_key)))
///         },
///     }
/// }
/// ```
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

// Inventory collection declaration - enables automatic metadata gathering
inventory::collect!(ProviderMetadata);

/// Retrieve metadata for a specific provider by name
/// 
/// # Arguments
/// * `name` - Provider identifier (e.g., "stormglass", "openweathermap")
/// 
/// # Returns
/// * `Some(&'static ProviderMetadata)` if provider is registered
/// * `None` if provider name is not found
/// 
/// # Example
/// ```rust
/// if let Some(meta) = get_provider_metadata("stormglass") {
///     println!("Found provider: {}", meta.description);
/// } else {
///     eprintln!("Provider not registered");
/// }
/// ```
pub fn get_provider_metadata(name: &str) -> Option<&'static ProviderMetadata>;

/// Iterator over all registered provider names
/// 
/// # Returns
/// Iterator yielding provider names as `&'static str`
/// 
/// # Example
/// ```rust
/// for name in all_provider_names() {
///     println!("Available: {}", name);
/// }
/// ```
pub fn all_provider_names() -> impl Iterator<Item = &'static str>;

/// Iterator over all providers with (name, description) pairs
/// 
/// Useful for generating help text and documentation
/// 
/// # Returns
/// Iterator yielding `(&'static str, &'static str)` tuples of (name, description)
/// 
/// # Example
/// ```rust
/// println!("Available providers:");
/// for (name, desc) in all_provider_descriptions() {
///     println!("  {}: {}", name, desc);
/// }
/// ```
pub fn all_provider_descriptions() -> impl Iterator<Item = (&'static str, &'static str)>;

/// Create a provider instance by name
/// 
/// Looks up provider metadata and calls its `instantiate` function.
/// 
/// # Arguments
/// * `name` - Provider identifier (e.g., "stormglass", "openweathermap")
/// 
/// # Returns
/// * `Ok(Box<dyn ForecastProvider>)` on success
/// * `Err(anyhow::Error)` if provider not found or instantiation fails
/// 
/// # Errors
/// - Provider name not registered in the registry
/// - Provider's `instantiate()` function returned an error (e.g., missing API key)
/// 
/// # Example
/// ```rust
/// let provider = create_provider("stormglass")?;
/// let data = provider.fetch_weather_data(start, end, lat, lng).await?;
/// ```
pub fn create_provider(name: &str) -> Result<Box<dyn ForecastProvider>>;

/// Validate that a provider name exists in the registry
/// 
/// Used during CLI argument parsing to provide early validation and
/// helpful error messages listing available providers.
/// 
/// # Arguments
/// * `name` - Provider identifier to validate
/// 
/// # Returns
/// * `Ok(())` if provider is registered
/// * `Err(anyhow::Error)` with message listing available providers if not found
/// 
/// # Errors
/// Returns error with format:
/// "Unknown provider '{name}'. Available providers: provider1, provider2, ..."
/// 
/// # Example
/// ```rust
/// validate_provider_name(&args.provider)
///     .context("Provider validation failed")?;
/// ```
pub fn validate_provider_name(name: &str) -> Result<()>;

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
/// 
/// # Example
/// ```rust
/// fn main() {
///     check_duplicates(); // Panics if duplicates found
///     // ... rest of application logic
/// }
/// ```
pub fn check_duplicates();

/// Get count of registered providers
/// 
/// Utility function for diagnostics and testing.
/// 
/// # Returns
/// Number of providers currently registered
/// 
/// # Example
/// ```rust
/// println!("Registered providers: {}", provider_count());
/// ```
pub fn provider_count() -> usize;