# Adding New Weather Forecast Providers

This guide explains how to add a new weather forecast provider to the application.

## Architecture Overview

The application uses a trait-based architecture where each weather provider implements the `ForecastProvider` trait defined in `src/forecast_provider.rs`.

## Steps to Add a New Provider

### 1. Create Provider Module

Create a new file in `src/providers/` for your provider (e.g., `src/providers/openweather.rs`).

### 2. Implement the Provider

Your provider must implement the `ForecastProvider` trait:

```rust
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::env;

use crate::forecast_provider::{ForecastProvider, WeatherDataPoint};

pub struct OpenWeatherProvider {
    api_key: String,
}

impl OpenWeatherProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl ForecastProvider for OpenWeatherProvider {
    fn name(&self) -> &str {
        "openweather"
    }

    fn get_api_key() -> Result<String> {
        env::var("OPENWEATHER_API_KEY").context(
            "OPENWEATHER_API_KEY not found. Please set it in your .env file."
        )
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
        todo!("Implement OpenWeather API integration")
    }
}
```

### 3. Register Provider in Module

Add your provider to `src/providers/mod.rs`:

```rust
pub mod stormglass;
pub mod openweather;  // Add this line
```

### 4. Update Main to Support New Provider

In `src/main.rs`, update the `validate_provider` function:

```rust
fn validate_provider(provider_name: &str) -> Result<()> {
    match provider_name {
        "stormglass" => Ok(()),
        "openweather" => Ok(()),  // Add this line
        _ => anyhow::bail!(
            "Unknown provider '{}'. Available providers: stormglass, openweather",
            provider_name
        ),
    }
}
```

Update the `create_provider` function:

```rust
fn create_provider(provider_name: &str, api_key: String) -> Result<Box<dyn ForecastProvider>> {
    match provider_name {
        "stormglass" => Ok(Box::new(StormGlassProvider::new(api_key))),
        "openweather" => Ok(Box::new(OpenWeatherProvider::new(api_key))),  // Add this line
        _ => unreachable!("Provider validation should have caught this"),
    }
}
```

Update the API key retrieval in `run()`:

```rust
let api_key = match args.provider.as_str() {
    "stormglass" => StormGlassProvider::get_api_key()?,
    "openweather" => OpenWeatherProvider::get_api_key()?,  // Add this line
    _ => unreachable!("Provider validation should have caught this"),
};
```

### 5. Add Imports

Add the import for your provider at the top of `src/main.rs`:

```rust
use providers::openweather::OpenWeatherProvider;
```

## Common Data Structure

All providers must transform their API-specific data into the common `WeatherDataPoint` structure:

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
cargo run -- --provider openweather --days-ahead 2
```

## Best Practices

1. **Error Handling**: Use descriptive error messages specific to your provider
2. **API Key Management**: Each provider should have its own environment variable
3. **Unit Conversions**: Handle any necessary unit conversions in the provider
4. **Documentation**: Add comments explaining provider-specific logic
5. **Testing**: Test edge cases and error conditions

## Example: StormGlass Provider

See `src/providers/stormglass.rs` for a complete reference implementation.
