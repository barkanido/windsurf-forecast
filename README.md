# Weather Forecast Application

A Rust binary for retrieving and processing weather forecast data from multiple weather API providers.

This application uses a modular provider architecture, allowing easy integration of different weather APIs. Currently supports Storm Glass API, with the ability to add more providers.

## Features

- Fetches weather forecast data from multiple weather API providers
- Supports configurable forecast periods (1-7 days ahead)
- Converts wind speeds from m/s to knots
- Configurable timezone support with "LOCAL" option and interactive picker
- Timezone settings are automatically persisted to config file
- Validates timezone against location coordinates
- Configurable location coordinates via CLI or config file
- Outputs formatted JSON with metadata and unit descriptions
- Comprehensive error handling with user-friendly messages
- Command-line interface with validation

## Installation

1. Install Rust if you haven't already: https://rustup.rs/

2. Clone this repository and navigate to the project directory

3. Build the project:
```bash
cargo build --release
```

The compiled binary will be available at `target/release/windsurf-forecast` (or `.exe` on Windows).

## Development

### Testing Workflow

When making code changes, follow this iterative workflow:

1. **Check compilation**: `cargo check` - Fix all errors and warnings
2. **Build debug**: `cargo build` - Fast builds for testing (don't use `--release` during development)
3. **Run linter**: `cargo clippy` - Address all warnings (fix immediately or add to TODO)
4. **Test changes**: `cargo run -- --provider stormglass --days-ahead 3`
5. **Final validation**: `cargo run --release` - Only for end-to-end testing before deployment

See [AGENTS.md](AGENTS.md) for detailed testing commands and workflows.

## Configuration

### API Keys

Each provider requires its own API key as an environment variable:
- **Storm Glass**: `STORMGLASS_API_KEY`
- **OpenWeatherMap**: `OPEN_WEATHER_MAP_API_KEY`
- For other providers, see [ADDING_PROVIDERS.md](ADDING_PROVIDERS.md)

Create a `.env` file in the project root:

```bash
STORMGLASS_API_KEY=your-api-key-here
OPEN_WEATHER_MAP_API_KEY=your-api-key-here
```

A `.env.example` template file is provided as a reference.

**Note:** The `.env` file is automatically ignored by git to keep your API key secure.

### Timezone Configuration

The application supports configurable timezones for displaying forecast timestamps.

**Default Behavior:**
- Default timezone is UTC
- A warning is displayed when using the default UTC timezone
- Timezone is saved to `~/.windsurf-config.toml` for future use

**Setting Timezone:**

1. **Command Line Flag**:
   ```bash
   # Use a specific timezone
   cargo run --release -- --timezone America/New_York
   # Or use short form:
   cargo run --release -- -z America/New_York
   # Use system/local timezone
   cargo run --release -- -z LOCAL
   ```
   When you specify a timezone via CLI, it's automatically saved to your config file for future use.

2. **Interactive Picker** (Recommended for first-time setup):
   ```bash
   cargo run --release -- --pick-timezone
   ```
   This launches a searchable, filterable list of all IANA timezones.

3. **Manual Config File**:
   Edit `~/.windsurf-config.toml`:
   ```toml
   [general]
   timezone = "America/New_York"
   default_provider = "stormglass"
   lat = 32.486722
   lng = 34.888722
   ```

4. **Custom Config Location**:
   ```bash
   cargo run --release -- --config /path/to/config.toml
   ```

**Timezone Validation:**
- The app validates that the configured timezone matches the location coordinates
- If a mismatch is detected, a warning is displayed
- Use standard IANA timezone names (e.g., "UTC", "America/New_York", "Asia/Jerusalem", "Europe/London")

## Usage

Run the application with optional command line arguments:

```bash
cargo run --release -- [OPTIONS]
```

Or run the compiled binary directly:

```bash
./target/release/windsurf-forecast [OPTIONS]
```

### Command Line Options

- `--days-ahead <N>`: Number of days to forecast ahead (1-7, default: 4)
- `--first-day-offset <N>`: Number of days to offset the start date (0-7, default: 0 for today)
- `--provider <PROVIDER>`: Weather forecast provider to use (default: "stormglass")
- `--timezone <TIMEZONE>`, `-z <TIMEZONE>`: Timezone for displaying timestamps (use "LOCAL" for system timezone, overrides and persists to config file)
- `--pick-timezone`: Launch interactive timezone picker and save to config
- `--lat <LAT>`: Latitude for the forecast location
- `--lng <LNG>`: Longitude for the forecast location
- `--config <PATH>`: Path to custom config file (default: ~/.windsurf-config.toml)
- `--list-providers`: List all available weather providers and exit
- `--help`: Display help information

**Available Providers:**
- `stormglass` - Storm Glass API (default)
- `openweathermap` - OpenWeatherMap API
- To add more providers, see [ADDING_PROVIDERS.md](ADDING_PROVIDERS.md)

**Important:** The sum of `--days-ahead` and `--first-day-offset` must not exceed 7 to ensure reliable forecasts.

### Examples

```bash
# Get 4-day forecast starting today with default provider (stormglass)
cargo run --release

# Get 3-day forecast starting today
cargo run --release -- --days-ahead 3

# Explicitly specify the provider
cargo run --release -- --provider openweathermap --days-ahead 2

# Use a specific timezone (saves to config file)
cargo run --release -- --timezone America/New_York
# Or use short form
cargo run --release -- -z America/New_York

# Use system/local timezone (saves to config file)
cargo run --release -- -z LOCAL

# Pick timezone interactively (saves to config file)
cargo run --release -- --pick-timezone

# Specify custom coordinates
cargo run --release -- --lat 40.7128 --lng -74.0060

# Get 2-day forecast starting 3 days from now
cargo run --release -- --days-ahead 2 --first-day-offset 3

# Get 5-day forecast starting tomorrow with timezone
cargo run --release -- --days-ahead 5 --first-day-offset 1 --timezone Europe/London

# Use custom config file location
cargo run --release -- --config ./my-config.toml

# Get help and see all options
cargo run --release -- --help
```

### Output

The application generates a JSON file named `weather_data_{N}d_{date}.json` where:
- `{N}` is the number of days in the forecast
- `{date}` is the start date in YYMMDD format

The output includes:
- Hourly weather data for the specified period
- Air and water temperatures in Celsius
- Wind speed and gust in knots (converted from m/s)
- Swell height (meters), period (seconds), and direction (degrees)
- Wind direction (degrees)
- Timestamps in configured timezone (default: UTC)
- Metadata including provider information and unit descriptions

## Weather Parameters

The application fetches the following weather parameters from Storm Glass:

- **Air Temperature**: Air temperature in degrees Celsius
- **Wind Speed**: Speed of wind at 10m above ground in knots
- **Gust**: Wind gust in knots
- **Wind Direction**: Direction of wind at 10m above ground (0° = north)
- **Swell Height**: Height of swell waves in meters
- **Swell Period**: Period of swell waves in seconds
- **Swell Direction**: Direction of swell waves (0° = north)
- **Water Temperature**: Water temperature in degrees Celsius

## Location

Location coordinates must be provided either via command-line arguments or config file:

**Via Command Line:**
```bash
cargo run --release -- --lat 32.486722 --lng 34.888722
```

**Via Config File** (`~/.windsurf-config.toml`):
```toml
[general]
lat = 32.486722
lng = 34.888722
```

Coordinates are validated against the configured timezone to detect potential mismatches.

## Architecture

The application uses a trait-based provider architecture:
- **ForecastProvider trait**: Common interface for all weather providers
- **Modular providers**: Each provider implements the trait independently
- **Easy extensibility**: Add new providers without modifying core logic

See [ADDING_PROVIDERS.md](ADDING_PROVIDERS.md) for detailed instructions on adding new weather API providers.

## Dependencies

- `reqwest` - HTTP client for API requests
- `tokio` - Async runtime
- `serde` & `serde_json` - JSON serialization/deserialization
- `chrono` & `chrono-tz` - Date/time handling and timezone conversion
- `iana-time-zone` - System timezone detection for "LOCAL" option
- `clap` - Command-line argument parsing
- `dotenv` - Environment variable loading from .env file
- `anyhow` & `thiserror` - Error handling
- `async-trait` - Async trait support for providers

## Error Handling

The application provides detailed error messages for common issues:

- **402 Payment Required**: Daily request limit exceeded
- **403 Forbidden**: Invalid or missing API key
- **404 Not Found**: Invalid API endpoint
- **422 Unprocessable Content**: Invalid request parameters
- **503 Service Unavailable**: Storm Glass API temporarily unavailable
- Network errors, configuration errors, and unexpected errors

## Provider Documentation

### Storm Glass API
For more information about the Storm Glass API, see: https://docs.stormglass.io/#/weather

### Adding New Providers
To integrate additional weather API providers, see [ADDING_PROVIDERS.md](ADDING_PROVIDERS.md) for a complete guide.

## License
