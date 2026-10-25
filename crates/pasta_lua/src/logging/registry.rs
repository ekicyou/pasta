//! Global Logger Registry - Manages multiple PastaLogger instances.
//!
//! This module provides a singleton registry for PastaLogger instances,
//! allowing log routing based on the current load_dir context.
//!
//! # Routing rules
//!
//! Each log event is routed by the calling thread's load_dir context:
//!
//! 1. Context set -> the logger registered for that load_dir; discarded if
//!    none is registered (never written to another instance's logger).
//! 2. No context and exactly one logger registered -> that logger.
//! 3. Otherwise (no context with zero or two or more loggers) -> discarded.
//!
//! Discarded output still reports success. Threads that know their load_dir
//! (actor threads, the loader) set the context with [`LoadDirGuard`] so that
//! routing stays correct when several loggers are registered.
//!
//! # 項目と占有
//!
//! 表の 1 項目は「ロガー（無いこともある）＋量の設定＋世代」。項目が在ること＝そのディレクトリが
//! 占有されていること。`try_claim` は項目が無いときだけ札（[`ClaimToken`]）を返し、
//! `release_claim` は札の世代が今の世代と同じときだけ項目を消す。量の判定も上の選び方で選んだ
//! 項目の量の設定で行う。
//!
//! # Usage
//!
//! Applications using pasta_lua (SHIORI DLLs, CLI tools, etc.) can register
//! their PastaLogger instances with this registry for centralized log routing.
//!
//! ```rust,ignore
//! use pasta_lua::logging::{GlobalLoggerRegistry, LoadDirGuard, PastaLogger};
//!
//! // Register a logger for a specific load_dir
//! let logger = Arc::new(PastaLogger::new(load_dir, config)?);
//! GlobalLoggerRegistry::instance().register(load_dir.clone(), logger);
//!
//! // Set context for log routing
//! let _guard = LoadDirGuard::new(load_dir);
//! tracing::info!("This goes to the registered logger");
//! ```

use super::PastaLogger;
use super::tracing_init::build_filter;
use crate::loader::LoggingConfig;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use tracing::Metadata;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Context;

/// Global singleton instance of the logger registry.
static REGISTRY: OnceLock<GlobalLoggerRegistry> = OnceLock::new();

/// 表の 1 項目（ゴーストのディレクトリ 1 つ）。項目が在ること＝占有されていること。
struct Entry {
    /// 書き先のロガー（占有だけでまだ無いこともある）
    logger: Option<Arc<PastaLogger>>,
    /// この項目の量の設定
    filter: EnvFilter,
    /// 占有の世代（札と比べる）
    generation: u64,
}

/// ディレクトリ→項目の表と、世代の数。
#[derive(Default)]
struct Table {
    entries: HashMap<PathBuf, Entry>,
    /// 次に配る世代。項目を消しても戻さないので、古い札が新しい項目と重ならない
    next_generation: u64,
}

impl Table {
    fn new_generation(&mut self) -> u64 {
        self.next_generation += 1;
        self.next_generation
    }

    /// 項目が無ければ「ロガー無し・既定の量」で新しい世代の項目を足す
    fn entry_or_default(&mut self, load_dir: PathBuf) -> &mut Entry {
        if !self.entries.contains_key(&load_dir) {
            let generation = self.new_generation();
            self.entries.insert(
                load_dir.clone(),
                Entry {
                    logger: None,
                    filter: build_filter(&LoggingConfig::default()),
                    generation,
                },
            );
        }
        self.entries.get_mut(&load_dir).expect("inserted above")
    }

    /// 項目の選び方: 文脈があればその項目。無ければロガーを持つ項目がちょうど 1 つならそれ。
    /// ほかは選ばない（ほかのゴーストの設定へ逃げない）。
    fn select(&self, context: Option<&Path>) -> Option<&Entry> {
        match context {
            Some(dir) => self.entries.get(dir),
            None => {
                let mut with_logger = self.entries.values().filter(|e| e.logger.is_some());
                match (with_logger.next(), with_logger.next()) {
                    (Some(entry), None) => Some(entry),
                    _ => None,
                }
            }
        }
    }
}

/// ディレクトリの占有の札（鍵＋世代）。
///
/// [`GlobalLoggerRegistry::release_claim`] は、札の世代が項目の今の世代と同じときだけ項目を消す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimToken {
    load_dir: PathBuf,
    generation: u64,
}

/// Global Logger Registry - ディレクトリごとのロガー・量の設定・占有を 1 つの表で持つ。
///
/// Each application instance registers its PastaLogger with load_dir as key.
/// The registry routes log output based on the current thread's load_dir context
/// (see the module-level routing rules).
///
/// This is useful for:
/// - SHIORI DLLs with multiple ghost instances
/// - CLI tools managing multiple projects
/// - Any application needing instance-specific logging
#[derive(Clone)]
pub struct GlobalLoggerRegistry {
    /// Map of load_dir -> entry (logger, filter, generation)
    table: Arc<Mutex<Table>>,
}

impl GlobalLoggerRegistry {
    /// Create a new empty registry.
    fn new() -> Self {
        Self {
            table: Arc::new(Mutex::new(Table::default())),
        }
    }

    /// Get the singleton instance.
    pub fn instance() -> &'static Self {
        REGISTRY.get_or_init(GlobalLoggerRegistry::new)
    }

    /// Lock the table, recovering from a poisoned mutex.
    fn table(&self) -> MutexGuard<'_, Table> {
        self.table
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// 表を変えた後（ロックを外した後）に、コールサイトの関心の表を作り直す。
    /// ロックを持ったまま呼ぶと、購読者のフィルタの `max_level_hint` が同じロックを取って止まる。
    fn rebuild_interest() {
        tracing::callsite::rebuild_interest_cache();
    }

    /// Register a logger for the given load_dir.
    ///
    /// 項目のロガーを入れ替える。量の設定は残す（項目が無ければ既定の量で足す）。
    /// The replaced logger is dropped (flushed) after the lock is released.
    pub fn register(&self, load_dir: PathBuf, logger: Arc<PastaLogger>) {
        let replaced = self
            .table()
            .entry_or_default(load_dir)
            .logger
            .replace(logger);
        drop(replaced);
        Self::rebuild_interest();
    }

    /// Unregister the given load_dir.
    ///
    /// 項目ごと消す（ロガー・量の設定・占有）。世代は見ない。
    /// The removed logger is dropped (flushed) after the lock is released.
    pub fn unregister(&self, load_dir: &Path) {
        let removed = self.table().entries.remove(load_dir);
        drop(removed);
        Self::rebuild_interest();
    }

    /// Get the logger for the given load_dir.
    pub fn get(&self, load_dir: &Path) -> Option<Arc<PastaLogger>> {
        self.table()
            .entries
            .get(load_dir)
            .and_then(|e| e.logger.clone())
    }

    /// ディレクトリを占有する。項目が無ければ「ロガー無し・既定の量」の項目を新しい世代で足して札を返す。
    /// 項目が在れば（占有されていれば）`None`。
    pub fn try_claim(&self, load_dir: PathBuf) -> Option<ClaimToken> {
        let token = {
            let mut table = self.table();
            if table.entries.contains_key(&load_dir) {
                return None;
            }
            let generation = table.entry_or_default(load_dir.clone()).generation;
            ClaimToken {
                load_dir,
                generation,
            }
        };
        Self::rebuild_interest();
        Some(token)
    }

    /// 占有を新しい世代で奪って札を返す（ロガーと量の設定は引き継ぐ。項目が無ければ足す）。
    /// 旧来の口の再読み込みだけが使う。
    pub fn take_over(&self, load_dir: PathBuf) -> ClaimToken {
        let token = {
            let mut table = self.table();
            let generation = table.new_generation();
            table.entry_or_default(load_dir.clone()).generation = generation;
            ClaimToken {
                load_dir,
                generation,
            }
        };
        Self::rebuild_interest();
        token
    }

    /// 札の世代が項目の今の世代と同じときだけ項目を消す（古い世代の札では何もしない）。
    pub fn release_claim(&self, token: &ClaimToken) {
        let removed = {
            let mut table = self.table();
            let current = table
                .entries
                .get(&token.load_dir)
                .is_some_and(|e| e.generation == token.generation);
            if !current {
                return;
            }
            table.entries.remove(&token.load_dir)
        };
        drop(removed);
        Self::rebuild_interest();
    }

    /// 項目の量の設定を替える（項目が無ければ作る）。
    pub(crate) fn set_filter(&self, load_dir: &Path, filter: EnvFilter) {
        let replaced = std::mem::replace(
            &mut self.table().entry_or_default(load_dir.to_path_buf()).filter,
            filter,
        );
        drop(replaced);
        Self::rebuild_interest();
    }

    /// スレッドの文脈で選んだ項目の量の設定でイベントを判定する。選べなければ出さない。
    pub(crate) fn event_enabled<S>(&self, meta: &Metadata<'_>, cx: &Context<'_, S>) -> bool {
        CURRENT_LOAD_DIR.with(|cell| {
            self.table()
                .select(cell.borrow().as_deref())
                .is_some_and(|e| e.filter.enabled(meta, cx.clone()))
        })
    }

    /// 全部の項目の量の最大（項目が無ければ OFF。分からない項目が 1 つでもあれば `None`）。
    pub(crate) fn max_level_hint(&self) -> Option<LevelFilter> {
        self.table()
            .entries
            .values()
            .try_fold(LevelFilter::OFF, |max, e| {
                e.filter.max_level_hint().map(|hint| max.max(hint))
            })
    }

    /// Decide which logger a log event goes to (項目の選び方は量の判定と同じ).
    fn resolve(&self, context: Option<&Path>) -> Option<Arc<PastaLogger>> {
        self.table().select(context).and_then(|e| e.logger.clone())
    }
}

/// Writer that routes to the appropriate PastaLogger based on current context.
///
/// Uses thread-local storage to determine which logger to use.
pub struct RoutingWriter {
    /// The logger to write to, or None for no-op.
    logger: Option<Arc<PastaLogger>>,
}

impl Write for RoutingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(ref logger) = self.logger {
            logger.write(buf)
        } else {
            // No-op: silently discard
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref logger) = self.logger {
            logger.flush()
        } else {
            Ok(())
        }
    }
}

impl<'a> MakeWriter<'a> for GlobalLoggerRegistry {
    type Writer = RoutingWriter;

    fn make_writer(&'a self) -> Self::Writer {
        let logger = CURRENT_LOAD_DIR.with(|cell| self.resolve(cell.borrow().as_deref()));
        RoutingWriter { logger }
    }
}

// Thread-local storage for the current load_dir context.
//
// Set this before logging to route logs to the correct file.
thread_local! {
    static CURRENT_LOAD_DIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Set the current load_dir context for logging.
///
/// All log messages in this thread will be routed to the logger
/// registered for this load_dir.
pub fn set_current_load_dir(load_dir: Option<PathBuf>) {
    CURRENT_LOAD_DIR.with(|cell| {
        *cell.borrow_mut() = load_dir;
    });
}

/// Get the current load_dir context.
pub fn get_current_load_dir() -> Option<PathBuf> {
    CURRENT_LOAD_DIR.with(|cell| cell.borrow().clone())
}

/// Guard that sets the load_dir context and restores it on drop.
///
/// Use this to ensure proper context cleanup in scoped operations.
pub struct LoadDirGuard {
    previous: Option<PathBuf>,
}

impl LoadDirGuard {
    /// Create a new guard that sets the load_dir context.
    pub fn new(load_dir: PathBuf) -> Self {
        let previous = get_current_load_dir();
        set_current_load_dir(Some(load_dir));
        Self { previous }
    }
}

impl Drop for LoadDirGuard {
    fn drop(&mut self) {
        set_current_load_dir(self.previous.take());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_singleton() {
        let r1 = GlobalLoggerRegistry::instance();
        let r2 = GlobalLoggerRegistry::instance();
        // Same address
        assert!(std::ptr::eq(r1, r2));
    }

    #[test]
    fn test_load_dir_context() {
        // Initially None
        assert!(get_current_load_dir().is_none());

        // Set and get
        set_current_load_dir(Some(PathBuf::from("/test/path")));
        assert_eq!(get_current_load_dir(), Some(PathBuf::from("/test/path")));

        // Clear
        set_current_load_dir(None);
        assert!(get_current_load_dir().is_none());
    }

    #[test]
    fn test_load_dir_guard() {
        set_current_load_dir(Some(PathBuf::from("/original")));

        {
            let _guard = LoadDirGuard::new(PathBuf::from("/guarded"));
            assert_eq!(get_current_load_dir(), Some(PathBuf::from("/guarded")));
        }

        // Restored after guard drops
        assert_eq!(get_current_load_dir(), Some(PathBuf::from("/original")));

        // Cleanup
        set_current_load_dir(None);
    }

    #[test]
    fn test_load_dir_guard_nested() {
        // Nested guards must restore the previous context level by level.
        set_current_load_dir(None);

        {
            let _outer = LoadDirGuard::new(PathBuf::from("/outer"));
            assert_eq!(get_current_load_dir(), Some(PathBuf::from("/outer")));

            {
                let _inner = LoadDirGuard::new(PathBuf::from("/inner"));
                assert_eq!(get_current_load_dir(), Some(PathBuf::from("/inner")));
            }

            // Inner guard restores outer context
            assert_eq!(get_current_load_dir(), Some(PathBuf::from("/outer")));
        }

        // Outer guard restores the original (None) context
        assert!(get_current_load_dir().is_none());
    }

    /// A logger writing under a fresh temp dir (kept alive by the returned TempDir).
    fn temp_logger() -> (tempfile::TempDir, Arc<PastaLogger>) {
        let dir = tempfile::TempDir::new().unwrap();
        let logger = Arc::new(PastaLogger::new(dir.path(), None).unwrap());
        (dir, logger)
    }

    fn same(a: Option<Arc<PastaLogger>>, b: &Arc<PastaLogger>) -> bool {
        a.is_some_and(|a| Arc::ptr_eq(&a, b))
    }

    // The tests below use a private registry (not the process-wide singleton),
    // so they are not affected by other tests running in parallel.

    #[test]
    fn test_resolve_no_context_zero_loggers_discards() {
        let registry = GlobalLoggerRegistry::new();
        assert!(registry.resolve(None).is_none());
    }

    #[test]
    fn test_resolve_no_context_sole_logger() {
        let registry = GlobalLoggerRegistry::new();
        let (dir, logger) = temp_logger();
        registry.register(dir.path().to_path_buf(), logger.clone());
        assert!(same(registry.resolve(None), &logger));
    }

    #[test]
    fn test_resolve_no_context_two_loggers_discards() {
        let registry = GlobalLoggerRegistry::new();
        let (dir_a, a) = temp_logger();
        let (dir_b, b) = temp_logger();
        registry.register(dir_a.path().to_path_buf(), a);
        registry.register(dir_b.path().to_path_buf(), b);
        assert!(registry.resolve(None).is_none());
    }

    #[test]
    fn test_resolve_context_registered() {
        let registry = GlobalLoggerRegistry::new();
        let (dir_a, a) = temp_logger();
        let (dir_b, b) = temp_logger();
        registry.register(dir_a.path().to_path_buf(), a.clone());
        registry.register(dir_b.path().to_path_buf(), b.clone());
        assert!(same(registry.resolve(Some(dir_a.path())), &a));
        assert!(same(registry.resolve(Some(dir_b.path())), &b));
    }

    #[test]
    fn test_resolve_context_unregistered_does_not_fall_back() {
        // Another ghost's logger is the only one registered: still discard (4.6).
        let registry = GlobalLoggerRegistry::new();
        let (dir, logger) = temp_logger();
        registry.register(dir.path().to_path_buf(), logger);
        let other = tempfile::TempDir::new().unwrap();
        assert!(registry.resolve(Some(other.path())).is_none());
    }

    #[test]
    fn test_registry_register_get_unregister() {
        let registry = GlobalLoggerRegistry::new();
        let (dir, logger) = temp_logger();
        let load_dir = dir.path().to_path_buf();

        assert!(registry.get(&load_dir).is_none());

        registry.register(load_dir.clone(), logger.clone());
        assert!(same(registry.get(&load_dir), &logger));

        registry.unregister(&load_dir);
        assert!(registry.get(&load_dir).is_none());
    }

    #[test]
    fn test_registry_register_replaces_existing() {
        let registry = GlobalLoggerRegistry::new();
        let (dir, first) = temp_logger();
        let (_dir_b, second) = temp_logger();
        let key = dir.path().to_path_buf();

        registry.register(key.clone(), first);
        registry.register(key.clone(), second.clone());

        assert!(
            same(registry.get(&key), &second),
            "second registration should replace the first"
        );
    }

    #[test]
    fn test_make_writer_without_logger_is_noop() {
        // No logger resolved (empty registry, with and without context):
        // the routing writer silently discards output but still reports success.
        let registry = GlobalLoggerRegistry::new();
        let other = tempfile::TempDir::new().unwrap();

        for context in [None, Some(other.path().to_path_buf())] {
            set_current_load_dir(context);
            let mut writer = registry.make_writer();
            assert!(writer.logger.is_none());
            let n = writer.write(b"discarded").unwrap();
            assert_eq!(n, b"discarded".len());
            writer.flush().unwrap();
        }
        set_current_load_dir(None);
    }

    #[test]
    fn test_make_writer_routes_to_registered_logger() {
        // End-to-end: register a logger, set the context, write through the
        // MakeWriter interface, then verify the bytes reached the log file.
        let registry = GlobalLoggerRegistry::new();
        let (dir, logger) = temp_logger();
        let load_dir = dir.path().to_path_buf();
        let log_path = logger.log_path().to_path_buf();
        registry.register(load_dir.clone(), logger);

        {
            let _guard = LoadDirGuard::new(load_dir.clone());
            let mut writer = registry.make_writer();
            writer.write_all(b"routed via registry\n").unwrap();
            writer.flush().unwrap();
        }

        // Drop the last Arc reference so the worker guard flushes on drop.
        registry.unregister(&load_dir);

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(
            content.contains("routed via registry"),
            "log file should contain the routed line, got: {:?}",
            content
        );
    }

    // ---- 占有（6.2） ----

    #[test]
    fn test_try_claim_only_first_time_and_again_after_release() {
        let registry = GlobalLoggerRegistry::new();
        let dir = tempfile::TempDir::new().unwrap();
        let key = dir.path().to_path_buf();

        let token = registry.try_claim(key.clone()).expect("空の表なら札が返る");
        assert!(registry.try_claim(key.clone()).is_none(), "2 回目は断る");

        registry.release_claim(&token);
        assert!(registry.try_claim(key).is_some(), "解放の後は再び札が返る");
    }

    #[test]
    fn test_try_claim_trailing_separator_is_same_key() {
        let registry = GlobalLoggerRegistry::new();
        let dir = tempfile::TempDir::new().unwrap();
        let mut with_sep = dir.path().as_os_str().to_os_string();
        with_sep.push(std::path::MAIN_SEPARATOR_STR);

        let _token = registry.try_claim(dir.path().to_path_buf()).unwrap();
        assert!(
            registry.try_claim(PathBuf::from(with_sep)).is_none(),
            "末尾の区切りの違う同じパスは同じ鍵"
        );
    }

    #[test]
    fn test_take_over_old_token_keeps_entry_new_token_removes() {
        let registry = GlobalLoggerRegistry::new();
        let (dir, logger) = temp_logger();
        let key = dir.path().to_path_buf();

        let old = registry.try_claim(key.clone()).unwrap();
        registry.register(key.clone(), logger.clone());
        let new = registry.take_over(key.clone());
        assert_ne!(old, new);
        assert!(same(registry.get(&key), &logger), "ロガーは引き継ぐ");

        registry.release_claim(&old);
        assert!(registry.get(&key).is_some(), "古い札では項目は消えない");
        assert!(registry.try_claim(key.clone()).is_none(), "占有も残る");

        registry.release_claim(&new);
        assert!(registry.get(&key).is_none(), "新しい札でなら消える");
        assert!(registry.try_claim(key).is_some());
    }

    #[test]
    fn test_old_token_does_not_remove_reclaimed_entry() {
        // 消えた後に別の持ち主が占有し直しても、古い札の世代とは重ならない
        let registry = GlobalLoggerRegistry::new();
        let dir = tempfile::TempDir::new().unwrap();
        let key = dir.path().to_path_buf();

        let old = registry.try_claim(key.clone()).unwrap();
        let mid = registry.take_over(key.clone());
        registry.release_claim(&mid);
        let _current = registry.try_claim(key.clone()).unwrap();

        registry.release_claim(&old);
        assert!(registry.try_claim(key).is_none(), "今の持ち主の占有は残る");
    }

    // ---- 量の設定と項目の選び方（5.3・5.4） ----

    use std::sync::atomic::{AtomicUsize, Ordering};
    use tracing_subscriber::layer::{Context, Filter, Layer, SubscriberExt};

    /// 試験用: 登録の項目で判定するフィルタ（3.2 の購読者のフィルタと同じ形）
    struct TestFilter(GlobalLoggerRegistry);

    impl<S: tracing::Subscriber> Filter<S> for TestFilter {
        fn enabled(&self, meta: &tracing::Metadata<'_>, cx: &Context<'_, S>) -> bool {
            self.0.event_enabled(meta, cx)
        }
        fn callsite_enabled(
            &self,
            _meta: &'static tracing::Metadata<'static>,
        ) -> tracing::subscriber::Interest {
            tracing::subscriber::Interest::sometimes()
        }
        fn max_level_hint(&self) -> Option<LevelFilter> {
            self.0.max_level_hint()
        }
    }

    /// 試験用: 通ったイベントを数える層
    struct CountLayer(Arc<AtomicUsize>);

    impl<S: tracing::Subscriber> Layer<S> for CountLayer {
        fn on_event(&self, _event: &tracing::Event<'_>, _cx: Context<'_, S>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// `registry` の判定で info のイベントが通るかを、文脈 `context` で確かめる
    fn info_passes(registry: &GlobalLoggerRegistry, context: Option<&Path>) -> bool {
        let count = Arc::new(AtomicUsize::new(0));
        let subscriber = tracing_subscriber::registry()
            .with(CountLayer(count.clone()).with_filter(TestFilter(registry.clone())));
        set_current_load_dir(context.map(Path::to_path_buf));
        tracing::subscriber::with_default(subscriber, || tracing::info!("probe"));
        set_current_load_dir(None);
        count.load(Ordering::SeqCst) == 1
    }

    #[test]
    fn test_event_enabled_context_uses_that_entry() {
        let registry = GlobalLoggerRegistry::new();
        let (dir_a, a) = temp_logger();
        let (dir_b, b) = temp_logger();
        registry.register(dir_a.path().to_path_buf(), a);
        registry.register(dir_b.path().to_path_buf(), b);
        registry.set_filter(dir_a.path(), EnvFilter::new("warn"));
        registry.set_filter(dir_b.path(), EnvFilter::new("info"));

        assert!(!info_passes(&registry, Some(dir_a.path())));
        assert!(info_passes(&registry, Some(dir_b.path())));

        // 登録の無いディレクトリの文脈は、ほかのゴーストの設定へ逃げない
        let other = tempfile::TempDir::new().unwrap();
        assert!(!info_passes(&registry, Some(other.path())));
    }

    #[test]
    fn test_event_enabled_no_context_sole_logger_entry() {
        // ロガーを持つ項目は A だけ（C は占有だけでロガー無し）→ A の量で判定
        let registry = GlobalLoggerRegistry::new();
        let (dir_a, a) = temp_logger();
        let dir_c = tempfile::TempDir::new().unwrap();
        registry.register(dir_a.path().to_path_buf(), a.clone());
        registry.set_filter(dir_a.path(), EnvFilter::new("info"));
        let _claim = registry.try_claim(dir_c.path().to_path_buf()).unwrap();
        registry.set_filter(dir_c.path(), EnvFilter::new("warn"));

        assert!(info_passes(&registry, None));
        assert!(same(registry.resolve(None), &a));

        registry.set_filter(dir_a.path(), EnvFilter::new("warn"));
        assert!(!info_passes(&registry, None));
    }

    #[test]
    fn test_event_enabled_no_context_two_loggers_selects_none() {
        let registry = GlobalLoggerRegistry::new();
        let (dir_a, a) = temp_logger();
        let (dir_b, b) = temp_logger();
        registry.register(dir_a.path().to_path_buf(), a);
        registry.register(dir_b.path().to_path_buf(), b);
        registry.set_filter(dir_a.path(), EnvFilter::new("info"));
        registry.set_filter(dir_b.path(), EnvFilter::new("info"));

        assert!(!info_passes(&registry, None));
    }

    #[test]
    fn test_set_filter_while_subscribed_takes_effect() {
        // 購読者が入ったまま量を替えても止まらず（ロックの外で作り直す）、判定が変わる
        let registry = GlobalLoggerRegistry::new();
        let (dir, logger) = temp_logger();
        registry.register(dir.path().to_path_buf(), logger);
        registry.set_filter(dir.path(), EnvFilter::new("warn"));

        let count = Arc::new(AtomicUsize::new(0));
        let subscriber = tracing_subscriber::registry()
            .with(CountLayer(count.clone()).with_filter(TestFilter(registry.clone())));
        tracing::subscriber::with_default(subscriber, || {
            let _guard = LoadDirGuard::new(dir.path().to_path_buf());
            tracing::debug!("before");
            registry.set_filter(dir.path(), EnvFilter::new("debug"));
            tracing::debug!("after");
        });
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_max_level_hint_is_max_of_entries() {
        let registry = GlobalLoggerRegistry::new();
        assert_eq!(registry.max_level_hint(), Some(LevelFilter::OFF));

        let dir_a = tempfile::TempDir::new().unwrap();
        let dir_b = tempfile::TempDir::new().unwrap();
        registry.set_filter(dir_a.path(), EnvFilter::new("warn"));
        registry.set_filter(dir_b.path(), EnvFilter::new("debug"));
        assert_eq!(registry.max_level_hint(), Some(LevelFilter::DEBUG));

        registry.unregister(dir_b.path());
        assert_eq!(registry.max_level_hint(), Some(LevelFilter::WARN));
    }

    #[test]
    fn test_register_keeps_filter() {
        let registry = GlobalLoggerRegistry::new();
        let (dir, logger) = temp_logger();
        registry.set_filter(dir.path(), EnvFilter::new("debug"));
        registry.register(dir.path().to_path_buf(), logger);
        assert_eq!(registry.max_level_hint(), Some(LevelFilter::DEBUG));
    }
}
