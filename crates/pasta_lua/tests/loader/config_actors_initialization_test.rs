//! CONFIG.actor → STORE.actors 自動初期化テスト
//!
//! Requirements:
//! - 2.1: CONFIG.actor がテーブル型の場合、STORE.actors に参照共有で設定
//! - 2.2: CONFIG.actor がテーブル型でない場合、空テーブル維持
//! - 2.3: メタテーブル設定は pasta.actor モジュールに委譲
//! - 2.4: STORE.actors の各テーブル要素に ACTOR_IMPL メタテーブル設定
//! - 2.5: 非テーブル要素はスキップ
//! - 2.6: get_or_create で既存アクター保持
//! - 4.1: 動的追加と CONFIG 由来アクターの共存
//! - 4.2: ACTOR.get_or_create が CONFIG 由来プロパティを保持

use crate::common;

use common::copy_fixture_to_temp;
use pasta_lua::loader::PastaLoader;

// ============================================================================
// Task 2.1: pasta.store 初期化ロジックの単体テスト
// ============================================================================

/// CONFIG.actor がテーブル型の場合、STORE.actors が同一参照になること
/// (Requirement 2.1)
#[test]
fn test_store_actors_is_config_actor_reference() {
    let temp = copy_fixture_to_temp("with_actor_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // CONFIG.actor と STORE.actors が同一参照であることを確認
    let result = runtime
        .exec(
            r#"
        local CONFIG = require "@pasta_config"
        local STORE = require "pasta.store"
        return CONFIG.actor == STORE.actors
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "CONFIG.actor と STORE.actors は同一参照であるべき"
    );
}

/// CONFIG.actor の値が STORE.actors から取得できること
/// (Requirement 2.1)
#[test]
fn test_store_actors_has_config_values() {
    let temp = copy_fixture_to_temp("with_actor_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // さくらアクターの spot 値を確認
    let result = runtime
        .exec(
            r#"
        local STORE = require "pasta.store"
        return STORE.actors["さくら"].spot
    "#,
        )
        .unwrap();
    assert_eq!(result.as_i64(), Some(0), "さくらの spot は 0 であるべき");

    // うにゅうアクターの spot 値を確認
    let result = runtime
        .exec(
            r#"
        local STORE = require "pasta.store"
        return STORE.actors["うにゅう"].spot
    "#,
        )
        .unwrap();
    assert_eq!(result.as_i64(), Some(1), "うにゅうの spot は 1 であるべき");
}

/// CONFIG.actor が nil の場合、STORE.actors が空テーブルであること
/// (Requirement 2.2)
#[test]
fn test_store_actors_empty_when_no_config_actor() {
    let temp = copy_fixture_to_temp("minimal");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // STORE.actors が空テーブルであることを確認
    let result = runtime
        .exec(
            r#"
        local STORE = require "pasta.store"
        local count = 0
        for _ in pairs(STORE.actors) do count = count + 1 end
        return count
    "#,
        )
        .unwrap();
    assert_eq!(
        result.as_i64(),
        Some(0),
        "CONFIG.actor が無い場合、STORE.actors は空テーブル"
    );
}

// ============================================================================
// Task 2.2: pasta.actor メタテーブル設定の単体テスト
// ============================================================================

/// STORE.actors の各テーブル要素に ACTOR_IMPL メタテーブルが設定されること
/// (Requirement 2.4)
#[test]
fn test_config_actors_have_metatable() {
    let temp = copy_fixture_to_temp("with_actor_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // メタテーブルが設定されていることを確認（:create_word が呼べる）
    let result = runtime
        .exec(
            r#"
        local STORE = require "pasta.store"
        local actor = STORE.actors["さくら"]
        return getmetatable(actor) ~= nil
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "CONFIG 由来アクターにはメタテーブルが設定されるべき"
    );
}

/// メタテーブルメソッド（create_word）が利用可能なこと
/// (Requirement 2.4, 4.2)
/// 注: CONFIG由来アクターにはnameフィールドがないため、ACTOR.get_or_create経由で
/// nameが設定されたアクターを取得してからcreate_wordをテストする
#[test]
fn test_config_actors_can_use_metatable_methods() {
    let temp = copy_fixture_to_temp("with_actor_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // ACTOR.get_or_create経由で取得したアクターでcreate_wordが使えることを確認
    let result = runtime
        .exec(
            r#"
        local ACTOR = require "pasta.actor"
        local actor = ACTOR.get_or_create("さくら")
        local builder = actor:create_word("表情")
        return type(builder) == "table"
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        ":create_word メソッドが利用可能であるべき"
    );
}

// ============================================================================
// Task 2.3: CONFIG.actor と動的追加の共存テスト（統合テスト）
// ============================================================================

/// ACTOR.get_or_create が CONFIG 由来プロパティを保持したアクターを返すこと
/// (Requirement 4.2)
#[test]
fn test_get_or_create_preserves_config_properties() {
    let temp = copy_fixture_to_temp("with_actor_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // ACTOR.get_or_create で取得したアクターが CONFIG 由来プロパティを保持
    let result = runtime
        .exec(
            r#"
        local ACTOR = require "pasta.actor"
        local actor = ACTOR.get_or_create("さくら")
        return actor.spot
    "#,
        )
        .unwrap();
    assert_eq!(
        result.as_i64(),
        Some(0),
        "get_or_create は CONFIG 由来の spot を保持すべき"
    );
}

/// 動的追加と CONFIG 由来アクターが共存すること
/// (Requirement 4.1)
#[test]
fn test_dynamic_and_config_actors_coexist() {
    let temp = copy_fixture_to_temp("with_actor_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // 動的にアクターを追加しても CONFIG 由来アクターが残っていることを確認
    let result = runtime
        .exec(
            r#"
        local STORE = require "pasta.store"
        local ACTOR = require "pasta.actor"
        
        -- 動的にアクターを追加
        local new_actor = ACTOR.get_or_create("マル")
        new_actor.spot = 2
        
        -- CONFIG 由来アクターが残っていることを確認
        local sakura = STORE.actors["さくら"]
        local maru = STORE.actors["マル"]
        
        return sakura ~= nil and maru ~= nil and sakura.spot == 0 and maru.spot == 2
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "動的追加と CONFIG 由来アクターが共存すべき"
    );
}

// ============================================================================
// co-exec-actor-init: CONFIG由来アクターの name フィールド注入テスト
// ============================================================================

/// CONFIG由来アクターに name フィールドが正しく注入されていること
/// (co-exec-actor-init Requirement 1.1, 1.5)
#[test]
fn test_config_actors_have_name_field() {
    let temp = copy_fixture_to_temp("with_actor_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // さくらアクターの name フィールドを確認
    let result = runtime
        .exec(
            r#"
        local STORE = require "pasta.store"
        return STORE.actors["さくら"].name == "さくら"
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "CONFIG由来アクターに name = 'さくら' が注入されるべき"
    );

    // うにゅうアクターの name フィールドを確認
    let result = runtime
        .exec(
            r#"
        local STORE = require "pasta.store"
        return STORE.actors["うにゅう"].name == "うにゅう"
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "CONFIG由来アクターに name = 'うにゅう' が注入されるべき"
    );
}

/// CONFIG.actor 経由でも name が参照できること（参照共有の確認）
/// (co-exec-actor-init Requirement 1.1)
#[test]
fn test_config_actor_name_via_config_module() {
    let temp = copy_fixture_to_temp("with_actor_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    let result = runtime
        .exec(
            r#"
        local CONFIG = require "@pasta_config"
        return CONFIG.actor["さくら"].name == "さくら"
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "CONFIG.actor 経由でも name が参照できるべき（参照共有）"
    );
}

/// 動的追加アクターが CONFIG.actor にも反映されること（参照共有の検証）
/// (Requirement 4.1)
#[test]
fn test_dynamic_actor_reflects_in_config() {
    let temp = copy_fixture_to_temp("with_actor_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // STORE.actors への追加が CONFIG.actor にも反映されることを確認
    let result = runtime
        .exec(
            r#"
        local CONFIG = require "@pasta_config"
        local STORE = require "pasta.store"
        local ACTOR = require "pasta.actor"
        
        -- 動的にアクターを追加
        local new_actor = ACTOR.get_or_create("マル")
        new_actor.spot = 2
        
        -- CONFIG.actor にも反映されていることを確認（参照共有）
        return CONFIG.actor["マル"] ~= nil and CONFIG.actor["マル"].spot == 2
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "STORE.actors への変更は CONFIG.actor にも反映されるべき（参照共有）"
    );
}
