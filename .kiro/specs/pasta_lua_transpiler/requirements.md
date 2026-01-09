# Requirements Document

## Introduction

本仕様は `pasta_lua_design_refactor` 親仕様から派生するトランスパイラー子仕様である。

`code_generator.rs` の出力形式を Act-first アーキテクチャに変更し、親仕様の設計に準拠したLuaコードを生成するよう修正する。

### 親仕様との関係

- **親仕様**: `.kiro/specs/pasta_lua_design_refactor/`
- **参照セクション**: 
  - Supporting References（シーン関数の標準パターン）
  - pasta.init Service Interface
  - pasta.act Service Interface
  - pasta.actor ActorProxy
  - Requirement 8（Rust生成コードとの互換性）

### 現状と設計の差異

| 項目 | 現状の出力 | 設計が求める出力 |
|------|-----------|-----------------|
| シーン関数シグネチャ | `function SCENE.__start__(ctx, ...)` | `function SCENE.__start__(act, ...)` |
| セッション初期化 | `local act, save, var = PASTA.create_session(SCENE, ctx)` | `local save, var = act:init_scene(SCENE)` |
| スポットクリア | `PASTA.clear_spot(ctx)` | `act:clear_spot()` |
| スポット設定 | `PASTA.set_spot(ctx, "name", number)` | `act:set_spot("name", number)` |
| create_scene呼び出し | `PASTA.create_scene("module_name")` | `PASTA.create_scene("global_name", "local_name", scene_func)` |

## Requirements

### Requirement 1: シーン関数シグネチャの変更
**Objective:** As a トランスパイラ開発者, I want シーン関数がactを第1引数で受け取る, so that Act-firstアーキテクチャに準拠する

**親仕様参照**: Requirement 5.6, 8.4

#### Acceptance Criteria
1. The code_generator shall シーン関数を `function SCENE.__start__(act, ...)` 形式で出力する
2. The code_generator shall ローカルシーン関数を `function SCENE.__name_N__(act, ...)` 形式で出力する
3. The code_generator shall 引数変数 `args` を `local args = { ... }` で定義する

### Requirement 2: init_scene呼び出しパターンの変更
**Objective:** As a トランスパイラ開発者, I want シーン関数冒頭でact:init_scene()を呼び出す, so that save/var参照を取得できる

**親仕様参照**: Requirement 5.7, 8.5

#### Acceptance Criteria
1. The code_generator shall シーン関数冒頭に `local save, var = act:init_scene(SCENE)` を出力する
2. The code_generator shall `PASTA.create_session()` 呼び出しを削除する
3. The code_generator shall save, var 変数を init_scene() の戻り値から取得する形式に変更する

### Requirement 3: スポット管理APIの変更
**Objective:** As a トランスパイラ開発者, I want スポット管理がact経由で行われる, so that 設計に準拠したコードを生成できる

**親仕様参照**: Requirement 6.1, 6.2, 6.3

#### Acceptance Criteria
1. The code_generator shall `act:clear_spot()` 形式でスポットクリアを出力する
2. The code_generator shall `act:set_spot("name", number)` 形式でスポット設定を出力する
3. The code_generator shall スポット管理をinit_scene()呼び出しの前に配置する

### Requirement 4: create_scene APIの変更
**Objective:** As a トランスパイラ開発者, I want create_sceneがグローバル名・ローカル名・関数の3引数を取る, so that 階層構造に対応できる

**親仕様参照**: Requirement 5.1, 8.3

#### Acceptance Criteria
1. The code_generator shall `PASTA.create_scene(global_name, local_name, scene_func)` 形式で呼び出しを生成する
2. The code_generator shall グローバルシーン名をモジュール名から生成する
3. The code_generator shall ローカルシーン名を `__start__`, `__name_N__` パターンで生成する
4. The code_generator shall シーン関数への参照を第3引数として渡す

### Requirement 5: アクタープロキシ呼び出しパターン
**Objective:** As a トランスパイラ開発者, I want talk/wordがアクタープロキシ経由で呼び出される, so that 設計に準拠したコードを生成できる

**親仕様参照**: Requirement 5.8, 8.8

#### Acceptance Criteria
1. The code_generator shall `act.アクター:talk("テキスト")` 形式で発話を出力する
2. The code_generator shall `act.アクター:word("name")` 形式で単語参照を出力する
3. The code_generator shall 現在の `act:talk()` パターンを廃止する

### Requirement 6: シーン遷移APIの変更
**Objective:** As a トランスパイラ開発者, I want act:call()がsearch_resultを受け取る, so that Rust側検索結果を使用できる

**親仕様参照**: Requirement 8.7

#### Acceptance Criteria
1. The code_generator shall `act:call(search_result, opts, ...)` 形式でシーン呼び出しを出力する
2. The code_generator shall search_result を `{global_name, local_name}` タプル形式で生成する
3. The code_generator shall 現在のシーン呼び出しパターンを新形式に変更する

### Requirement 7: テスト互換性
**Objective:** As a トランスパイラ開発者, I want 既存テストが新出力形式に対応する, so that リグレッションを防止できる

#### Acceptance Criteria
1. The 修正 shall 既存の transpiler_integration_test.rs を新出力形式に対応させる
2. The 修正 shall 新出力形式を検証するテストケースを追加する
3. The 修正 shall 全テストが成功することを確認する

## Out of Scope

- Lua側モジュールの実装（pasta_lua_implementation 仕様）
- Rust側検索モジュールの実装（pasta_search_module 仕様）
- areka/shiori拡張モジュール
