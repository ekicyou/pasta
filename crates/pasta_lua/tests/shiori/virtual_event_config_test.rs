//! Integration tests for pasta.shiori.event.virtual_dispatcher module.
//!
//! Tests for configuration, state management, and second_change integration.

use crate::common;

use common::create_runtime_with_pasta_path;

// ============================================================================
// Task 4.5: Config and Status Tests
// ============================================================================

#[test]
fn test_config_default_values() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        -- _get_config() を直接呼び出してデフォルト値を検証
        local cfg = dispatcher._get_config()

        -- Default values: min=180, max=300, margin=30
        return cfg ~= nil
           and cfg.talk_interval_min == 180
           and cfg.talk_interval_max == 300
           and cfg.hour_margin == 30
    "#,
    );

    assert!(
        result.is_ok(),
        "Config should have default values: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_skip_when_talking() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()
        
        -- Set up mock scene executor (should never be called when "talking")
        local scene_results = {
            OnHour = "hour",
            OnTalk = "talk"
        }
        dispatcher._set_scene_executor(function(event_name)
            return scene_results[event_name]
        end)
        
        -- Initialize
        local act1 = { req = {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 }
        } }
        dispatcher.dispatch(act1)
        
        -- Call at next hour with "talking" status - should skip
        local act2 = { req = {
            id = "OnSecondChange",
            status = "talking",  -- Currently talking
            date = { unix = 1702652400 }  -- Next hour
        } }
        local hour_result = dispatcher.check_hour(act2)
        local talk_result = dispatcher.check_talk(act2)
        
        return hour_result == nil and talk_result == nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Should skip when status is 'talking': {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 4.6: State Management Tests
// ============================================================================

#[test]
fn test_module_state_reset() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        
        -- Set some state
        local act = { req = {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 }
        } }
        dispatcher.dispatch(act)
        
        local state_before = dispatcher._get_internal_state()
        local had_state = state_before.next_hour_unix > 0

        -- Reset
        dispatcher._reset()

        local state_after = dispatcher._get_internal_state()
        -- リセット後は _get_config() がデフォルト値を返すことを検証
        local cfg_after_reset = dispatcher._get_config()
        local is_reset = state_after.next_hour_unix == 0
                     and state_after.next_talk_time == 0
                     and cfg_after_reset.talk_interval_min == 180
                     and cfg_after_reset.talk_interval_max == 300

        return had_state and is_reset
    "#,
    );

    assert!(result.is_ok(), "State should reset properly: {:?}", result);
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_internal_state_getter() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()
        
        local state = dispatcher._get_internal_state()

        return type(state) == "table"
           and state.next_hour_unix == 0
           and state.next_talk_time == 0
    "#,
    );

    assert!(
        result.is_ok(),
        "Internal state getter should work: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// second_change module integration
// ============================================================================

#[test]
fn test_second_change_module_loads() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local second_change = require "pasta.shiori.event.second_change"
        return second_change ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "second_change module should load: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_onsecondchange_handler_registered() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        require "pasta.shiori.event.second_change"
        
        return type(REG.OnSecondChange) == "function"
    "#,
    );

    assert!(
        result.is_ok(),
        "OnSecondChange handler should be registered: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 3.1: SAVE優先・実行時変更・部分設定テスト
// ============================================================================

#[test]
fn test_save_priority_over_toml_and_default() {
    let runtime = create_runtime_with_pasta_path();

    // tomlモックを設定（SAVE値とは異なる値）
    runtime
        .exec(
            r#"
        package.loaded["@pasta_config"] = {
            ghost = { talk_interval_min = 90, talk_interval_max = 150 }
        }
    "#,
        )
        .expect("Failed to set inline mock for @pasta_config");

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        -- SAVEテーブルに値を設定（tomlとは異なる値）
        local save = require "pasta.save"
        save.pasta_talk_interval_min = 60
        save.pasta_talk_interval_max = 120

        -- _get_config() がSAVE値（toml値ではなく）を返すことを検証
        local cfg = dispatcher._get_config()
        return cfg.talk_interval_min == 60
           and cfg.talk_interval_max == 120
    "#,
    );

    assert!(
        result.is_ok(),
        "SAVE values should take priority over toml and default: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_runtime_change_reflected_immediately() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        local save = require "pasta.save"
        local act = { req = {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 }
        } }

        -- 初回dispatch（next_talk_time設定）
        dispatcher.dispatch(act)

        -- SAVE値を初期設定
        save.pasta_talk_interval_min = 60
        save.pasta_talk_interval_max = 120
        local cfg1 = dispatcher._get_config()

        -- SAVE値を変更
        save.pasta_talk_interval_min = 30
        save.pasta_talk_interval_max = 60
        local cfg2 = dispatcher._get_config()

        -- キャッシュ廃止により変更が即時反映されること
        return cfg1.talk_interval_min == 60
           and cfg2.talk_interval_min == 30
           and cfg2.talk_interval_max == 60
    "#,
    );

    assert!(
        result.is_ok(),
        "Runtime changes should be reflected immediately: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_partial_save_configuration() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        -- minのみSAVEに設定、maxは未設定
        local save = require "pasta.save"
        save.pasta_talk_interval_min = 50

        local cfg = dispatcher._get_config()
        -- min はSAVE値、max はデフォルト(300)
        return cfg.talk_interval_min == 50
           and cfg.talk_interval_max == 300
    "#,
    );

    assert!(
        result.is_ok(),
        "Partial save config: min from SAVE, max from default: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_partial_save_configuration_max_only() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        -- maxのみSAVEに設定、minは未設定
        local save = require "pasta.save"
        save.pasta_talk_interval_max = 600

        local cfg = dispatcher._get_config()
        -- min はデフォルト(180)、max はSAVE値
        return cfg.talk_interval_min == 180
           and cfg.talk_interval_max == 600
    "#,
    );

    assert!(
        result.is_ok(),
        "Partial save config: max from SAVE, min from default: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 3.2: tomlフォールバック・ハードコードデフォルトテスト
// ============================================================================

#[test]
fn test_toml_fallback_values() {
    let runtime = create_runtime_with_pasta_path();

    // 案B: インラインモックで @pasta_config を差し込む
    runtime
        .exec(
            r#"
        package.loaded["@pasta_config"] = {
            ghost = { talk_interval_min = 60, talk_interval_max = 90 }
        }
    "#,
        )
        .expect("Failed to set inline mock for @pasta_config");

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        -- SAVE未設定、tomlモック有り → toml値が使用される
        local cfg = dispatcher._get_config()
        return cfg.talk_interval_min == 60
           and cfg.talk_interval_max == 90
    "#,
    );

    assert!(
        result.is_ok(),
        "toml values should be used as fallback when SAVE is empty: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_hardcoded_default_values() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        -- SAVE・tomlともに未設定 → ハードコードデフォルト(180/300)
        local cfg = dispatcher._get_config()
        return cfg.talk_interval_min == 180
           and cfg.talk_interval_max == 300
    "#,
    );

    assert!(
        result.is_ok(),
        "Hardcoded defaults should be used when SAVE and toml are both unset: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 3.3: バリデーション（非数値・min>max補正）テスト
// ============================================================================

#[test]
fn test_non_numeric_save_values_fallback() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        -- SAVEに文字列値を設定 → 無視されてデフォルト値にフォールバック
        local save = require "pasta.save"
        save.pasta_talk_interval_min = "fast"
        save.pasta_talk_interval_max = "slow"

        local cfg = dispatcher._get_config()
        return cfg.talk_interval_min == 180
           and cfg.talk_interval_max == 300
    "#,
    );

    assert!(
        result.is_ok(),
        "Non-numeric SAVE values should be ignored and fall back to defaults: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_min_greater_than_max_correction() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        -- min > max のケース → max = min に補正
        local save = require "pasta.save"
        save.pasta_talk_interval_min = 500
        save.pasta_talk_interval_max = 100

        local cfg = dispatcher._get_config()
        return cfg.talk_interval_min == 500
           and cfg.talk_interval_max == 500
    "#,
    );

    assert!(
        result.is_ok(),
        "When min > max, max should be corrected to equal min: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// 検証レポート P1: 値バリデーション強化テスト
// ============================================================================

#[test]
fn test_values_below_minimum_clamped_to_10() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        local save = require "pasta.save"
        save.pasta_talk_interval_min = 3
        save.pasta_talk_interval_max = 5

        local cfg = dispatcher._get_config()
        -- 10未満の値は10にクランプされる
        return cfg.talk_interval_min == 10
           and cfg.talk_interval_max == 10
    "#,
    );

    assert!(
        result.is_ok(),
        "Values below 10 should be clamped to 10: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_float_values_are_floored() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        local save = require "pasta.save"
        save.pasta_talk_interval_min = 45.7
        save.pasta_talk_interval_max = 120.9

        local cfg = dispatcher._get_config()
        -- 小数値はfloorで切り捨て
        return cfg.talk_interval_min == 45
           and cfg.talk_interval_max == 120
    "#,
    );

    assert!(
        result.is_ok(),
        "Float values should be floored: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}
