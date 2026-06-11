//! Tracing subscriber initialization with dynamic filter reload.
//!
//! Provides two-stage initialization:
//! - `init_tracing_with_reload()` - Stage 1: initialize subscriber with reload::Layer
//! - `update_tracing_filter()` - Stage 1.5: update filter via saved handle

use std::sync::OnceLock;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::reload;

use super::GlobalLoggerRegistry;
use crate::loader::LoggingConfig;

/// Handle type for dynamically reloading the EnvFilter.
type FilterHandle = reload::Handle<EnvFilter, tracing_subscriber::Registry>;

/// Global handle for filter reload (set once by `init_tracing_with_reload`).
static FILTER_HANDLE: OnceLock<FilterHandle> = OnceLock::new();

/// Build an `EnvFilter` from `LoggingConfig`.
///
/// Priority: PASTA_LOG env var > config.filter > config.level > default ("debug")
fn build_filter(config: &LoggingConfig) -> EnvFilter {
    EnvFilter::try_from_env("PASTA_LOG")
        .or_else(|_| EnvFilter::try_new(config.to_filter_directive()))
        .unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to parse log filter '{}', using default: {}",
                config.to_filter_directive(),
                e
            );
            EnvFilter::new("info")
        })
}

/// Initialize the global tracing subscriber with a reloadable filter layer.
///
/// Must be called once (Stage 1) before `PastaLoader::load()`.
/// Subsequent calls are safely ignored (`try_init()`).
///
/// The `reload::Handle` is stored in a `OnceLock` so that
/// `update_tracing_filter()` can swap in a new `EnvFilter` later.
pub fn init_tracing_with_reload(config: &LoggingConfig) {
    let filter = build_filter(config);

    let (filter_layer, handle) = reload::Layer::new(filter);

    let result = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(GlobalLoggerRegistry::instance().clone())
                .with_ansi(false)
                .with_target(true)
                .with_level(true)
                .with_filter(filter_layer),
        )
        .try_init();

    if result.is_ok() {
        let _ = FILTER_HANDLE.set(handle);
    }
}

/// Update the tracing filter via the saved reload handle.
///
/// Called at Stage 1.5 after `pasta.toml` is loaded, to apply
/// the user's `[logging]` configuration.
///
/// No-op if `init_tracing_with_reload()` was not called yet.
pub fn update_tracing_filter(config: &LoggingConfig) {
    if let Some(handle) = FILTER_HANDLE.get() {
        let new_filter = build_filter(config);
        let _ = handle.reload(new_filter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercise the full two-stage lifecycle in a single test so the call
    /// order is deterministic (the subscriber and FILTER_HANDLE are
    /// process-global).
    ///
    /// Contract under test:
    /// - `update_tracing_filter` never panics, with or without prior init
    /// - `init_tracing_with_reload` is idempotent (second call is ignored)
    /// - reloading with a valid or an unparsable filter never panics
    ///   (unparsable input falls back to the "info" filter)
    #[test]
    fn test_init_and_update_lifecycle_no_panic() {
        let config = LoggingConfig::default();

        // Update before (or regardless of) init: must not panic.
        update_tracing_filter(&config);

        // Stage 1: install subscriber + reload handle.
        init_tracing_with_reload(&config);

        // Second init must be safely ignored (try_init contract).
        init_tracing_with_reload(&config);

        // Stage 1.5: reload with a valid custom directive.
        let custom = LoggingConfig {
            filter: Some("warn,pasta_lua=debug".to_string()),
            ..LoggingConfig::default()
        };
        update_tracing_filter(&custom);

        // Reload with an unparsable directive: falls back without panicking.
        let invalid = LoggingConfig {
            filter: Some("===not a directive===".to_string()),
            ..LoggingConfig::default()
        };
        update_tracing_filter(&invalid);
    }
}
