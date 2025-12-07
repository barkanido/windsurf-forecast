# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Build & Run Commands

```bash
# Build release binary
cargo build --release

# Run with default provider (stormglass)
cargo run --release

# Run with specific provider
cargo run --release -- --provider openweathermap --days-ahead 3
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

### Provider Registration (3 Required Locations)
When adding a new provider, you MUST update these 3 functions in [`main.rs`](src/main.rs:1):
1. [`create_provider()`](src/main.rs:39) - instantiate provider
2. API key retrieval in [`run()`](src/main.rs:130) - get env var
3. [`validate_provider()`](src/args.rs:77) - register provider name

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