//! PastaLuaRuntime 公開 API の統合テスト（review-improvement-loop セル 3.16 / G1）
//!
//! 既存テストで未到達だった Rust 側 runtime API を検証する:
//! - `exec_file` — ファイル実行の成功・失敗パス（従来は定義のみで全テスト未到達）
//! - `exec_named` — チャンク名がエラーメッセージへ反映される契約
//! - `register_module` — カスタムモジュール登録と require 連携・上書き
//! - `with_config` — 不正ライブラリ名の ConfigError 伝播
//! - `PastaLoader::load` — 変換した辞書の実行・辞書の実行時のエラーの文脈
//! - `@pasta_config` — toml_to_lua の型変換（int/float/bool/datetime/array/nested）と
//!   actor name 注入（モジュール登録境界での検証。loader 統合テストの増分）

use crate::common::{copy_fixture_to_temp, create_empty_context, create_temp_with_pasta};
use pasta_lua::loader::PastaLoader;
use pasta_lua::{ConfigError, PastaLuaRuntime, RuntimeConfig};
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

/// 同梱の LuaJIT（mlua 0.12 から）は `continue` 文を構文の誤りにせず、飛ばした回を数えない。
/// マニュアルの book/src/lua/basics.md の「前提」に書いた挙動の差を固定する（LuaJIT が変えたらここが落ちる）。
#[test]
fn test_luajit_accepts_continue_statement() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();
    let value = runtime
        .exec_named(
            "local n = 0\nfor i = 1, 3 do\n  if i == 2 then continue end\n  n = n + i\nend\nreturn n",
            "continue_chunk",
        )
        .unwrap();
    assert_eq!(value.as_i64(), Some(1 + 3));
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
    runtime
        .register_module("@cell316_overwrite", first)
        .unwrap();

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

/// `std_package` を欠いた構成は、VM を作らずに名前付きの構成エラーになる（3.1、7.2）。
#[test]
fn test_with_config_missing_std_package_fails_with_name() {
    for libs in [vec!["std_string"], vec!["std_all", "-std_package"]] {
        let config = RuntimeConfig::from_libs(libs.iter().map(|s| s.to_string()).collect());
        let err = match PastaLuaRuntime::with_config(create_empty_context(), config) {
            Ok(_) => panic!("with_config should fail without std_package: {libs:?}"),
            Err(e) => e,
        };
        let config_err = match &err {
            pasta_lua::mlua::Error::ExternalError(e) => e.downcast_ref::<ConfigError>(),
            _ => None,
        };
        assert!(
            matches!(config_err, Some(ConfigError::MissingRequiredLibrary(name)) if name == "std_package"),
            "{libs:?}: expected MissingRequiredLibrary(std_package), got {err:?}"
        );
        assert!(err.to_string().contains("std_package"), "{libs:?}: {err}");
    }
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

/// VM ごとに乱数の種が与えられ、2 つのランタイムの math.random の列が一致しない。
#[test]
fn test_math_random_sequence_differs_between_runtimes() {
    let script = "local t = {} for i = 1, 8 do t[i] = math.random(1, 1000000) end \
                  return table.concat(t, ',')";
    let a = PastaLuaRuntime::new(create_empty_context()).unwrap();
    let b = PastaLuaRuntime::new(create_empty_context()).unwrap();
    let seq_a = a.exec(script).unwrap().to_string().unwrap();
    let seq_b = b.exec(script).unwrap().to_string().unwrap();
    assert_ne!(seq_a, seq_b, "math.random sequences must differ per VM");
}

/// math を読み込まない構成でも VM を作れる（種の設定は math がある時だけ）。
#[test]
fn test_runtime_without_math_library_still_builds() {
    let config = RuntimeConfig::from_libs(vec!["std_all".into(), "-std_math".into()]);
    let runtime = PastaLuaRuntime::with_config(create_empty_context(), config).unwrap();
    assert!(runtime.exec("return math").unwrap().is_nil());
}

/// mlua-stdlib の 6 つのモジュールは、既定の構成でも 6 つの名前を並べた構成でも入らない（1.1・1.5）。
#[test]
fn test_mlua_stdlib_modules_are_not_requirable() {
    const NAMES: [&str; 6] = ["assertions", "testing", "env", "regex", "json", "yaml"];
    let mut libs = vec!["std_all".to_string()];
    libs.extend(NAMES.iter().map(|s| s.to_string()));

    for (label, config) in [
        ("new", RuntimeConfig::new()),
        ("from_libs", RuntimeConfig::from_libs(libs)),
    ] {
        let runtime = PastaLuaRuntime::with_config(create_empty_context(), config).unwrap();
        for name in NAMES {
            let ok = runtime
                .exec(&format!("return (pcall(require, '@{name}'))"))
                .unwrap();
            assert_eq!(
                ok.as_boolean(),
                Some(false),
                "{label}: @{name} must not load"
            );
        }
    }
}
// ============================================================================
// 本番の読み込み — 変換した辞書の実行
// ============================================================================

/// `dic/*.pasta` の Lua のブロックは、読み込みのときに変換したモジュールとして実行される。
#[test]
fn test_load_executes_transpiled_modules() {
    let temp = create_temp_with_pasta(
        "＊テスト\n  ゴースト：「こんにちは」\n```lua\nCELL316_TRANSPILED_A = 7\n```\n",
    );
    let runtime = PastaLoader::load(temp.path()).unwrap();

    let value = runtime.exec("return CELL316_TRANSPILED_A").unwrap();
    assert_eq!(value.as_i64(), Some(7));
}

/// 辞書の実行時のエラーは読み込みエラーになり、モジュール名（チャンク名）と
/// 起動モジュール `pasta.scene_dic` の文脈を持つ。
#[test]
fn test_load_transpiled_error_propagates_with_module_name() {
    // 実行時エラー（nil への算術）でモジュール名がチャンク名として現れることを観測する
    let temp = create_temp_with_pasta(
        "＊テスト\n  ゴースト：「こんにちは」\n```lua\nlocal x = nil\nCELL316_ERR = x + 1\n```\n",
    );
    let msg = match PastaLoader::load(temp.path()) {
        Ok(_) => panic!("load should fail when transpiled code errors"),
        Err(e) => e.to_string(),
    };

    assert!(
        msg.contains("failed to load startup module 'pasta.scene_dic'"),
        "startup module context should appear: {msg}"
    );
    // モジュール `pasta.scene.test.hello` のチャンク名はキャッシュのファイルのパス（Windows では区切りが混ざる）
    assert!(
        msg.replace('\\', "/")
            .contains("pasta/scene/test/hello.lua:"),
        "module name should appear as chunk name: {msg}"
    );
    assert!(
        msg.contains("attempt to perform arithmetic"),
        "root cause should be preserved: {msg}"
    );
}

// ============================================================================
// @pasta_config — toml_to_lua conversions & actor name injection
// ============================================================================

/// `pasta.toml` に独自の表を書いた一時ゴーストを本番の読み込みに通す。
fn runtime_with_custom_fields(custom_toml: &str) -> (TempDir, PastaLuaRuntime) {
    let temp = copy_fixture_to_temp("minimal");
    std::fs::write(
        temp.path().join("pasta.toml"),
        format!("{custom_toml}\n[loader]\ndebug_mode = true\n"),
    )
    .unwrap();
    let runtime = PastaLoader::load(temp.path()).unwrap();
    (temp, runtime)
}
/// TOML の各値型（integer/float/bool/datetime/array/nested table）が
/// @pasta_config 上で期待どおりの Lua 値に変換される。
#[test]
fn test_pasta_config_toml_value_conversions() {
    let (_temp, runtime) = runtime_with_custom_fields(
        r#"
        int_val = 42
        float_val = 1.5
        bool_val = true
        date_val = 2026-06-11T00:00:00Z
        arr = ["a", "b", "c"]

        [nested]
        inner = "deep"
        "#,
    );
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
    let (_temp, runtime) = runtime_with_custom_fields(
        r#"
        [actor."さくら"]
        spot = 0

        [actor."うにゅう"]
        spot = 1
        name = "should_be_overwritten"
        "#,
    );
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
    let (_temp, runtime) = runtime_with_custom_fields(r#"actor = "not_a_table""#);
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
/// 深さ上限（MAX_NESTING_DEPTH）を超えるテーブルは共有の厳しい変換（深さ 10）が
/// Err を返し tostring() フォールバックへ退避することを、プロセス生存と正常復帰で確認する。
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

/// 深さ上限の境界ペア（その1）: ちょうど深さ上限（MAX_NESTING_DEPTH = 10）の
/// テーブルは TOML の 1 行に書かれ、リーフの値がログへそのまま現れる
/// （review-improvement-loop セル 3.18 / 3.17 申し送り — 閾値ドリフト防止）。
#[tracing_test::traced_test]
#[test]
fn test_log_table_at_depth_limit_is_toml_inline() {
    let runtime = PastaLuaRuntime::new(create_empty_context()).unwrap();

    // ルートを深さ1として 9 回ラップ → 総深さ 10 = ちょうど上限（超過しない）
    runtime
        .exec(
            r#"
            local log = require "@pasta_log"
            local t = { leaf = "d10_marker" }
            for _ = 1, 9 do t = { t } end
            log.info(t)
            "#,
        )
        .unwrap();

    assert!(
        logs_contain("d10_marker"),
        "depth-10 table should be written as one TOML line, exposing the leaf value"
    );
}

/// 深さ上限の境界ペア（その2）: 上限を 1 だけ超えた深さ 11 のテーブルは
/// 共有の厳しい変換（深さ 10）が Err を返し tostring() フォールバック（"table: 0x..."）へ退避し、
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
        "leaf value must not leak into the TOML line past the depth limit"
    );
}

/// 自己参照（循環）テーブルも log 呼び出しを失敗させない
/// （循環では共有の厳しい変換（深さ 10）が必ず Err を返し tostring() フォールバックへ退避する境界の固定）。
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
