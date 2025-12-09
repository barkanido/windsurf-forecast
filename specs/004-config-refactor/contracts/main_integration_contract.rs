// API Contract: Main Function Integration
// Feature: 004-config-refactor
// Location: src/main.rs integration with src/config/ module

use anyhow::Result;
use crate::args::Args;
use crate::config::ResolvedConfig;

// ============================================================================
// CONTRACT: Simplified Main Function Structure
// ============================================================================

/// Main function integration with config module
///
/// CONTRACT GUARANTEES:
/// - Main function reduced from 275+ lines to ~100 lines
/// - Configuration logic moved to config module
/// - Clear phase separation in main function
/// - No precedence logic in main (all in config::resolver)
/// - No coordinate resolution in main (all in config::resolver)
/// - No timezone parsing in main (all in config::timezone)
///
/// TARGET STRUCTURE:
/// ```rust
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // Phase 1: Environment setup
///     dotenv::dotenv().ok();
///     let args = Args::parse();
///     
///     // Phase 2: Handle special flags
///     if args.list_providers {
///         list_providers();
///         return Ok(());
///     }
///     
///     if args.pick_timezone {
///         handle_interactive_timezone_picker(&args)?;
///         return Ok(());
///     }
///     
///     // Phase 3: Validate CLI arguments
///     validate_args(&args)?;
///     
///     // Phase 4: Resolve configuration (SINGLE CALL to config module)
///     let config = config::resolve_from_args_and_file(&args)?;
///     
///     // Phase 5: Display timezone warning if needed
///     // (Only if using default timezone)
///     
///     // Phase 6: Instantiate provider
///     let provider = provider_registry::create_provider(&config.provider)?;
///     
///     // Phase 7: Fetch weather data
///     let data = provider.fetch_forecast(
///         config.lat,
///         config.lng,
///         config.days_ahead,
///         config.first_day_offset,
///         config.timezone,
///     ).await?;
///     
///     // Phase 8: Output results
///     let json = serde_json::to_string_pretty(&data)?;
///     println!("{}", json);
///     
///     // Phase 9: Persistence (if --save flag provided)
///     if args.save {
///         config::save_config_from_resolved(&config, args.config.as_ref())?;
///         eprintln!("✓ Configuration saved");
///     }
///     
///     Ok(())
/// }
/// ```

// ============================================================================
// CONTRACT: Config Module Entry Point
// ============================================================================

/// Single entry point for configuration resolution
///
/// CONTRACT GUARANTEES:
/// - Main function calls ONLY this function for config resolution
/// - All precedence logic internal to config module
/// - Returns fully validated ResolvedConfig
/// - Errors include helpful context for users
///
/// SIGNATURE (from src/config/mod.rs):
/// pub fn resolve_from_args_and_file(args: &Args) -> Result<ResolvedConfig>
///
/// USAGE IN MAIN:
/// ```rust
/// let config = config::resolve_from_args_and_file(&args)?;
/// // config is now ready to use, all values validated
/// ```

// ============================================================================
// CONTRACT: Persistence Entry Point
// ============================================================================

/// Convert ResolvedConfig back to Config and save to file
///
/// CONTRACT GUARANTEES:
/// - Only called when args.save == true
/// - Converts ResolvedConfig → Config structure
/// - Saves to file specified by args.config or default
/// - Displays confirmation message to stderr
///
/// SIGNATURE (from src/config/mod.rs):
/// pub fn save_config_from_resolved(
///     resolved: &ResolvedConfig,
///     path: Option<&PathBuf>
/// ) -> Result<()>
///
/// IMPLEMENTATION:
/// ```rust
/// pub fn save_config_from_resolved(
///     resolved: &ResolvedConfig,
///     path: Option<&PathBuf>
/// ) -> Result<()> {
///     let config = Config {
///         general: GeneralConfig {
///             timezone: resolved.timezone.name().to_string(),
///             default_provider: resolved.provider.clone(),
///             lat: Some(resolved.lat),
///             lng: Some(resolved.lng),
///         },
///     };
///     
///     loader::save_config(&config, path)?;
///     
///     let config_path = path
///         .cloned()
///         .unwrap_or_else(|| loader::get_default_config_path().unwrap());
///     eprintln!("✓ Configuration saved to {}", config_path.display());
///     
///     Ok(())
/// }
/// ```
///
/// USAGE IN MAIN:
/// ```rust
/// if args.save {
///     config::save_config_from_resolved(&config, args.config.as_ref())?;
/// }
/// ```

// ============================================================================
// CONTRACT: Config Module Public API
// ============================================================================

/// Public exports from src/config/mod.rs
///
/// CONTRACT GUARANTEES:
/// - Minimal public surface area
/// - Main function only needs these exports
/// - Internal implementation hidden
///
/// REQUIRED EXPORTS:
/// ```rust
/// // src/config/mod.rs
/// pub mod types;
/// pub mod loader;
/// pub mod resolver;
/// pub mod timezone;
///
/// // Re-export for convenience
/// pub use types::ResolvedConfig;
/// pub use resolver::resolve_from_args_and_file;
/// pub use loader::{Config, GeneralConfig};
///
/// // New function for persistence
/// pub fn save_config_from_resolved(
///     resolved: &ResolvedConfig,
///     path: Option<&PathBuf>
/// ) -> Result<()> {
///     // Convert ResolvedConfig → Config → save
/// }
/// ```

// ============================================================================
// CONTRACT: Args.rs Changes
// ============================================================================

/// New CLI argument for explicit persistence
///
/// CONTRACT GUARANTEES:
/// - New --save flag added to Args structure
/// - No changes to existing argument names or types
/// - Backward compatible (new optional flag)
///
/// ADDITION TO Args STRUCT:
/// ```rust
/// #[derive(Parser, Debug)]
/// pub struct Args {
///     // ... existing fields unchanged ...
///     
///     /// Save configuration to file after successful execution
///     /// (Applies to provider, timezone, and coordinates)
///     #[arg(long)]
///     pub save: bool,
/// }
/// ```

// ============================================================================
// CONTRACT: Breaking Changes from Current Implementation
// ============================================================================

/// Timezone auto-save removal
///
/// CURRENT BEHAVIOR (TO BE REMOVED):
/// ```rust
/// // Current main.rs auto-saves timezone when provided via CLI
/// if args.timezone.is_some() {
///     config.general.timezone = timezone_config.timezone.name().to_string();
///     save_config(&config, args.config.as_ref())?;
/// }
/// ```
///
/// NEW BEHAVIOR:
/// ```rust
/// // New main.rs requires explicit --save flag
/// if args.save {
///     config::save_config_from_resolved(&config, args.config.as_ref())?;
/// }
/// ```
///
/// MIGRATION FOR USERS:
/// - Users who relied on automatic timezone saving must add --save flag once
/// - Example: `windsurf-forecast --timezone "America/New_York" --save`
/// - Subsequent runs can omit both flags (timezone persisted in config file)

// ============================================================================
// CONTRACT: Complexity Metrics
// ============================================================================

/// Main function complexity before and after refactor
///
/// BEFORE (current implementation):
/// - Lines of code: 275+
/// - Precedence implementations: 3 separate patterns (provider, timezone, coordinates)
/// - Validation sites: Scattered across main function
/// - Configuration variables: 6+ scattered variables
/// - Coordinate resolution: Inline with multiple .or() chains
///
/// AFTER (refactored):
/// - Lines of code: ~100 (>60% reduction)
/// - Precedence implementations: 0 (all in config module)
/// - Validation sites: 1 (config::resolve_from_args_and_file)
/// - Configuration variables: 1 (ResolvedConfig)
/// - Coordinate resolution: Hidden in config module
///
/// MAINTAINABILITY IMPROVEMENTS:
/// - Adding new config parameter: Change config module only
/// - Changing precedence logic: Change resolver.rs only
/// - Changing validation rules: Change resolver.rs only
/// - Main function remains stable across config changes

// ============================================================================
// CONTRACT: Testing Requirements
// ============================================================================

/// Integration tests for main function flow
///
/// REQUIRED TESTS:
/// 1. Full flow with CLI arguments only
/// 2. Full flow with config file only
/// 3. Full flow with CLI overriding config
/// 4. Full flow with --save flag persisting config
/// 5. Error handling for invalid coordinates
/// 6. Error handling for invalid date range
/// 7. Error handling for invalid provider
///
/// EXAMPLE TEST:
/// ```rust
/// #[tokio::test]
/// async fn test_main_flow_with_cli_args() {
///     let args = Args {
///         provider: "stormglass".to_string(),
///         timezone: Some("America/New_York".to_string()),
///         lat: Some(40.7128),
///         lng: Some(-74.0060),
///         days_ahead: 3,
///         first_day_offset: 0,
///         save: false,
///         // ... other fields
///     };
///     
///     let config = config::resolve_from_args_and_file(&args).unwrap();
///     
///     assert_eq!(config.provider, "stormglass");
///     assert_eq!(config.timezone.name(), "America/New_York");
///     assert_eq!(config.lat, 40.7128);
///     assert_eq!(config.lng, -74.0060);
///     assert_eq!(config.days_ahead, 3);
///     assert_eq!(config.first_day_offset, 0);
/// }
/// ```

// ============================================================================
// CONTRACT: Documentation Requirements
// ============================================================================

/// Help text updates for new --save flag
///
/// REQUIRED ADDITIONS TO ARGS HELP:
/// ```
/// --save
///     Save configuration to file after successful execution.
///     Persists provider, timezone, and coordinates to ~/.windsurf-config.toml
///     for future runs. Without this flag, CLI arguments are used only for
///     the current run.
///     
///     Example: windsurf-forecast --timezone "America/New_York" --save
/// ```
///
/// README.md UPDATES:
/// - Document new --save flag behavior
/// - Explain removal of automatic timezone persistence
/// - Provide migration guide for users relying on auto-save

// ============================================================================
// PUBLIC API SUMMARY
// ============================================================================

/// Complete public API between main.rs and config module
///
/// IMPORTS IN MAIN:
/// ```rust
/// use crate::config::{ResolvedConfig, resolve_from_args_and_file, save_config_from_resolved};
/// ```
///
/// FUNCTION CALLS IN MAIN:
/// ```rust
/// // Resolution (single call)
/// let config = resolve_from_args_and_file(&args)?;
///
/// // Persistence (conditional, single call)
/// if args.save {
///     save_config_from_resolved(&config, args.config.as_ref())?;
/// }
/// ```
///
/// NO OTHER CONFIG MODULE CALLS NEEDED IN MAIN