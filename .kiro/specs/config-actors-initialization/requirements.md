# Requirements Document

## Introduction

pasta.toml の `[actor.*]` セクションを使用してアクター定数を定義し、Luaランタイム起動時に `STORE.actors` を自動初期化する機能。これにより、ゴースト作成者は設定ファイルでアクターのプロパティ（spot、surface等）を宣言的に定義できるようになる。

## Project Description (Input)

STORE.actors = CONFIG.actorとなるように、STORE.actorsの初期化処理を行う。もし、CONFIG.actorが存在しない場合は、空テーブル{}で初期化する。

### 追加のスコープ

Luaランタイム作成直後に直ちに有効化可能な公開ライブラリは先に公開する。
`@pasta_config`の公開タイミングはコンフィグ読み込みが終わり、Luaランタイム起動後最初に行うこと。そのほか、公開遅延する理由が無いライブラリモジュールは直ちにセットアップすること。

### 利用イメージ

```toml
# pasta.toml
[actor.さくら]
spot = 0
default_surface = 0

[actor.うにゅう]
spot = 1
default_surface = 10
```

これにより `STORE.actors["さくら"].spot` = 0 等が自動設定される。

## Requirements

### Requirement 1: pasta.toml での actor セクション定義

**Status:** ✅ **既存実装により満たされている** - 実装不要

**Objective:** As a ゴースト作成者, I want pasta.toml の `[actor.*]` セクションでアクター定数を定義したい, so that Lua コードを書かずにアクターのプロパティを設定できる

#### Acceptance Criteria（検証のみ）

1. ✅ pasta.toml の `[actor.アクター名]` セクションは `CONFIG.actor` テーブルとして `@pasta_config` に自動公開される（`PastaConfig::parse()` が `[loader]` 以外の全セクションを `custom_fields` に格納）
2. ✅ セクション内のキー・バリューペアはアクターオブジェクトのプロパティとして保持される（`toml_to_lua()` が再帰的にテーブル変換）
3. ✅ 任意のカスタムフィールド（spot, default_surface等）が許可される（TOML構造がそのまま保持）
4. ✅ `[actor.*]` セクション不在時は `CONFIG.actor` が `nil` となる（TOMLパース仕様）

**検証方法:** 既存のconfigテスト（`loader_integration_test.rs:L97-L148`）で確認済み

### Requirement 2: STORE.actors の自動初期化

**Objective:** As a Pasta ランタイム, I want Lua VM 起動時に `STORE.actors` を `CONFIG.actor` で初期化したい, so that トランスパイル済みコードがアクター定数を即座に利用できる

#### Acceptance Criteria

**pasta.store モジュール初期化時（依存: @pasta_config のみ）:**

1. the pasta.store モジュール shall モジュール読み込み時に `STORE.reset()` を呼び出す（既存フィールドの初期化ロジックを再利用）
2. When `STORE.reset()` 内で `STORE.actors = {}` が実行された後, the pasta.store モジュール shall `CONFIG.actor` がテーブル型なら `STORE.actors = CONFIG.actor` で上書きする
3. When `CONFIG.actor` がテーブル型でない（nil, 文字列, 数値等）場合, the pasta.store モジュール shall `STORE.actors` を空テーブル `{}` のままにする
4. the pasta.store モジュール shall メタテーブル設定を行わない（pasta.actor モジュールに委譲）

**pasta.actor モジュール初期化時（依存: pasta.store のみ）:**

5. the pasta.actor モジュール shall `STORE.actors` の各要素（テーブル型のみ）に `ACTOR_IMPL` メタテーブルを設定する
6. When `STORE.actors[name]` の要素がテーブル型でない場合, the pasta.actor モジュール shall そのエントリをスキップする（エラーにしない）
7. While `STORE.actors[name]` にメタテーブルが設定されたアクターが存在する場合, the ACTOR.get_or_create shall 既存アクターを返し、上書きしない

### Requirement 3: ライブラリモジュールの早期公開

**Status:** ✅ **既存実装により満たされている** - 実装不要

**Objective:** As a Lua スクリプト開発者, I want ランタイム起動直後から組み込みモジュールを利用したい, so that 初期化コードでも全機能を使用できる

#### Acceptance Criteria（検証のみ）

1. ✅ Lua VM 作成直後、`@pasta_config` モジュールが最初に登録されている（`runtime/mod.rs:538`）
2. ✅ `@enc`, mlua-stdlibモジュール（`@assertions`, `@regex`, `@json`, `@yaml`）が即座に登録されている
3. ✅ シーン辞書読み込み前に `@pasta_persistence` モジュールが登録されている（`runtime/mod.rs:544`）
4. ✅ `@pasta_search` モジュールの登録のみがシーン辞書読み込み後（`finalize_scene()` 内）

**検証方法:** 既存のランタイム初期化テスト（`loader_integration_test.rs`）で確認済み

### Requirement 4: 既存コードとの後方互換性

**Objective:** As a 既存ゴースト開発者, I want 既存の STORE.actors 操作コードが引き続き動作することを保証したい, so that 移行コストなく新機能を利用できる

#### Acceptance Criteria

1. When 既存コードが `STORE.actors[name] = {...}` で動的にアクターを追加した場合, the pasta.store モジュール shall CONFIG 由来のアクターと共存させる（`CONFIG.actor` にも反映される）
2. When ACTOR.get_or_create(name) が呼ばれた場合 and CONFIG 由来のアクターが存在する場合, the pasta.actor モジュール shall CONFIG 由来のプロパティを保持したアクターを返す
3. When STORE.reset() が呼ばれた場合, the pasta.store モジュール shall 全フィールドをクリアした後、`CONFIG.actor` がテーブル型なら `STORE.actors = CONFIG.actor` で再設定する
4. When STORE.reset() 後に `STORE.actors` が参照共有されている場合, the `ACTOR_IMPL` メタテーブル shall すでに設定済みのため維持される（pasta.actor の初期化時に設定済み）
