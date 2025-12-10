# Weather Forecast Application

A Rust command-line application for retrieving and processing weather forecast data from multiple weather API providers.

This application uses a modular provider architecture, allowing easy integration of different weather APIs. Currently supports StormGlass, OpenWeatherMap, and Windy.com providers.

## Features

- **Multiple Providers**: StormGlass, OpenWeatherMap, and Windy.com weather APIs
- **Flexible Forecasting**: Configure forecast periods (1-7 days ahead) with offset support
- **Unit Conversion**: Automatic wind speed conversion (provider-dependent)
- **Timezone Support**: Configurable timezones with "LOCAL" option and interactive picker
- **Persistent Configuration**: Automatic saving of timezone and location settings
- **Location Validation**: Validates timezone against location coordinates
- **JSON Output**: Formatted JSON with comprehensive metadata and unit descriptions
- **Robust Error Handling**: User-friendly error messages for common API issues
- **Extensible Architecture**: Easy addition of new weather providers

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

Each provider requires its own API key configured as an environment variable:

| Provider | Environment Variable | Sign Up Link |
|----------|---------------------|--------------|
| **StormGlass** | `STORMGLASS_API_KEY` | https://stormglass.io/ |
| **OpenWeatherMap** | `OPEN_WEATHER_MAP_API_KEY` | https://openweathermap.org/api |
| **Windy.com** | `WINDY_API_KEY` | https://api.windy.com/ |

**Setup:**

1. Create a `.env` file in the project root (use `.env.example` as template):
   ```bash
   STORMGLASS_API_KEY=your-stormglass-key-here
   OPEN_WEATHER_MAP_API_KEY=your-openweathermap-key-here
   WINDY_API_KEY=your-windy-key-here
   ```

2. (Optional) Use a custom `.env` file location:
   ```bash
   cargo run --release -- --env-file /path/to/custom.env
   ```

**Security Note:** The `.env` file is automatically ignored by git to keep your API keys secure.

### Timezone Configuration

The application supports configurable timezones for displaying forecast timestamps.

**Default Behavior:**
- Default timezone is UTC
- A warning is displayed when using the default UTC timezone
- Timezone is saved to `~/.windsurf-config.toml` for future use

**Setting Timezone:**

1. **Command Line Flag** (automatically persists to config file):
   ```bash
   # Use a specific timezone
   cargo run --release -- --timezone America/New_York
   # Or use short form:
   cargo run --release -- -z America/New_York
   # Use system/local timezone (uppercase `LOCAL`)
   cargo run --release -- -z LOCAL
   ```

2. **Interactive Picker** (automatically persists to config file):
   ```bash
   cargo run --release -- --pick-timezone
   ```
   This launches a searchable, filterable list of all IANA timezones. Cannot be used together with `--timezone`.

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
   cargo run --release -- --config-file-path /path/to/config.toml
   ```

**Configuration Precedence:** CLI arguments override config file values, which override defaults.

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

#### Core Forecast Options
| Flag | Description | Default | Range |
|------|-------------|---------|-------|
| `--days-ahead <N>` | Number of days to forecast ahead | 4 | 1-7 |
| `--first-day-offset <N>` | Days to offset start date (0=today) | 0 | 0-7 |
| `--provider <PROVIDER>` | Weather forecast provider | "stormglass" | See below |

#### Timezone Options
| Flag | Description |
|------|-------------|
| `--timezone <TZ>`, `-z <TZ>` | Timezone for timestamps (e.g., "America/New_York", "LOCAL") |
| `--pick-timezone` | Launch interactive timezone picker (conflicts with `--timezone`) |

#### Location Options
| Flag | Description | Range |
|------|-------------|-------|
| `--lat <LAT>` | Latitude for forecast location (required if not in config) | -90.0 to 90.0 |
| `--lng <LNG>` | Longitude for forecast location (required if not in config) | -180.0 to 180.0 |

#### Configuration Options
| Flag | Description |
|------|-------------|
| `--config-file-path <PATH>` | Custom config file path (default: `~/.windsurf-config.toml`) |
| `--save` | Save configuration after successful execution |
| `--env-file <PATH>` | Custom `.env` file path (default: `.env` in current directory) |

#### Information Options
| Flag | Description |
|------|-------------|
| `--list-providers` | List all available weather providers and exit |
| `--help` | Display help information |

**Available Providers:**
| Provider | Name | Description |
|----------|------|-------------|
| `stormglass` | StormGlass | Marine weather data (default) |
| `openweathermap` | OpenWeatherMap | Global weather data |
| `windy` | Windy.com | High-resolution weather models |

To add more providers, see [`ADDING_PROVIDERS.md`](ADDING_PROVIDERS.md).

**Important Constraints:**
- The sum of `--days-ahead` and `--first-day-offset` must not exceed 7 to ensure reliable forecasts
- Latitude must be between -90.0 and 90.0
- Longitude must be between -180.0 and 180.0
- `--pick-timezone` cannot be used together with `--timezone` flag
- Coordinates must be provided either via CLI (`--lat`/`--lng`) or in config file

### Examples

#### Basic Usage

```bash
# Get 4-day forecast starting today with default provider (stormglass)
# Note: Requires coordinates in config file or via --lat/--lng
cargo run --release

# Get 3-day forecast starting today
cargo run --release -- --days-ahead 3

# Explicitly specify the provider
cargo run --release -- --provider openweathermap --days-ahead 2
```

#### Timezone Configuration

```bash
# Use a specific timezone (automatically saves to config file)
cargo run --release -- --timezone America/New_York
# Or use short form
cargo run --release -- -z America/New_York

# Use system/local timezone (automatically saves to config file)
cargo run --release -- -z LOCAL

# Pick timezone interactively (automatically saves to config file)
cargo run --release -- --pick-timezone
```

#### Location Configuration

```bash
# Specify custom coordinates (required on first run if not in config)
cargo run --release -- --lat 40.7128 --lng -74.0060

# Specify coordinates and save them to config file for future use
cargo run --release -- --lat 40.7128 --lng -74.0060 --save
```

#### Advanced Usage

```bash
# Get 2-day forecast starting 3 days from now
cargo run --release -- --days-ahead 2 --first-day-offset 3

# Get 5-day forecast starting tomorrow with timezone
cargo run --release -- --days-ahead 5 --first-day-offset 1 --timezone Europe/London

# Use custom config file location
cargo run --release -- --config-file-path ./my-config.toml

# Save current configuration (provider, timezone, coordinates) to file
cargo run --release -- --provider openweathermap --timezone America/New_York --save
```

#### Information Commands

```bash
# List all available providers
cargo run --release -- --list-providers

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

The application fetches comprehensive weather data from multiple providers. Available parameters vary by provider:

### Common Parameters (All Providers)
- **Air Temperature**: Temperature in degrees Celsius
- **Wind Speed**: Wind speed in m/s or knots (provider-dependent)
- **Wind Gust**: Wind gust speed
- **Wind Direction**: Direction in degrees (0° = north)
- **Swell Height**: Height of swell waves in meters
- **Swell Period**: Period of swell waves in seconds
- **Swell Direction**: Direction of swell waves (0° = north)

### Provider-Specific Features

#### StormGlass
- Wind speeds **converted from m/s to knots** (×1.94384)
- Water temperature
- High-quality marine-focused data

#### OpenWeatherMap
- Wind speeds remain in **m/s** (no conversion)
- Comprehensive global coverage
- Standard meteorological parameters

#### Windy.com
- Wind speeds in **m/s**
- Separate wind waves and swell data
- Cloud cover (low/medium/high altitude layers)
- Precipitation data
- Calculated from wind components (u/v vectors)
- Temperature converted from Kelvin to Celsius

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

### StormGlass API
Marine-focused weather data with premium quality.
- **Documentation**: https://docs.stormglass.io/#/weather
- **API Key**: Free tier available
- **Output Units**: Wind speed in knots, temperature in Celsius

### OpenWeatherMap API
Global weather data with comprehensive coverage.
- **Documentation**: https://openweathermap.org/api
- **API Key**: Free tier available
- **Output Units**: Wind speed in m/s, temperature in Celsius

### Windy.com API
High-resolution weather models with advanced parameters.
- **Documentation**: https://api.windy.com/
- **API Key**: Registration required
- **Output Units**: Wind speed in m/s, temperature in Celsius
- **Special Features**: Separate wind waves and swell, multi-layer cloud data

### Adding New Providers
To integrate additional weather API providers, see [`ADDING_PROVIDERS.md`](ADDING_PROVIDERS.md) for a complete guide.

## License
