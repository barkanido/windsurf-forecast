# Project Coding Rules (Non-Obvious Only)

## Provider Registration (Centralized Registry Pattern)

When adding a new weather provider, you ONLY need to add registration in the provider module itself:

1. Implement [`ForecastProvider`](../../src/forecast_provider.rs:1) trait in your provider module
2. Add `inventory::submit!()` call in provider module to register with [`provider_registry`](../../src/provider_registry.rs:1)
3. Declare module in [`src/providers/mod.rs`](../../src/providers/mod.rs:1)

The registry automatically handles provider discovery, instantiation, and validation. No updates needed to [`main.rs`](../../src/main.rs:1) or [`args.rs`](../../src/args.rs:1).

Example registration (add to provider module):
```rust
inventory::submit! {
    ProviderMetadata {
        name: "providername",
        description: "Provider Description",
        api_key_var: "PROVIDER_API_KEY",
        instantiate: || {
            let api_key = ProviderName::get_api_key()?;
            Ok(Box::new(ProviderName::new(api_key)))
        },
    }
}
```

## Unit Conversion Inconsistency Between Providers

- [`StormGlassProvider`](../../src/providers/stormglass.rs:103) multiplies wind speeds by `MS_TO_KNOTS = 1.94384` constant
- [`OpenWeatherMapProvider`](../../src/providers/openweathermap.rs:50) returns wind speeds in m/s WITHOUT any conversion
- This is INTENTIONAL per provider API design - do not "fix" by making them consistent
- Output JSON will have different units depending on which provider is used

## Timezone Serialization Pattern

All [`WeatherDataPoint`](../../src/forecast_provider.rs:19) timestamps use custom serializer [`serialize_time_jerusalem()`](../../src/forecast_provider.rs:7) that:
- Converts UTC to Asia/Jerusalem timezone automatically during JSON serialization
- Formats as "YYYY-MM-DD HH:MM" (not ISO 8601)
- This happens in serde, not in the provider transform logic

## OpenWeatherMap Uses Different dotenv Access

- [`OpenWeatherMapProvider::get_api_key()`](../../src/providers/openweathermap.rs:71) calls `dotenv::var()` directly
- Other code uses `std::env::var()` after `dotenv::dotenv().ok()` in main
- Env var name is `OPEN_WEATHER_MAP_API_KEY` (underscores, not `OPENWEATHERMAP_API_KEY`)

## Hard-coded Coordinates

Location coordinates are hard-coded in [`main.rs:155`](../../src/main.rs:155) - not configurable via CLI:
```rust
let lat = 32.486722;
let lng = 34.888722;
```

## Date Range Business Rule

The constraint `days_ahead + first_day_offset <= 7` in [`args.rs:64`](../../src/args.rs:64) is a business rule for "reliable forecasts", NOT an API limitation. The APIs themselves support longer ranges.