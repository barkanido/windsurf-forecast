# Weather Forecast Application

A Rust binary for retrieving and processing weather forecast data from multiple weather API providers.

This application uses a modular provider architecture, allowing easy integration of different weather APIs. Currently supports Storm Glass API, with the ability to add more providers.

## Features

- Fetches weather forecast data from Storm Glass API
- Supports configurable forecast periods (1-7 days ahead)
- Converts wind speeds from m/s to knots
- Converts timestamps from UTC to Asia/Jerusalem timezone
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

The compiled binary will be available at `target/release/stromglass-windsurf-forecast` (or `.exe` on Windows).

## Configuration

The application requires an API key for your chosen weather provider. By default, it uses Storm Glass API.

### API Keys

Each provider requires its own API key as an environment variable:
- **Storm Glass**: `STORMGLASS_API_KEY`
- For other providers, see [ADDING_PROVIDERS.md](ADDING_PROVIDERS.md)

### Option 1: Using .env file (Recommended)

Create a `.env` file in the project root:

```bash
STORMGLASS_API_KEY=your-api-key-here
```

A `.env.example` template file is provided as a reference.

### Option 2: Set environment variable directly

Set the environment variable in your shell:

```bash
# Windows (Command Prompt)
set STORMGLASS_API_KEY=your-api-key-here

# Windows (PowerShell)
$env:STORMGLASS_API_KEY="your-api-key-here"

# Linux/macOS
export STORMGLASS_API_KEY=your-api-key-here
```

**Note:** The `.env` file is automatically ignored by git to keep your API key secure.

## Usage

Run the application with optional command line arguments:

```bash
cargo run --release -- [OPTIONS]
```

Or run the compiled binary directly:

```bash
./target/release/stromglass-windsurf-forecast [OPTIONS]
```

### Command Line Options

- `--days-ahead <N>`: Number of days to forecast ahead (1-7, default: 4)
- `--first-day-offset <N>`: Number of days to offset the start date (0-7, default: 0 for today)
- `--provider <PROVIDER>`: Weather forecast provider to use (default: "stormglass")
- `--help`: Display help information

**Available Providers:**
- `stormglass` - Storm Glass API (default)
- To add more providers, see [ADDING_PROVIDERS.md](ADDING_PROVIDERS.md)

**Important:** The sum of `--days-ahead` and `--first-day-offset` must not exceed 7 to ensure reliable forecasts.

### Examples

```bash
# Get 4-day forecast starting today with default provider (stormglass)
cargo run --release

# Get 3-day forecast starting today
cargo run --release -- --days-ahead 3

# Explicitly specify the provider
cargo run --release -- --provider stormglass --days-ahead 2

# Get 2-day forecast starting 3 days from now
cargo run --release -- --days-ahead 2 --first-day-offset 3

# Get 5-day forecast starting tomorrow
cargo run --release -- --days-ahead 5 --first-day-offset 1

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
- Timestamps in Asia/Jerusalem timezone
- Metadata including API quota usage and unit descriptions

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

The application is configured for coordinates:
- Latitude: 32.486722°N
- Longitude: 34.888722°E
- (32°29'12.2"N 34°53'19.4"E)

To change the location, modify the `lat` and `lng` variables in `src/main.rs`.

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
