//! PastaLuaRuntime 公開 API の統合テスト（review-improvement-loop セル 3.16 / G1）
//!
//! 既存テストで未到達だった Rust 側 runtime API を検証する:
//! - `exec_file` — ファイル実行の成功・失敗パス（従来は定義のみで全テスト未到達）
//! - `exec_named` — チャンク名がエラーメッセージへ反映される契約
//! - `register_module` — カスタムモジュール登録と require 連携・上書き
//! - `with_config` — 不正ライブラリ名の ConfigError 伝播
//! - `load_scene_dic` — ファイル読込・チャンク名・欠損エラー
//! - `from_loader` — entry.lua の読込/失敗継続、transpiled コードの実行/エラー伝播
//! - `@pasta_config` — toml_to_lua の型変換（int/float/bool/datetime/array/nested）と
//!   actor name 注入（モジュール登録境界での検証。loader 統合テストの増分）

use crate::common::create_empty_context;
use pasta_lua::context::TranspileContext;
use pasta_lua::loader::{LoaderContext, TranspileResult};
use pasta_lua::{PastaLuaRuntime, RuntimeConfig};
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// exec_file
// ============================================================================

/// exec_file はファイル内の Lua スクリプトを実行し、戻り値を返す。
#[test]
fn test_exec_file_executes_script_and_returns_value() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("answer.lua");
    std::fs::write(&script_path, "return 6 * 7").unwrap();

    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    let value = runtime.exec_file(&script_path).unwrap();

    assert_eq!(value.as_i64(), Some(42));
}

/// exec_file は存在しないファイルでエラーを返す（panic しない）。
#[test]
fn test_exec_file_missing_file_returns_error() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("no_such_script.lua");

    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    let result = runtime.exec_file(&missing);

    assert!(result.is_err(), "missing file should produce an error");
}

/// exec_file はスクリプト内の実行時エラーを伝播する。
#[test]
fn test_exec_file_propagates_script_error() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("boom.lua");
    std::fs::write(&script_path, r#"error("file boom")"#).unwrap();

    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    let err = runtime.exec_file(&script_path).unwrap_err();

    assert!(
        err.to_string().contains("file boom"),
        "error message should contain the Lua error text: {err}"
    );
}

// ============================================================================
// exec_named
// ============================================================================

/// exec_named で設定したチャンク名がエラーメッセージに現れる
/// （デバッガのソース照合の前提となる契約）。
#[test]
fn test_exec_named_chunk_name_appears_in_error() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();

    // 実行時エラー（nil への算術）は「<チャンク名>:<行>:」プレフィックスを持つ
    let err = runtime
        .exec_named("local x = nil\nreturn x + 1", "cell316_chunk.lua")
        .unwrap_err();
    let msg = err.to_string();

    assert!(
        msg.contains("cell316_chunk"),
        "chunk name should appear in the error: {msg}"
    );
    assert!(
        msg.contains(":2:"),
        "line number of the failing statement should appear: {msg}"
    );
}

/// exec_named は exec と同様に評価結果を返す。
#[test]
fn test_exec_named_returns_value() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    let value = runtime.exec_named("return 1 + 2", "sum_chunk").unwrap();
    assert_eq!(value.as_i64(), Some(3));
}

// ============================================================================
// register_module
// ============================================================================

/// register_module で登録したモジュールが Lua 側から require できる。
#[test]
fn test_register_module_makes_module_requirable() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();

    let module = runtime.lua().create_table().unwrap();
    module.set("answer", 42).unwrap();
    runtime.register_module("@cell316_custom", module).unwrap();

    let value = runtime
        .exec(r#"return require("@cell316_custom").answer"#)
        .unwrap();
    assert_eq!(value.as_i64(), Some(42));
}

/// register_module の再登録は package.loaded を上書きする（last-wins 契約の文書化）。
#[test]
fn test_register_module_overwrites_existing_registration() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();

    let first = runtime.lua().create_table().unwrap();
    first.set("version", 1).unwrap();
    runtime.register_module("@cell316_overwrite", first).unwrap();

    let second = runtime.lua().create_table().unwrap();
    second.set("version", 2).unwrap();
    runtime
        .register_module("@cell316_overwrite", second)
        .unwrap();

    let value = runtime
        .exec(r#"return require("@cell316_overwrite").version"#)
        .unwrap();
    assert_eq!(value.as_i64(), Some(2), "later registration should win");
}

// ============================================================================
// with_config error propagation
// ============================================================================

/// 不正なライブラリ名は VM 構築時に ConfigError として伝播する。
#[test]
fn test_with_config_unknown_library_fails_with_name() {
    let config = RuntimeConfig::from_libs(vec!["std_bogus".into()]);
    let err = match PastaLuaRuntime::with_config(create_empty_context(), config) {
        Ok(_) => panic!("with_config should fail for unknown library"),
        Err(e) => e,
    };

    assert!(
        err.to_string().contains("std_bogus"),
        "error should name the unknown library: {err}"
    );
}

// ============================================================================
// runtime accessors (ad-hoc construction contract)
// ============================================================================

/// loader を経由しないランタイムでは config() は None。
#[test]
fn test_config_is_none_for_adhoc_runtime() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    assert!(runtime.config().is_none());
}

/// lua() アクセサは exec と同一の VM 状態を共有する。
#[test]
fn test_lua_accessor_shares_vm_state_with_exec() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    runtime.lua().globals().set("CELL316_SHARED", 7).unwrap();

    let value = runtime.exec("return CELL316_SHARED").unwrap();
    assert_eq!(value.as_i64(), Some(7));
}

// ============================================================================
// load_scene_dic
// ============================================================================

/// load_scene_dic は指定ファイルを実行する。
#[test]
fn test_load_scene_dic_executes_file() {
    let temp = TempDir::new().unwrap();
    let dic_path = temp.path().join("scene_dic.lua");
    std::fs::write(&dic_path, "SCENE_DIC_SENTINEL = 99").unwrap();

    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    runtime.load_scene_dic(&dic_path).unwrap();

    let value = runtime.exec("return SCENE_DIC_SENTINEL").unwrap();
    assert_eq!(value.as_i64(), Some(99));
}

/// load_scene_dic のチャンク名は "pasta.scene_dic" としてエラーに現れる。
#[test]
fn test_load_scene_dic_error_carries_chunk_name() {
    let temp = TempDir::new().unwrap();
    let dic_path = temp.path().join("scene_dic.lua");
    // 実行時エラー（nil への算術）で「<チャンク名>:<行>:」プレフィックスを観測する
    std::fs::write(&dic_path, "local x = nil\nreturn x + 1").unwrap();

    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    let err = runtime.load_scene_dic(&dic_path).unwrap_err();
    let msg = err.to_string();

    assert!(
        msg.contains(r#"[string "pasta.scene_dic"]:2:"#),
        "chunk name + line prefix should appear as [string \"pasta.scene_dic\"]:2: — {msg}"
    );
}

/// load_scene_dic はファイル欠損でエラーを返す。
#[test]
fn test_load_scene_dic_missing_file_returns_error() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    let result = runtime.load_scene_dic(Path::new("definitely/missing/scene_dic.lua"));
    assert!(result.is_err());
}

// ============================================================================
// from_loader — entry.lua handling
// ============================================================================

/// base_dir/scripts/pasta/shiori/entry.lua が存在する場合、from_loader が実行する。
#[test]
fn test_from_loader_loads_entry_lua_when_present() {
    let temp = TempDir::new().unwrap();
    let entry_dir = temp.path().join("scripts/pasta/shiori");
    std::fs::create_dir_all(&entry_dir).unwrap();
    std::fs::write(entry_dir.join("entry.lua"), "CELL316_ENTRY_LOADED = true").unwrap();

    let loader_context = LoaderContext::new(
        temp.path(),
        vec!["scripts".to_string()],
        toml::Table::new(),
    );
    let runtime = PastaLuaRuntime::from_loader(
        TranspileContext::new(),
        loader_context,
        RuntimeConfig::new(),
        &[],
        None,
    )
    .unwrap();

    let value = runtime.exec("return CELL316_ENTRY_LOADED == true").unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

/// entry.lua が実行時エラーでも from_loader は警告のみで継続する（graceful degradation）。
#[test]
fn test_from_loader_continues_when_entry_lua_fails() {
    let temp = TempDir::new().unwrap();
    let entry_dir = temp.path().join("scripts/pasta/shiori");
    std::fs::create_dir_all(&entry_dir).unwrap();
    std::fs::write(entry_dir.join("entry.lua"), r#"error("entry broken")"#).unwrap();

    let loader_context = LoaderContext::new(
        temp.path(),
        vec!["scripts".to_string()],
        toml::Table::new(),
    );
    let runtime = PastaLuaRuntime::from_loader(
        TranspileContext::new(),
        loader_context,
        RuntimeConfig::new(),
        &[],
        None,
    );

    assert!(
        runtime.is_ok(),
        "from_loader must continue despite entry.lua failure"
    );
}

// ============================================================================
// from_loader — transpiled module loading
// ============================================================================

/// transpiled に渡したコードがロード時に実行される。
#[test]
fn test_from_loader_executes_transpiled_modules() {
    let transpiled = vec![
        TranspileResult {
            module_name: "cell316_mod_a".to_string(),
            lua_code: "CELL316_TRANSPILED_A = 7".to_string(),
            source_path: std::path::PathBuf::from("a.pasta"),
        },
        TranspileResult {
            module_name: "cell316_mod_b".to_string(),
            lua_code: "CELL316_TRANSPILED_B = CELL316_TRANSPILED_A + 1".to_string(),
            source_path: std::path::PathBuf::from("b.pasta"),
        },
    ];

    let loader_context = LoaderContext::new(
        "/test/path",
        vec!["scripts".to_string()],
        toml::Table::new(),
    );
    let runtime = PastaLuaRuntime::from_loader(
        TranspileContext::new(),
        loader_context,
        RuntimeConfig::new(),
        &transpiled,
        None,
    )
    .unwrap();

    // 2 モジュールが宣言順に実行されたことを観測
    let value = runtime.exec("return CELL316_TRANSPILED_B").unwrap();
    assert_eq!(value.as_i64(), Some(8));
}

/// transpiled コードのエラーは from_loader から伝播し、モジュール名がチャンク名になる。
#[test]
fn test_from_loader_transpiled_error_propagates_with_module_name() {
    // 実行時エラー（nil への算術）でモジュール名がチャンク名として現れることを観測する
    let transpiled = vec![TranspileResult {
        module_name: "cell316_mod_err".to_string(),
        lua_code: "local x = nil\nreturn x + 1".to_string(),
        source_path: std::path::PathBuf::from("err.pasta"),
    }];

    let loader_context = LoaderContext::new(
        "/test/path",
        vec!["scripts".to_string()],
        toml::Table::new(),
    );
    let err = match PastaLuaRuntime::from_loader(
        TranspileContext::new(),
        loader_context,
        RuntimeConfig::new(),
        &transpiled,
        None,
    ) {
        Ok(_) => panic!("from_loader should fail when transpiled code errors"),
        Err(e) => e,
    };
    let msg = err.to_string();

    assert!(
        msg.contains("cell316_mod_err"),
        "module name should appear as chunk name: {msg}"
    );
}

// ============================================================================
// @pasta_config — toml_to_lua conversions & actor name injection
// ============================================================================

fn runtime_with_custom_fields(custom_fields: toml::Table) -> PastaLuaRuntime {
    let loader_context = LoaderContext::new(
        "/test/path",
        vec!["scripts".to_string()],
        custom_fields,
    );
    PastaLuaRuntime::from_loader(
        TranspileContext::new(),
        loader_context,
        RuntimeConfig::new(),
        &[],
        None,
    )
    .unwrap()
}

/// TOML の各値型（integer/float/bool/datetime/array/nested table）が
/// @pasta_config 上で期待どおりの Lua 値に変換される。
#[test]
fn test_pasta_config_toml_value_conversions() {
    let custom_fields: toml::Table = toml::from_str(
        r#"
        int_val = 42
        float_val = 1.5
        bool_val = true
        date_val = 2026-06-11T00:00:00Z
        arr = ["a", "b", "c"]

        [nested]
        inner = "deep"
        "#,
    )
    .unwrap();

    let runtime = runtime_with_custom_fields(custom_fields);
    let value = runtime
        .exec(
            r#"
            local config = require "@pasta_config"
            assert(config.int_val == 42, "int_val")
            assert(config.float_val == 1.5, "float_val")
            assert(config.bool_val == true, "bool_val")
            assert(type(config.date_val) == "string", "date_val type")
            assert(config.date_val:find("2026%-06%-11") ~= nil, "date_val content")
            assert(#config.arr == 3, "arr length")
            assert(config.arr[1] == "a" and config.arr[3] == "c", "arr 1-based order")
            assert(config.nested.inner == "deep", "nested table")
            return true
            "#,
        )
        .unwrap();

    assert!(value.as_boolean().unwrap_or(false));
}

/// [actor.<キー>] サブテーブルへ name = キー名 が注入され、既存 name は上書きされる。
#[test]
fn test_pasta_config_actor_name_injection() {
    let custom_fields: toml::Table = toml::from_str(
        r#"
        [actor."さくら"]
        spot = 0

        [actor."うにゅう"]
        spot = 1
        name = "should_be_overwritten"
        "#,
    )
    .unwrap();

    let runtime = runtime_with_custom_fields(custom_fields);
    let value = runtime
        .exec(
            r#"
            local config = require "@pasta_config"
            assert(config.actor["さくら"].name == "さくら", "injected name")
            assert(config.actor["うにゅう"].name == "うにゅう", "TOML key overwrites existing name")
            return true
            "#,
        )
        .unwrap();

    assert!(value.as_boolean().unwrap_or(false));
}

/// actor がテーブル以外の場合、name 注入は no-op で値はそのまま公開される。
#[test]
fn test_pasta_config_non_table_actor_is_noop() {
    let custom_fields: toml::Table = toml::from_str(r#"actor = "not_a_table""#).unwrap();

    let runtime = runtime_with_custom_fields(custom_fields);
    let value = runtime
        .exec(
            r#"
            local config = require "@pasta_config"
            return config.actor == "not_a_table"
            "#,
        )
        .unwrap();

    assert!(value.as_boolean().unwrap_or(false));
}

// ============================================================================
// @pasta_log ハードニング回帰（review-improvement-loop セル 3.17 / G3）
// ============================================================================

/// @pasta_log は極端に深い入れ子テーブル（Lua 入力で自由に構築可能）でも
/// ホストの Rust スタックを溢れさせない（abort は SHIORI ホストごと落とす — R3.7）。
/// 深さ上限（MAX_NESTING_DEPTH）を超えるテーブルは JSON 変換へ入る前に
/// tostring() フォールバックへ退避することを、プロセス生存と正常復帰で確認する。
#[test]
fn test_log_deeply_nested_table_does_not_overflow_host_stack() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();

    let value = runtime
        .exec(
            r#"
            local log = require "@pasta_log"
            local t = {}
            for _ = 1, 200000 do t = { t } end
            log.info(t)
            return true
            "#,
        )
        .unwrap();

    assert!(value.as_boolean().unwrap_or(false));
}

/// 深さゲートの境界ペア（その1）: ちょうど深さ上限（MAX_NESTING_DEPTH = 10）の
/// テーブルは JSON 変換され、リーフの値がログへそのまま現れる
/// （review-improvement-loop セル 3.18 / 3.17 申し送り — 閾値ドリフト防止）。
#[tracing_test::traced_test]
#[test]
fn test_log_table_at_depth_limit_is_json_converted() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();

    // ルートを深さ1として 9 回ラップ → 総深さ 10 = ちょうど上限（超過しない）
    runtime
        .exec(
            r#"
            local log = require "@pasta_log"
            local t = { leaf = "d10_json_marker" }
            for _ = 1, 9 do t = { t } end
            log.info(t)
            "#,
        )
        .unwrap();

    assert!(
        logs_contain("d10_json_marker"),
        "depth-10 table should be JSON-converted, exposing the leaf value"
    );
}

/// 深さゲートの境界ペア（その2）: 上限を 1 だけ超えた深さ 11 のテーブルは
/// JSON 変換せず tostring() フォールバック（"table: 0x..."）へ退避し、
/// リーフの値はログへ現れない（境界ペアその1との対で閾値を固定）。
#[tracing_test::traced_test]
#[test]
fn test_log_table_one_past_depth_limit_falls_back_to_tostring() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();

    // ルートを深さ1として 10 回ラップ → 総深さ 11 = 上限 10 を超過
    runtime
        .exec(
            r#"
            local log = require "@pasta_log"
            local t = { leaf = "d11_fallback_marker" }
            for _ = 1, 10 do t = { t } end
            log.info(t)
            "#,
        )
        .unwrap();

    assert!(
        logs_contain("table: "),
        "depth-11 table should fall back to Lua tostring()"
    );
    assert!(
        !logs_contain("d11_fallback_marker"),
        "leaf value must not leak through JSON conversion past the depth limit"
    );
}

/// 自己参照（循環）テーブルも log 呼び出しを失敗させない
/// （深さゲートが循環で必ず発火し tostring() フォールバックへ退避する境界の固定）。
#[test]
fn test_log_cyclic_table_falls_back_without_error() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();

    let value = runtime
        .exec(
            r#"
            local log = require "@pasta_log"
            local t = {}
            t.self = t
            log.warn(t)
            return true
            "#,
        )
        .unwrap();

    assert!(value.as_boolean().unwrap_or(false));
}
