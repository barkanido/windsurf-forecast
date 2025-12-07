// API Contract: Provider Module Modifications
// 
// This file documents the changes required in existing provider modules
// to integrate with the centralized registry system.

use anyhow::Result;
use crate::forecast_provider::ForecastProvider;
use crate::provider_registry::ProviderMetadata;

// ============================================================================
// PATTERN 1: StormGlass Provider Registration
// ============================================================================

/// Example registration for StormGlass provider
/// 
/// Location: src/providers/stormglass.rs
/// 
/// Add this registration block at the end of the provider module.
/// The provider struct, trait implementation, and API key retrieval
/// methods remain unchanged.
/// 
/// # Changes Required
/// - Add `use crate::provider_registry::ProviderMetadata;` import
/// - Add the `inventory::submit!()` block shown below
/// 
/// # No Changes Required
/// - StormGlassProvider struct definition
/// - ForecastProvider trait implementation
/// - get_api_key() method
/// - Internal provider logic
#[allow(dead_code)]
fn stormglass_registration_example() {
    inventory::submit! {
        ProviderMetadata {
            name: "stormglass",
            description: "StormGlass Marine Weather API",
            api_key_var: "STORMGLASS_API_KEY",
            instantiate: || {
                let api_key = StormGlassProvider::get_api_key()?;
                Ok(Box::new(StormGlassProvider::new(api_key)))
            },
        }
    }
}

// ============================================================================
// PATTERN 2: OpenWeatherMap Provider Registration
// ============================================================================

/// Example registration for OpenWeatherMap provider
/// 
/// Location: src/providers/openweathermap.rs
/// 
/// This provider uses `dotenv::var()` directly (different from StormGlass),
/// which is preserved in the instantiate closure.
/// 
/// # Changes Required
/// - Add `use crate::provider_registry::ProviderMetadata;` import
/// - Add the `inventory::submit!()` block shown below
/// 
/// # No Changes Required
/// - OpenWeatherMapProvider struct definition
/// - ForecastProvider trait implementation
/// - get_api_key() method (continues using dotenv::var() directly)
/// - Internal provider logic
#[allow(dead_code)]
fn openweathermap_registration_example() {
    inventory::submit! {
        ProviderMetadata {
            name: "openweathermap",
            description: "OpenWeatherMap Global Weather Data",
            api_key_var: "OPEN_WEATHER_MAP_API_KEY",
            instantiate: || {
                let api_key = OpenWeatherMapProvider::get_api_key()?;
                Ok(Box::new(OpenWeatherMapProvider::new(api_key)))
            },
        }
    }
}

// ============================================================================
// PATTERN 3: Future Provider Template
// ============================================================================

/// Template for adding a new weather provider
/// 
/// Location: src/providers/{provider_name}.rs
/// 
/// This template shows the complete pattern for adding a new provider.
/// Follow steps 1-3 to add a provider without modifying any central code.
/// 
/// # Step 1: Implement ForecastProvider Trait (EXISTING PATTERN)
/// ```rust
/// use crate::forecast_provider::ForecastProvider;
/// use anyhow::{Result, anyhow};
/// use async_trait::async_trait;
/// 
/// pub struct NewProvider {
///     api_key: String,
/// }
/// 
/// impl NewProvider {
///     pub fn new(api_key: String) -> Self {
///         Self { api_key }
///     }
///     
///     pub fn get_api_key() -> Result<String> {
///         std::env::var("NEW_PROVIDER_API_KEY")
///             .map_err(|_| anyhow!("NEW_PROVIDER_API_KEY not set in .env"))
///     }
/// }
/// 
/// #[async_trait]
/// impl ForecastProvider for NewProvider {
///     fn name(&self) -> &str {
///         "newprovider"
///     }
///     
///     async fn fetch_weather_data(
///         &self,
///         start: DateTime<Utc>,
///         end: DateTime<Utc>,
///         lat: f64,
///         lng: f64,
///     ) -> Result<Vec<WeatherDataPoint>> {
///         // Provider-specific implementation
///         todo!()
///     }
/// }
/// ```
/// 
/// # Step 2: Add Registry Import (NEW)
/// ```rust
/// use crate::provider_registry::ProviderMetadata;
/// ```
/// 
/// # Step 3: Register Provider (NEW - ONLY NEW STEP REQUIRED)
/// ```rust
/// inventory::submit! {
///     ProviderMetadata {
///         name: "newprovider",
///         description: "New Provider Weather Service",
///         api_key_var: "NEW_PROVIDER_API_KEY",
///         instantiate: || {
///             let api_key = NewProvider::get_api_key()?;
///             Ok(Box::new(NewProvider::new(api_key)))
///         },
///     }
/// }
/// ```
/// 
/// # Step 4: Declare Module (EXISTING PATTERN)
/// In src/providers/mod.rs:
/// ```rust
/// pub mod newprovider;
/// ```
/// 
/// # Result
/// Provider automatically available via:
/// - CLI: `cargo run -- --provider newprovider`
/// - Help: Listed in `cargo run -- --help` output
/// - Validation: Accepted by CLI argument parser
/// 
/// # No Central Code Changes Required
/// - ❌ No update to src/main.rs
/// - ❌ No update to src/args.rs
/// - ❌ No update to provider registry
/// - ❌ No update to any central registration code
#[allow(dead_code)]
fn future_provider_template() {}

// ============================================================================
// ANTI-PATTERNS TO AVOID
// ============================================================================

/// Common mistakes when registering providers
/// 
/// # Mistake 1: Provider name mismatch
/// BAD: name in metadata doesn't match provider's self-reported name
/// ```rust
/// inventory::submit! {
///     ProviderMetadata {
///         name: "stormglass",  // Metadata says "stormglass"
///         instantiate: || {
///             Ok(Box::new(StormGlassProvider::new(api_key)))
///             // But provider.name() returns "StormGlass" (wrong case)
///         },
///     }
/// }
/// ```
/// GOOD: Ensure consistency between metadata name and provider.name()
/// 
/// # Mistake 2: Duplicate provider names
/// BAD: Two providers with same name
/// ```rust
/// // In stormglass.rs
/// inventory::submit! { ProviderMetadata { name: "stormglass", ... } }
/// 
/// // In stormglass_v2.rs
/// inventory::submit! { ProviderMetadata { name: "stormglass", ... } }
/// // Runtime panic: duplicate detected
/// ```
/// GOOD: Use unique names (e.g., "stormglass" and "stormglass-v2")
/// 
/// # Mistake 3: Forgetting to handle errors in instantiate
/// BAD: Unwrapping in instantiate closure
/// ```rust
/// instantiate: || {
///     let api_key = std::env::var("API_KEY").unwrap(); // Panics!
///     Ok(Box::new(Provider::new(api_key)))
/// }
/// ```
/// GOOD: Propagate errors with ?
/// ```rust
/// instantiate: || {
///     let api_key = Provider::get_api_key()?;
///     Ok(Box::new(Provider::new(api_key)))
/// }
/// ```
/// 
/// # Mistake 4: Complex logic in instantiate
/// BAD: Doing too much in the closure
/// ```rust
/// instantiate: || {
///     // Complex initialization logic inline
///     let config = load_config()?;
///     let client = build_http_client()?;
///     let validator = create_validator()?;
///     Ok(Box::new(Provider::new(config, client, validator)))
/// }
/// ```
/// GOOD: Encapsulate in provider's constructor or static method
/// ```rust
/// instantiate: || {
///     Provider::create_from_env() // All logic in provider
/// }
/// 
/// impl Provider {
///     pub fn create_from_env() -> Result<Box<dyn ForecastProvider>> {
///         // Complex initialization logic here
///     }
/// }
/// ```
#[allow(dead_code)]
fn anti_patterns() {}