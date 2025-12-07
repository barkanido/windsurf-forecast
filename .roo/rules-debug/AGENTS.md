# Project Debug Rules (Non-Obvious Only)

## No Tests Present

This project has no test suite - there are no `#[test]` or `#[cfg(test)]` blocks in the codebase.

## Provider Error Handling Differs

- [`StormGlassProvider`](../../src/providers/stormglass.rs:15) uses custom `StormGlassAPIError` with detailed HTTP status code messages (402, 403, 404, 422, 503)
- [`OpenWeatherMapProvider`](../../src/providers/openweathermap.rs:104) uses generic `anyhow::anyhow!` errors
- StormGlass errors are more helpful for debugging API issues

## API Key Loading Inconsistency

- Main code calls `dotenv::dotenv().ok()` in [`main.rs:121`](../../src/main.rs:121) before accessing env vars
- [`OpenWeatherMapProvider::get_api_key()`](../../src/providers/openweathermap.rs:71) calls `dotenv::var()` directly (not `std::env::var()`)
- If `.env` file is malformed, OpenWeatherMap provider may fail differently than StormGlass

## Date Calculation Edge Case

Date range calculation in [`main.rs:142-152`](../../src/main.rs:142) uses `.date_naive().and_hms_opt()` pattern that could silently fail if `and_hms_opt()` returns `None`, but code unwraps without context.

## Output Files Silently Overwrite

[`write_weather_json()`](../../src/main.rs:93) in main.rs will silently overwrite existing files with same name pattern `weather_data_{}d_{}.json`.