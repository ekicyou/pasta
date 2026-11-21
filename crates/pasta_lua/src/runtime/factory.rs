//! Factory constructors for PastaLuaRuntime used by PastaLoader.

use super::PastaLuaRuntime;
use super::lua_require;
use super::register_finalize_scene;
use crate::context::TranspileContext;
use crate::debug::source_map::SourceMap;
use crate::loader::{LoaderContext, PastaConfig};
use crate::logging::PastaLogger;
use crate::runtime::runtime_config::RuntimeConfig;
use mlua::ErrorContext;
use mlua::Lua;
use mlua::Result as LuaResult;
use std::sync::Arc;

impl PastaLuaRuntime {
    /// Create a runtime from LoaderContext with scene_dic.lua loading.
    ///
    /// This is the factory method used by PastaLoader with incremental transpilation.
    /// It loads scene_dic.lua, which requires all cached scene modules.
    ///
    /// # Initialization Sequence
    /// 1. Setup package.path for module resolution
    /// 2. Register Rust modules (@pasta_config, @enc, @pasta_persistence, @pasta_log, @pasta_sakura_script)
    /// 3. Register finalize_scene Rust binding
    /// 4. require("main") - 利用者初期化スクリプト。失敗は**致命**
    /// 5. require("pasta.shiori.entry") - SHIORI 応答関数。失敗は**致命**
    /// 6. require("pasta.scene_dic") - シーン読み込みと finalize。失敗は**致命**
    /// 7. シーン identity 索引の突合（デバッグ有効時のみ）。best-effort で、失敗しても
    ///    起動は継続する
    ///
    /// 起動モジュール（4〜6）の失敗は、`module` / `fatal` フィールド付きの error ログと、
    /// 起動モジュール名の文脈を付けた `Err` になる。
    ///
    /// finalize 前の `@pasta_search` には常に空の登録表を渡す（キャッシュの状態によらず
    /// 見つからない）。finalize が Lua 側の登録から作り直す。
    ///
    /// # Arguments
    /// * `loader_context` - Configuration and paths from PastaLoader
    /// * `runtime_config` - Runtime configuration
    /// * `pasta_config` - Pasta configuration from pasta.toml
    /// * `logger` - Optional instance-specific logger (Arc-wrapped for sharing)
    /// * `source_map` - シーンマップ。ローダがデバッグを有効と解決したときだけ `Some`。
    ///   その有無が「デバッグが有効か」になる（`@pasta_debug.enabled()` も同じ値）。
    ///
    /// # Returns
    /// * `Ok(Self)` - Runtime initialized and scene_dic loaded
    /// * `Err(e)` - Initialization or scene_dic loading failed
    pub fn from_loader_with_scene_dic(
        loader_context: LoaderContext,
        runtime_config: RuntimeConfig,
        pasta_config: Option<PastaConfig>,
        logger: Option<Arc<PastaLogger>>,
        source_map: Option<Arc<SourceMap>>,
    ) -> LuaResult<Self> {
        // finalize_scene must re-register @pasta_search with the SAME alias table
        // as the initial registration inside `with_config_and_source_map`.
        let scene_aliases = runtime_config.scene_aliases.clone();

        let mut runtime =
            Self::with_config_and_source_map(TranspileContext::new(), runtime_config, source_map)?;

        runtime.logger = logger;
        // Store config and base_dir for Drop-time persistence save
        runtime.base_dir = Some(loader_context.base_dir.clone());
        runtime.config = pasta_config;

        Self::setup_package_path(&runtime.lua, &loader_context)?;
        Self::register_config_module(&runtime.lua, &loader_context.custom_fields)?;
        Self::register_enc_module(&runtime.lua)?;
        Self::register_persistence_module(&runtime.lua, &runtime.config, &runtime.base_dir)?;
        Self::register_log_module(&runtime.lua)?;
        Self::register_sakura_script_module(&runtime.lua, &runtime.config)?;
        Self::register_debug_module(
            &runtime.lua,
            runtime.debug_enabled(),
            runtime.source_map.clone(),
            &loader_context.base_dir,
        )?;

        // Overwrites the Lua stub; must precede scene_dic.lua, which calls finalize_scene().
        register_finalize_scene(&runtime.lua, scene_aliases)?;

        // ========================================
        // Module Loading Phase (all require-based)
        // ========================================

        // main runs before scene_dic finalization to allow dictionary registration.
        // SAFETY(injection): All module names below are compile-time string literals.
        require_startup_module(&runtime.lua, "main")?;

        require_startup_module(&runtime.lua, "pasta.shiori.entry")?;

        // Triggers SearchContext construction from Lua-side registries.
        require_startup_module(&runtime.lua, "pasta.scene_dic")?;

        // finalize（scene_dic のロード）の後なので、transpile 中の記録と実行時のシーン表が
        // 両方揃っている。`source_map` はデバッグ有効時だけ `Some`。突合の失敗は非致命で、
        // 索引なしで起動を続ける。
        if let Some(ref source_map) = runtime.source_map {
            let lua = &runtime.lua;
            let base_dir = &loader_context.base_dir;
            // 記録のキーは base_dir から見つけた `.pasta` のパスなので、相対パスを結び直す。
            let pasta_file_of = |src: &str| {
                super::searcher::pasta_file_of(lua, src.strip_prefix('@').unwrap_or(src))
                    .map(|rel| base_dir.join(rel).to_string_lossy().into_owned())
            };
            match crate::debug::source_map::build_scene_index(lua, source_map, pasta_file_of) {
                Ok(index) => {
                    // write-once。二重充填は本番経路では起こらない。
                    if source_map.set_scene_index(index).is_err() {
                        tracing::warn!(
                            "scene identity index already set; skipping re-population (task 2.2)"
                        );
                    } else {
                        tracing::debug!("Built and attached scene identity index (task 2.2)");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "scene identity index join failed; continuing without index (task 2.2, best-effort)");
                }
            }
        }

        Ok(runtime)
    }
}

/// 起動モジュールを `require` し、失敗を致命として文脈付きで伝搬する。
///
/// 失敗時は
/// 1. 構造化フィールド `module`（モジュール名）と `fatal`（致命なら `true`）を持つ
///    error ログへ、mlua の Display（複数行・スタックトレース込み）を欠落なく記録し、
/// 2. `failed to load startup module '<module>'` の文脈を付けた `Err` を返す。
///
/// この文脈は `LoaderError::Runtime` → `MyError::Load` を経て `X-ERROR-REASON` へ入るため、
/// 入れ子の `require` 失敗でも起動モジュール名と根本原因の両方が利用者へ届く。
fn require_startup_module(lua: &Lua, module: &'static str) -> LuaResult<()> {
    fatal_startup_module(module, lua_require(lua, module))?;
    tracing::debug!(module, "Loaded module via require");
    Ok(())
}

/// 起動モジュールのロード結果を致命として記録・文脈付けする共通処理。
///
/// 起動モジュールの失敗のログ構造と文脈の文言をそろえる単一の窓口。
fn fatal_startup_module<T>(module: &'static str, result: LuaResult<T>) -> LuaResult<T> {
    result
        .inspect_err(|e| {
            tracing::error!(
                module,
                fatal = true,
                error = %e,
                "Failed to load startup module"
            );
        })
        .context(format!("failed to load startup module '{module}'"))
}
