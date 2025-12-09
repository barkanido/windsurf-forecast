# windsurf-forecast Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-07

## Active Technologies
- Filesystem (config file: ~/.windsurf-config.toml, .env for API keys, JSON output) (003-unit-tests)
- Rust 1.75+ (edition 2021) + clap 4.0 (CLI parsing), serde 1.0 + toml 0.8 (serialization), chrono 0.4 + chrono-tz 0.8 (timezones), anyhow 1.0 (error handling) (004-config-refactor)
- File-based configuration (~/.windsurf-config.toml), environment variables (.env for API keys) (004-config-refactor)

- Rust 1.75+ (edition 2021) + chrono 0.4, chrono-tz 0.8, serde 1.0, clap 4.4, anyhow 1.0, async-trait 0.1 (002-timezone-refactor)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test; cargo clippy

## Code Style

Rust 1.75+ (edition 2021): Follow standard conventions

## Recent Changes
- 004-config-refactor: Added Rust 1.75+ (edition 2021) + clap 4.0 (CLI parsing), serde 1.0 + toml 0.8 (serialization), chrono 0.4 + chrono-tz 0.8 (timezones), anyhow 1.0 (error handling)
- 003-unit-tests: Added Rust 1.75+ (edition 2021)

- 002-timezone-refactor: Added Rust 1.75+ (edition 2021) + chrono 0.4, chrono-tz 0.8, serde 1.0, clap 4.4, anyhow 1.0, async-trait 0.1

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
