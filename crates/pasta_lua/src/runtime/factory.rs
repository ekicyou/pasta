//! Factory constructors for PastaLuaRuntime.
//!
//! This file contains the `impl PastaLuaRuntime` factory methods used by
//! PastaLoader to create a runtime from a LoaderContext (split impl pattern).

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
    /// 4. require("main") - 利用者初期化スクリプト。失敗は**致命**（要件 5.2）
    /// 5. require("pasta.shiori.entry") - SHIORI 応答関数。失敗は**致命**（要件 4.1）
    /// 6. require("pasta.scene_dic") - シーン読み込みと finalize。失敗は**致命**（要件 5.1）
    /// 7. シーン identity 索引の突合 - モジュールロードではない best-effort 処理。
    ///    失敗しても起動は継続する（デバッグ有効時のみ実行）
    ///
    /// 起動モジュール（4〜6）はすべて `require_startup_module` を経由し、失敗時は
    /// `module` / `fatal` フィールド付きの error ログと、起動モジュール名の文脈を
    /// 付けた `Err` で可視化される（要件 4.5 / 4.7 / 5.3 / 5.5）。
    ///
    /// finalize 前の `@pasta_search` には常に空の登録表を渡す（キャッシュの状態によらず
    /// 見つからない。3.1〜3.3）。finalize が Lua 側の登録から作り直す。
    ///
    /// # Arguments
    /// * `loader_context` - Configuration and paths from PastaLoader
    /// * `runtime_config` - Runtime configuration
    /// * `pasta_config` - Pasta configuration from pasta.toml
    /// * `logger` - Optional instance-specific logger (Arc-wrapped for sharing)
    /// * `source_map` - Optional aggregated `.pasta`↔`.lua` source map built by the
    ///   loader AFTER transpile, ONLY when debugging is enabled (task 4.3/4.4).
    ///   `None` on the disabled (default) zero-cost path, where nothing is built or
    ///   held (requirements 3.1 / 7.1).
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
        // Derive the debug gate BEFORE building the VM (task 4.2/4.4): funnel the
        // pasta.toml `[debug]` section (`PastaConfig::debug()` — now carrying
        // `present_as`/`source_map_sidecar`, task 4.4) plus the `PASTA_DEBUG*`
        // environment through the single pure `DebugConfig::resolve` choke point and
        // attach it to `runtime_config`. `with_config_and_source_map` then performs
        // the SINGLE `debug::enable` at VM init, HANDING the aggregated source map
        // (built by the loader after transpile) to it (requirement 3.1 ordering:
        // transpile → build map → enable-with-map). When there is no `[debug]`
        // section (and no env override) this resolves to the disabled, zero-cost
        // default and `source_map` is `None` (R5.2 / R5.5 / 7.1).
        let debug_file = pasta_config.as_ref().and_then(|c| c.debug());
        let runtime_config = runtime_config.with_debug_from_file_and_env(debug_file.as_ref());
        // finalize_scene must re-register @pasta_search with the SAME alias table
        // as the initial registration inside `with_config_and_source_map`.
        let scene_aliases = runtime_config.scene_aliases.clone();
        // `@pasta_debug.enabled()` は待ち受けの有無ではなく解決した設定の値を見る（4.2）。
        let debug_enabled = runtime_config.debug.enabled;

        // Create base runtime (installs the debug hook exactly once iff enabled,
        // and holds + forwards the source map to enable when present).
        let mut runtime =
            Self::with_config_and_source_map(TranspileContext::new(), runtime_config, source_map)?;

        // Set logger if provided
        runtime.logger = logger;

        // Store config and base_dir for Drop-time persistence save
        runtime.base_dir = Some(loader_context.base_dir.clone());
        runtime.config = pasta_config;

        // Setup package.path for module resolution
        Self::setup_package_path(&runtime.lua, &loader_context)?;

        // Register @pasta_config module
        Self::register_config_module(&runtime.lua, &loader_context.custom_fields)?;

        // Register @enc module for encoding conversion
        Self::register_enc_module(&runtime.lua)?;

        // Register @pasta_persistence module for persistent data storage
        Self::register_persistence_module(&runtime.lua, &runtime.config, &runtime.base_dir)?;

        // Register @pasta_log module for Lua logging bridge
        Self::register_log_module(&runtime.lua)?;

        // Register @pasta_sakura_script module
        Self::register_sakura_script_module(&runtime.lua, &runtime.config)?;

        // Register internal @pasta_debug module (production entry only; scene-play-event 2.3)
        Self::register_debug_module(
            &runtime.lua,
            debug_enabled,
            runtime.source_map.clone(),
            &loader_context.base_dir,
        )?;

        // Register finalize_scene Rust binding to overwrite Lua stub (Requirement 4.3)
        // This must be done before loading scene_dic.lua which calls finalize_scene()
        register_finalize_scene(&runtime.lua, scene_aliases)?;

        // ========================================
        // Module Loading Phase (all require-based)
        // ========================================

        // Step 4: require("main") - User initialization script
        // Runs before scene_dic finalization to allow dictionary registration
        // SAFETY(injection): All module names below are compile-time string literals.
        // 起動モジュール 3 種はいずれも致命であり、失敗は `?` で伝搬する。
        require_startup_module(&runtime.lua, "main")?;

        // Step 5: require("pasta.shiori.entry") - SHIORI handlers
        require_startup_module(&runtime.lua, "pasta.shiori.entry")?;

        // Step 6: require("pasta.scene_dic") - Scene loading and finalization
        // This triggers SearchContext construction from Lua-side registries
        require_startup_module(&runtime.lua, "pasta.scene_dic")?;

        // Step 7 (pasta-scene-kick task 2.2): finalize シーン突合。
        //
        // この時点で BOTH が揃う: (a) loader が transpile 中に蓄積した
        // `(join_key, .pasta 宣言行)` 記録（`runtime.source_map` 内）、(b) finalize
        // （scene_dic ロードで実行済み）が登録した runtime 実シーン identity
        // （シーン表の登録名を定義元 `.pasta` ファイルごとに分けたもの）。両者をファイルごとの順位で突き合わせ、`.pasta` (ファイル,
        // 行範囲) → (scene_id, parent) の `SceneIdentityIndex` を確定し、共有 `Arc<
        // SourceMap>` の write-once `scene_index` スロットへ充填する（resolver=task 3.1
        // が読む）。
        //
        // ゲート: `source_map` は debug 有効時のみ `Some`（loader Phase 5.5）。無効
        // （既定）では `None` で本ブロックは丸ごとスキップ＝索引非構築・行マッピング
        // 不変（requirements 7.1 通常モード非破壊）。突合失敗（記録空・identity 不一致）
        // は非致命（best-effort）: 空/部分索引のまま続行し、起動を妨げない。
        if let Some(ref source_map) = runtime.source_map {
            match crate::debug::source_map::build_scene_index(&runtime.lua, source_map) {
                Ok(index) => {
                    // write-once: 初回のみ成功。二重充填（Err）は本番経路では起こらない。
                    if source_map.set_scene_index(index).is_err() {
                        tracing::warn!(
                            "scene identity index already set; skipping re-population (task 2.2)"
                        );
                    } else {
                        tracing::debug!("Built and attached scene identity index (task 2.2)");
                    }
                }
                Err(e) => {
                    // 突合自体の失敗は非致命: 索引なしでも通常実行・既存 kick は不変。
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
///    error ログへ、mlua の Display（複数行・スタックトレース込み）を欠落なく記録し
///    （要件 4.7 / 5.3）、
/// 2. `failed to load startup module '<module>'` の文脈を付けた `Err` を返す。
///
/// この文脈は `LoaderError::Runtime` → `MyError::Load` を経て `X-ERROR-REASON` へ入るため、
/// 入れ子の `require` 失敗でも起動モジュール名と根本原因の両方が利用者へ届く（要件 4.5）。
///
/// 継続可能な起動モジュールを将来追加する場合は、同じフィールド構成のまま
/// `fatal = false`・レベル warn で記録する。
fn require_startup_module(lua: &Lua, module: &'static str) -> LuaResult<()> {
    fatal_startup_module(module, lua_require(lua, module))?;
    tracing::debug!(module, "Loaded module via require");
    Ok(())
}

/// 起動モジュールのロード結果を致命として記録・文脈付けする共通処理。
///
/// 起動モジュールの失敗のログ構造と文脈の文言をそろえる単一の窓口（要件 5.3 / 5.5）。
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
