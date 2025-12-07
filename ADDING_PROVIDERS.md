# Adding New Weather Forecast Providers

This guide explains how to add a new weather forecast provider to the application using the centralized provider registry.

## Architecture Overview

The application uses a trait-based architecture with automatic provider discovery:
- Each weather provider implements the [`ForecastProvider`](src/forecast_provider.rs:1) trait
- Providers self-register using the `inventory` crate pattern
- The [`provider_registry`](src/provider_registry.rs:1) automatically discovers and manages all providers

## Quick Start: 3 Simple Steps

### 1. Create Provider Module

Create a new file in `src/providers/` for your provider (e.g., `src/providers/weatherapi.rs`).

### 2. Implement the Provider Trait

Implement the `ForecastProvider` trait for your provider:

```rust
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::env;

use crate::forecast_provider::{ForecastProvider, WeatherDataPoint};
use crate::provider_registry::ProviderMetadata;

pub struct WeatherAPIProvider {
    api_key: String,
}

impl WeatherAPIProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn get_api_key() -> Result<String> {
        env::var("WEATHERAPI_API_KEY").context(
            "WEATHERAPI_API_KEY not found. Please set it in your .env file."
        )
    }
}

#[async_trait]
impl ForecastProvider for WeatherAPIProvider {
    fn name(&self) -> &str {
        "weatherapi"
    }

    fn get_api_key() -> Result<String> {
        Self::get_api_key()
    }

    async fn fetch_weather_data(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        lat: f64,
        lng: f64,
    ) -> Result<Vec<WeatherDataPoint>> {
        // Implement API call and transformation logic here
        // Return Vec<WeatherDataPoint>
        todo!("Implement WeatherAPI integration")
    }
}

// Register provider with central registry
inventory::submit! {
    ProviderMetadata {
        name: "weatherapi",
        description: "WeatherAPI.com Global Weather Data",
        api_key_var: "WEATHERAPI_API_KEY",
        instantiate: || {
            let api_key = WeatherAPIProvider::get_api_key()?;
            Ok(Box::new(WeatherAPIProvider::new(api_key)))
        },
    }
}
```

### 3. Declare Module

Add your provider to [`src/providers/mod.rs`](src/providers/mod.rs:1):

```rust
pub mod stormglass;
pub mod openweathermap;
pub mod weatherapi;  // Add this line
```

**That's it!** Your provider is now automatically:
- ✅ Discovered by the registry
- ✅ Available via CLI: `cargo run -- --provider weatherapi`
- ✅ Listed in `--help` output
- ✅ Validated during argument parsing

## No Central Code Changes Required

Unlike the old approach, you **DO NOT** need to update:
- ❌ `src/main.rs` - No provider imports or instantiation code
- ❌ `src/args.rs` - No validation updates
- ❌ Any central registration code

The registry handles everything automatically.

## Common Data Structure

All providers must transform their API-specific data into the common [`WeatherDataPoint`](src/forecast_provider.rs:19) structure:

```rust
pub struct WeatherDataPoint {
    pub time: DateTime<Utc>,
    pub air_temperature: Option<f64>,
    pub wind_speed: Option<f64>,
    pub wind_direction: Option<f64>,
    pub gust: Option<f64>,
    pub swell_height: Option<f64>,
    pub swell_period: Option<f64>,
    pub swell_direction: Option<f64>,
    pub water_temperature: Option<f64>,
}
```

## Testing Your Provider

Test your provider with:

```bash
# Development testing
cargo check
cargo build
cargo clippy
cargo run -- --provider weatherapi --days-ahead 2

# Verify it appears in help
cargo run -- --help

# Test error handling
cargo run -- --provider weatherapi  # Without API key
```

## Provider Registry Details

### ProviderMetadata Fields

- **name**: Unique identifier used in CLI (e.g., "weatherapi")
  - Must be lowercase alphanumeric with optional hyphens/underscores
  - Must be unique across all providers
  
- **description**: Human-readable description shown in `--help` output
  - Keep concise (< 100 chars recommended)
  
- **api_key_var**: Environment variable name for the API key
  - Follow SCREAMING_SNAKE_CASE convention (e.g., "WEATHERAPI_API_KEY")
  
- **instantiate**: Factory function that creates provider instances
  - Responsible for retrieving its own API key
  - Returns `Result<Box<dyn ForecastProvider>>`
  - Called on-demand when provider is selected

### API Key Retrieval

All providers should use `std::env::var()` for consistent environment variable access:
- The `.env` file is loaded once at application startup in [`main.rs`](src/main.rs:1)
- After that, all providers use `std::env::var()` to read environment variables
- This ensures consistent behavior across all providers

## Best Practices

1. **Error Handling**: Use descriptive error messages specific to your provider
2. **API Key Management**: Each provider should have its own unique environment variable
3. **Unit Conversions**: Handle any necessary unit conversions in the provider
4. **Documentation**: Add comments explaining provider-specific logic
5. **Testing**: Test edge cases and error conditions
6. **Naming Consistency**: Ensure `ProviderMetadata.name` matches the provider's `name()` method

## Removing a Provider

To remove a provider:
1. Delete the provider file (e.g., `src/providers/weatherapi.rs`)
2. Remove the module declaration from `src/providers/mod.rs`

The registry will automatically update - no other changes needed.

## Renaming a Provider

To rename a provider, simply change the `name` field in the `inventory::submit!()` block:

```rust
inventory::submit! {
    ProviderMetadata {
        name: "weatherapi-v2",  // Changed from "weatherapi"
        // ... rest unchanged
    }
}
```

The new name automatically propagates to CLI validation, help text, and error messages.

## Examples

### Complete Example: StormGlass Provider
See [`src/providers/stormglass.rs`](src/providers/stormglass.rs:1) for a complete reference implementation.

### Complete Example: OpenWeatherMap Provider
See [`src/providers/openweathermap.rs`](src/providers/openweathermap.rs:1) for another complete implementation example.

## Registry API Reference

For advanced use cases, the registry provides:

```rust
// Query functions
provider_registry::get_provider_metadata(name: &str) -> Option<&'static ProviderMetadata>
provider_registry::all_provider_names() -> impl Iterator<Item = &'static str>
provider_registry::all_provider_descriptions() -> impl Iterator<Item = (&'static str, &'static str)>

// Instantiation
provider_registry::create_provider(name: &str) -> Result<Box<dyn ForecastProvider>>

// Validation
provider_registry::validate_provider_name(name: &str) -> Result<()>
provider_registry::check_duplicates()  // Called automatically at startup
```

## Troubleshooting

### "Unknown provider" error
- Verify the `name` field in `inventory::submit!()` matches the CLI argument
- Check that the module is declared in `src/providers/mod.rs`
- Run `cargo clean && cargo build` to ensure registry is rebuilt

### "Duplicate provider name" panic
- Ensure each provider has a unique `name` field
- Check for accidentally duplicated `inventory::submit!()` blocks

### Provider not appearing in help
- Verify `cargo build` completed successfully
- Check that the `inventory::submit!()` block is present
- Ensure the module is public (`pub mod`) in `src/providers/mod.rs`
