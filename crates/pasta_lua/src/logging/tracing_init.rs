//! Tracing subscriber initialization with per-directory filters.
//!
//! - `init_tracing()` - Stage 1: 購読者を 1 度だけ入れる（量の設定は持たない）
//! - `update_tracing_filter()` - Stage 1.5: そのディレクトリの項目の量の設定を替える
//!
//! 量の判定は [`GlobalLoggerRegistry`] の項目（ディレクトリごと）に任せる。プロセスに 1 つの
//! 量の設定は持たない。1 つの `EnvFilter` を項目ごとに `enabled` で呼ぶだけなので、スパンの
//! 中身で絞る動的な指定（`[span{field=…}]`）は効かない。ターゲットと量の指定は効く。

use std::path::Path;
use tracing::Metadata;
use tracing::subscriber::Interest;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::{Context, Filter};
use tracing_subscriber::prelude::*;

use super::GlobalLoggerRegistry;
use crate::loader::LoggingConfig;

/// SHIORI の読み込みの終わりの行の target。ログの設定によらず info を通す。
pub const LOAD_ENTRY_TARGET: &str = "pasta::load_entry";

/// Build an `EnvFilter` from `LoggingConfig`.
///
/// Priority: PASTA_LOG env var > config.filter > config.level.
/// A `PASTA_LOG` that is unset or unparsable is skipped; an unparsable
/// config directive falls back to "info".
/// どの枝でも [`LOAD_ENTRY_TARGET`] の info は通す。
pub(super) fn build_filter(config: &LoggingConfig) -> EnvFilter {
    build_filter_from(std::env::var("PASTA_LOG").ok().as_deref(), config)
}

/// 読んだ `PASTA_LOG` の値（`pasta_log`）と `config` から `EnvFilter` を作る。
fn build_filter_from(pasta_log: Option<&str>, config: &LoggingConfig) -> EnvFilter {
    pasta_log
        .ok_or(())
        .and_then(|s| EnvFilter::try_new(s).map_err(drop))
        .or_else(|_| EnvFilter::try_new(config.to_filter_directive()))
        .unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to parse log filter '{}', using default: {}",
                config.to_filter_directive(),
                e
            );
            EnvFilter::new("info")
        })
        // 後から足した指示が作者の同じ target の指示に勝つ
        .add_directive(
            format!("{LOAD_ENTRY_TARGET}=info")
                .parse()
                .expect("固定の指示"),
        )
}

/// 登録の項目の量の設定で判定する層のフィルタ。
struct RegistryFilter;

impl<S> Filter<S> for RegistryFilter {
    fn enabled(&self, meta: &Metadata<'_>, cx: &Context<'_, S>) -> bool {
        GlobalLoggerRegistry::instance().event_enabled(meta, cx)
    }

    // 判定はスレッドの文脈で変わるので、コールサイトごとに覚えさせない
    fn callsite_enabled(&self, _meta: &'static Metadata<'static>) -> Interest {
        Interest::sometimes()
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        GlobalLoggerRegistry::instance().max_level_hint()
    }
}

/// Initialize the global tracing subscriber.
///
/// Must be called once (Stage 1) before `PastaLoader::load()`.
/// Subsequent calls are safely ignored (`try_init()`).
pub fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(GlobalLoggerRegistry::instance().clone())
                .with_ansi(false)
                .with_target(true)
                .with_level(true)
                .with_filter(RegistryFilter),
        )
        .try_init();
}

/// `load_dir` の項目の量の設定を `config` から作り直す（項目が無ければ作る）。
///
/// Called at Stage 1.5 after `pasta.toml` is loaded, to apply
/// the user's `[logging]` configuration to that directory only.
pub fn update_tracing_filter(load_dir: &Path, config: &LoggingConfig) {
    GlobalLoggerRegistry::instance().set_filter(load_dir, build_filter(config));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tracing_subscriber::Layer;

    struct CountLayer(Arc<AtomicUsize>);

    impl<S: tracing::Subscriber> Layer<S> for CountLayer {
        fn on_event(&self, _event: &tracing::Event<'_>, _cx: Context<'_, S>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// `filter` で（読み込みの終わりの行の info, ほかの target の info）が通るか
    fn info_passes(filter: EnvFilter) -> (bool, bool) {
        let count = Arc::new(AtomicUsize::new(0));
        let subscriber =
            tracing_subscriber::registry().with(CountLayer(count.clone()).with_filter(filter));
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: LOAD_ENTRY_TARGET, "probe");
            let entry = count.swap(0, Ordering::SeqCst) == 1;
            tracing::info!(target: "pasta_lua::other", "probe");
            (entry, count.load(Ordering::SeqCst) == 1)
        })
    }

    fn level(level: &str) -> LoggingConfig {
        LoggingConfig {
            level: level.to_string(),
            ..LoggingConfig::default()
        }
    }

    #[test]
    fn test_load_entry_info_passes_level_warn() {
        assert_eq!(
            info_passes(build_filter_from(None, &level("warn"))),
            (true, false)
        );
    }

    #[test]
    fn test_load_entry_info_passes_pasta_log() {
        let f = build_filter_from(Some("error"), &level("info"));
        assert_eq!(info_passes(f), (true, false));
        let f = build_filter_from(Some("pasta_lua=debug,warn"), &level("info"));
        assert_eq!(info_passes(f), (true, true));
    }

    #[test]
    fn test_load_entry_info_passes_author_off() {
        let config = LoggingConfig {
            filter: Some(format!("{LOAD_ENTRY_TARGET}=off,warn")),
            ..LoggingConfig::default()
        };
        assert_eq!(info_passes(build_filter_from(None, &config)), (true, false));
    }

    #[test]
    fn test_load_entry_info_passes_fallback() {
        let config = LoggingConfig {
            filter: Some("===not a directive===".to_string()),
            ..LoggingConfig::default()
        };
        // 既定の info に落ちるので、ほかの target の info も通る
        assert_eq!(info_passes(build_filter_from(None, &config)), (true, true));
    }

    #[test]
    fn test_unparsable_pasta_log_falls_back_to_config() {
        let f = build_filter_from(Some("===not a directive==="), &level("warn"));
        assert_eq!(info_passes(f), (true, false));
    }

    /// Exercise the full two-stage lifecycle in a single test so the call
    /// order is deterministic (the subscriber is process-global).
    ///
    /// Contract under test:
    /// - `update_tracing_filter` never panics, with or without prior init
    /// - `init_tracing` is idempotent (second call is ignored)
    /// - updating with a valid or an unparsable filter never panics
    ///   (unparsable input falls back to the "info" filter)
    #[test]
    fn test_init_and_update_lifecycle_no_panic() {
        let config = LoggingConfig::default();
        // 他の試験と重ならないディレクトリ（終わりに項目を消す）
        let dir = Path::new("/tracing_init/lifecycle");

        // Update before (or regardless of) init: must not panic.
        update_tracing_filter(dir, &config);

        // Stage 1: install subscriber.
        init_tracing();

        // Second init must be safely ignored (try_init contract).
        init_tracing();

        // Stage 1.5: update with a valid custom directive.
        let custom = LoggingConfig {
            filter: Some("warn,pasta_lua=debug".to_string()),
            ..LoggingConfig::default()
        };
        update_tracing_filter(dir, &custom);

        // Update with an unparsable directive: falls back without panicking.
        let invalid = LoggingConfig {
            filter: Some("===not a directive===".to_string()),
            ..LoggingConfig::default()
        };
        update_tracing_filter(dir, &invalid);

        GlobalLoggerRegistry::instance().unregister(dir);
    }
}
