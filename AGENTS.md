# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Build & Run Commands

### Development Testing Workflow (Constitution VI)

Follow this sequence for all code changes:

```bash
# 1. Check compilation (fix errors and warnings)
cargo check

# 2. Build for testing (DO NOT use --release during development)
cargo build

# 3. Run clippy (address all warnings - fix now or add to TODO)
cargo clippy

# 4. Test the application (debug build for fast iteration)
cargo run -- --provider stormglass --days-ahead 3
cargo run -- --provider openweathermap --days-ahead 2
```

### Production Testing (Final Validation Only)

```bash
# Use --release ONLY for final end-to-end testing
cargo build --release
cargo run --release -- --provider stormglass
cargo run --release -- --provider openweathermap --days-ahead 3
```

### Quick Reference

```bash
# Most common development commands
cargo check          # Fast compile check
cargo build          # Debug build (fast)
cargo clippy         # Linting
cargo run            # Run debug build

# Production commands (slow, use sparingly)
cargo build --release
cargo run --release
```

## Critical Non-Obvious Patterns

### Wind Speed Unit Conversion (StormGlass Only)
- [`StormGlassProvider`](src/providers/stormglass.rs:83) converts wind speeds from m/s to knots using `MS_TO_KNOTS = 1.94384`
- [`OpenWeatherMapProvider`](src/providers/openweathermap.rs:34) returns wind speeds in m/s WITHOUT conversion
- This inconsistency means output units differ between providers

### Timezone Transformation
- All timestamps are automatically converted from UTC to Asia/Jerusalem timezone in serialization
- See custom serializer [`serialize_time_jerusalem()`](src/forecast_provider.rs:7) in WeatherDataPoint
- Format: "YYYY-MM-DD HH:MM" in Jerusalem time

### Provider Registration (Centralized Registry)
When adding a new provider, you ONLY need to add registration in the provider module itself:
1. Implement [`ForecastProvider`](src/forecast_provider.rs:1) trait in your provider module
2. Add `inventory::submit!()` call in provider module to register with [`provider_registry`](src/provider_registry.rs:1)
3. Declare module in [`src/providers/mod.rs`](src/providers/mod.rs:1)

The registry automatically handles provider discovery, instantiation, and validation. No updates needed to [`main.rs`](src/main.rs:1) or [`args.rs`](src/args.rs:1).

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

### Hard-coded Location
- Coordinates are hard-coded in [`main.rs`](src/main.rs:155): `lat = 32.486722, lng = 34.888722`
- Location: 32°29'12.2"N 34°53'19.4"E (not configurable via CLI)

### Date Range Constraint
- `days_ahead + first_day_offset` must NOT exceed 7 (enforced in [`args.rs`](src/args.rs:64))
- This is a business rule for "reliable forecasts", not an API limitation

### OpenWeatherMap Provider Uses `dotenv::var()` Directly
- [`OpenWeatherMapProvider::get_api_key()`](src/providers/openweathermap.rs:71) calls `dotenv::var()` instead of `std::env::var()`
- Env var name: `OPEN_WEATHER_MAP_API_KEY` (note underscores, not `OPENWEATHERMAP_API_KEY`)

## Code Style

- Use `anyhow::Result` for error handling (not `std::result::Result`)
- Provider-specific errors use `thiserror` (see [`StormGlassAPIError`](src/providers/stormglass.rs:15))
- Field naming: use snake_case in structs, camelCase in JSON via `#[serde(rename = "camelCase")]`