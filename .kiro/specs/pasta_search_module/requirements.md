# Requirements Document

## Introduction

本仕様は `pasta_lua_design_refactor` 親仕様から派生する検索モジュール子仕様である。

Rust側のシーン辞書・単語辞書検索機能を mlua バインディングでLua側に公開し、`act:word()`, `PROXY:word()`, `act:call()` から呼び出せるようにする。

### 親仕様との関係

- **親仕様**: `.kiro/specs/pasta_lua_design_refactor/`
- **参照セクション**:
  - pasta.actor (PROXY:word) - 4レベル検索
  - pasta.act (ACT:word) - 3レベル検索
  - pasta.scene - シーン検索
  - Requirement 4.6, 4.8, 5.5

### 前提条件

- `pasta_lua_implementation` 仕様のスタブ実装が完了していること
- pasta_core の SceneRegistry, WordRegistry が利用可能であること

## Requirements

### Requirement 1: シーン検索API
**Objective:** As a Lua開発者, I want Rust側のシーン検索をLuaから呼び出せる, so that act:call()が動作する

**親仕様参照**: Requirement 5.5, 8.7

#### Acceptance Criteria
1. The 検索モジュール shall `search_scene(prefix)` 関数をLuaに公開する
2. The 検索モジュール shall 前方一致検索を実行する
3. The 検索モジュール shall `{global_name, local_name}` タプルを返す
4. The 検索モジュール shall 複数候補がある場合はランダム選択する
5. The 検索モジュール shall 候補がない場合は nil を返す

### Requirement 2: 単語検索API（アクター指定）
**Objective:** As a Lua開発者, I want アクター名を指定して単語検索できる, so that PROXY:word()が動作する

**親仕様参照**: Requirement 4.6, 4.8

#### Acceptance Criteria
1. The 検索モジュール shall `search_word(name, actor_name, global_scene_name)` 関数をLuaに公開する
2. The 検索モジュール shall 4レベル優先順位で検索する:
   - Level 1: アクターfield（Lua側で処理）
   - Level 2: SCENEfield（Lua側で処理）
   - Level 3: グローバルシーン名での単語検索
   - Level 4: 全体での単語検索
3. The 検索モジュール shall 検索結果の文字列を返す
4. The 検索モジュール shall 候補がない場合は nil を返す

### Requirement 3: 単語検索API（アクター非指定）
**Objective:** As a Lua開発者, I want アクター名なしで単語検索できる, so that ACT:word()が動作する

**親仕様参照**: Requirement 3.11

#### Acceptance Criteria
1. The 検索モジュール shall `search_word_global(name, global_scene_name)` 関数をLuaに公開する
2. The 検索モジュール shall 3レベル優先順位で検索する:
   - Level 1: SCENEfield（Lua側で処理）
   - Level 2: グローバルシーン名での単語検索
   - Level 3: 全体での単語検索
3. The 検索モジュール shall 検索結果の文字列を返す
4. The 検索モジュール shall 候補がない場合は nil を返す

### Requirement 4: mluaバインディング
**Objective:** As a Rust開発者, I want mlua経由でLuaに関数を公開できる, so that 検索機能が利用可能

#### Acceptance Criteria
1. The バインディング shall pasta_lua crate 内に実装する
2. The バインディング shall Lua globals に関数を登録する
3. The バインディング shall エラー時にLuaエラーを発生させる
4. The バインディング shall pasta_core のレジストリを参照する

### Requirement 5: ランダム選択の一貫性
**Objective:** As a システム設計者, I want ランダム選択が循環動作する, so that 多様な結果が得られる

**親仕様参照**: PROXY:word の副作用記述

#### Acceptance Criteria
1. The 検索モジュール shall 同一キーでの検索で循環的に異なる結果を返す
2. The 検索モジュール shall pasta_core の RandomSelector を使用する
3. The 検索モジュール shall 循環状態をセッション内で維持する

## Out of Scope

- Lua側モジュールの実装（pasta_lua_implementation 仕様）
- code_generator.rs の修正（pasta_lua_transpiler 仕様）
- シーン/単語の登録機能（pasta_core で既存）
