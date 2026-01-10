# Requirements Document

## Introduction

本仕様は `pasta_lua_design_refactor` 親仕様から派生する検索モジュール子仕様である。

Rust側のシーン辞書・単語辞書検索機能を mlua バインディングでLua側に公開し、`act:word()`, `PROXY:word()`, `act:call()` から呼び出せるようにする。

### 親仕様との関係

- **親仕様**: `.kiro/specs/completed/pasta_lua_design_refactor/`
- **参照セクション**:
  - pasta.actor (PROXY:word) - 4レベル検索
  - pasta.act (ACT:word) - 3レベル検索
  - pasta.scene - シーン検索
  - design.md Requirement 4.6, 4.8, 5.5

### 前提条件

- pasta_core の SceneRegistry, WordDefRegistry, SceneTable, WordTable が利用可能であること
- pasta_core の RandomSelector トレイトが実装されていること
- pasta_lua crate が mlua 依存関係を持つこと（Cargo.toml に記載済み、未使用）
- pasta_lua の `LuaTranspiler::transpile()` が `TranspileContext` を返すこと

**注記**: pasta_lua に Lua VM 実行のためのランタイム層は**存在しない**（トランスパイラ層のみ）。本仕様の Requirement 9 でランタイム層を新規作成する。

### 技術コンテキスト

本モジュールは pasta_core の既存レジストリ構造を活用する：

| pasta_core コンポーネント | 用途 |
|--------------------------|------|
| `SceneTable` | シーン前方一致検索・ランダム選択 |
| `WordTable` | 単語検索・ランダム選択 |
| `RandomSelector` | ランダム選択の抽象化（循環動作） |
| `DefaultRandomSelector` | 本番用ランダム選択器 |

## Requirements

### Requirement 1: `@pasta_search` モジュール公開
**Objective:** As a Lua開発者, I want `require "@pasta_search"` でモジュールを取得できる, so that 検索APIを統一的に利用できる

**親仕様参照**: design.md Module Registration Pattern

#### Acceptance Criteria
1. The 検索モジュール shall `@pasta_search` というモジュール名で Lua に登録される
2. When `local SEARCH = require "@pasta_search"` を実行した場合, the 検索モジュール shall 公開API一覧を含むテーブルを返す
3. The `SEARCH` テーブル shall SearchContext を単一 UserData として公開し、本仕様で定義する全公開関数を属性として含む（`search_scene`, `search_word`, `set_scene_selector`, `set_word_selector`）。Lua側の呼び出しは メタテーブル設定により `SEARCH:search_scene(...)` および `SEARCH.search_scene(...)` の両形式で可能
4. The 検索モジュール shall pasta_lua の初期化時に自動的に登録される
5. The 検索モジュール shall 複数回の require でも同じモジュールインスタンスを返す（一度だけ初期化）

### Requirement 2: シーン検索API
**Objective:** As a Lua開発者, I want Rust側のシーン検索を全体・シーン指定の両方で呼び出せる, so that act:call()が動作する

**親仕様参照**: design.md Requirement 5.5, 8.7

#### Acceptance Criteria
1. The 検索モジュール shall `search_scene(name, global_scene_name)` 関数を `@pasta_search` モジュール内に属性として提供する
2. When `global_scene_name` が指定された場合（`search_scene(name, "シーン名")`）, the 検索モジュール shall 段階的フォールバック戦略で検索を実行する（ローカル → グローバル）
3. When ローカルシーン（`:global_scene_name` + `name` 形式）で結果が見つかった場合, the 検索モジュール shall そこから選択したシーン名を `global_name, local_name` として返す（複数戻り値タプル）
4. When ローカルシーン検索結果が０件の場合, the 検索モジュール shall グローバルシーン（`name` 形式）で検索を再実行する
5. When グローバルシーン検索でも結果が見つかった場合, the 検索モジュール shall そこから選択したシーン名を `global_name, local_name` として返す（複数戻り値タプル、local_name は `__start__`）
6. When `global_scene_name` が nil を明示的に渡された場合（`search_scene(name, nil)`）, the 検索モジュール shall グローバルシーンのみで検索を実行する
7. When 複数候補がある場合, the 検索モジュール shall RandomSelector でランダム選択する
8. When 全検索で候補がない場合, the 検索モジュール shall nil を返す

### Requirement 3: 単語検索API
**Objective:** As a Lua開発者, I want Rust側の単語検索を全体・シーン指定の両方で呼び出せる, so that PROXY:word() と ACT:word() の全レベル検索が動作する

**親仕様参照**: design.md Requirement 8.7, research.md (PASTA_RUNTIME.search_word API)

#### Acceptance Criteria
1. The 検索モジュール shall `search_word(name, global_scene_name)` 関数を `@pasta_search` モジュール内に属性として提供する
2. When `global_scene_name` が指定された場合（`search_word(name, "シーン名")`）, the 検索モジュール shall 段階的フォールバック戦略で検索を実行する（ローカル → グローバル）
3. When ローカルスコープ（`:global_scene_name:name` 形式）で結果が見つかった場合, the 検索モジュール shall そこから選択結果を返す
4. When ローカルスコープ検索結果が０件の場合, the 検索モジュール shall グローバルスコープ（`name` 形式）で検索を再実行する
5. When `global_scene_name` が nil を明示的に渡された場合（`search_word(name, nil)`）, the 検索モジュール shall グローバルスコープのみで検索を実行する
6. When グローバルスコープ（`name` 形式）で結果が見つかった場合, the 検索モジュール shall 文字列を返す
7. When 複数候補がある場合, the 検索モジュール shall RandomSelector でランダム選択する
8. When 全検索で候補がない場合, the 検索モジュール shall nil を返す

### Requirement 4: mlua バインディング実装と @pasta_search モジュール登録
**Objective:** As a Rust開発者, I want mlua経由で `@pasta_search` モジュールとその検索関数をLuaに公開できる, so that 検索機能が統一的なモジュールインターフェース経由で利用可能

#### Acceptance Criteria
1. The バインディング shall pasta_lua crate 内の `search` モジュールとして実装する
2. The バインディング shall `loader(lua: &Lua) -> Result<Table>` 関数を提供し、`@pasta_search` モジュールテーブルを生成する
3. The バインディング shall `register(lua: &Lua) -> Result<Table>` 関数を提供し、loader を呼び出してモジュールを Lua に登録する
4. The `loader` 関数 shall モジュール内に `search_scene`, `search_word` を属性として配置する（Requirement 1 の `SEARCH` テーブルに対応）
5. The バインディング shall mlua-stdlib の loader/register パターンに従って実装する
6. The バインディング shall SceneTable/WordTable が SceneInfo/String を直接返すことを期待する（pasta_core の `resolve_scene()`, `search_word()` メソッドで実装予定）
7. If Lua側でエラーが発生した場合, the バインディング shall mlua::Error を返す
8. The バインディング shall 複数の独立した Lua ランタイムインスタンスで各インスタンス用の SearchContext を管理する（design段階で UserData/Arc<Mutex<>>/mlua Registry パターンを決定）

### Requirement 5: ランダム選択の循環動作（pasta_core機能の確認）
**Objective:** As a システム設計者, I want Rust側の既存ランダム選択機能がLua側でも動作する, so that 同一キー検索で多様な結果を得られることが保証される

**親仕様参照**: design.md PROXY:word の副作用記述

**実装上の注記**: 本Requirementの検索ロジック（Requirement 2/3）は pasta_core の SceneTable/WordTable によって既に実装されており、mlua バインディング（Requirement 4）を通じて正しく呼び出されれば自動的に満たされます。

#### Acceptance Criteria
1. When Requirement 2/3 で定義した検索関数が呼び出される場合, the 検索結果 shall 同一キーでの複数回検索で循環的に異なる値を返す
2. The 循環動作 shall pasta_core の DefaultRandomSelector によって実装される
3. The RandomSelector 状態 shall SceneTable/WordTable のキャッシュ内で管理される（Lua側での明示的な状態管理不要）
4. When テスト時に MockRandomSelector を用いる場合, the 循環動作 shall 決定的な結果を返す

### Requirement 6: エラーハンドリング
**Objective:** As a Lua開発者, I want 検索エラー時に適切なエラーメッセージを受け取れる, so that デバッグが容易

#### Acceptance Criteria
1. If 引数の型が不正な場合, the 検索モジュール shall "expected string argument" エラーを返す
2. If 内部エラーが発生した場合, the 検索モジュール shall エラー詳細を含むLuaエラーを発生させる
3. The 検索モジュール shall 検索失敗（候補なし）をエラーではなく nil として処理する

### Requirement 7: パフォーマンス考慮
**Objective:** As a システム設計者, I want 検索が高速に動作する, so that 大量の呼び出しでもパフォーマンス低下が最小

#### Acceptance Criteria
1. The 検索モジュール shall SceneTable/WordTable への参照を保持し、検索ごとの再構築を避ける
2. The 検索モジュール shall ヒープアロケーションを最小化する
3. The 検索モジュール shall 検索結果の文字列コピーを最小限にする

### Requirement 8: RandomSelector 制御API（テスト機能）
**Objective:** As a テスト開発者, I want テスト時に検索結果を決定的に制御できる, so that 循環動作を検証できる

**親仕様参照**: Requirement 5 (ランダム選択の循環動作)

#### Acceptance Criteria
1. The 検索モジュール shall `set_scene_selector()` 関数を `@pasta_search` モジュール内に属性として提供する（シーン検索用）
2. The 検索モジュール shall `set_word_selector()` 関数を `@pasta_search` モジュール内に属性として提供する（単語検索用）
3. When `set_scene_selector()` を引数なしで呼び出した場合, the 検索モジュール shall SceneTable のRandomSelector をデフォルト（DefaultRandomSelector）にリセットする
4. When `set_scene_selector(n1, n2, n3, ...)` を整数の可変長引数で呼び出した場合, the 検索モジュール shall SceneTable 用の MockRandomSelector に切り替える
5. When `set_word_selector()` を引数なしで呼び出した場合, the 検索モジュール shall WordTable のRandomSelector をデフォルト（DefaultRandomSelector）にリセットする
6. When `set_word_selector(n1, n2, n3, ...)` を整数の可変長引数で呼び出した場合, the 検索モジュール shall WordTable 用の MockRandomSelector に切り替える
7. The MockRandomSelector shall 与えられたシーケンス順に確定的に選択を実行する
8. The MockRandomSelector shall シーケンスの末尾に達したら先頭にループして選択を続ける
9. If 引数が整数以外を含む場合, the 検索モジュール shall "expected integer argument" エラーを返す
10. The 各関数 shall モジュール初期化以外の任意のタイミングで呼び出し可能

### Requirement 9: pasta_lua ランタイム層（PastaLuaRuntime 構造体）
**Objective:** As a pasta_lua利用者, I want Rust側からLua VMを初期化し検索モジュールを自動登録できる, so that Luaスクリプト実行環境が統一的に提供される

**背景**: 本仕様の Requirement 1-8 は「Lua 環境に @pasta_search モジュールが登録されている」ことを前提としているが、その Lua 環境自体を提供するランタイム層が pasta_lua に存在しない。本 Requirement はその隠れた前提条件を明示化する。

**設計決定の経緯**: 議題2（DESIGN_REVIEW_ACTIONS.md）で「pasta_lua ランタイム構造体パターン」を採用決定

#### Acceptance Criteria
1. The pasta_lua crate shall `PastaLuaRuntime` 構造体を公開する
2. The `PastaLuaRuntime` shall 内部に mlua の `Lua` インスタンスを保持する
3. The `PastaLuaRuntime::new()` shall `TranspileContext` を入力として受け取る
4. The `PastaLuaRuntime::new()` shall 内部で mlua の `Lua::new()` を呼び出して Lua VM を初期化する
5. The `PastaLuaRuntime::new()` shall 初期化時に `search::loader()` を呼び出して `@pasta_search` モジュールを登録する（一度のみ）
6. The `PastaLuaRuntime` shall `TranspileContext` から `SceneRegistry` と `WordDefRegistry` を取得して `SearchContext` を生成する
7. The `PastaLuaRuntime` shall 複数インスタンス生成をサポートする（各インスタンスが独立した Lua VM と SearchContext を持つ）
8. The `PastaLuaRuntime` shall Static 変数を使用しない（スレッドセーフ要件）
9. The `PastaLuaRuntime` shall 将来の拡張用に他のモジュール登録メカニズムを持つ（例: `register_module()` メソッド）
10. The `PastaLuaRuntime` shall Lua スクリプト実行用の `exec()` または `run()` メソッドを提供する

#### 依存関係
- **入力**: `TranspileContext`（`LuaTranspiler::transpile()` の出力）
- **出力**: 初期化済み Lua VM + 登録済み `@pasta_search` モジュール
- **内部依存**: Requirement 4（mlua バインディング）、Requirement 1（モジュール登録）

## Implementation Guidance

### Design Decisions

#### global_scene_nameパラメータのオプション化 ✅ 決定

本仕様では `search_scene(prefix, global_scene_name)` と `search_word(name, global_scene_name)` の両関数で `global_scene_name` を **nil値対応のオプションパラメータ** として実装することを決定しました。

**決定内容**:
- Rust側：`global_scene_name` を `Option<&str>` として扱う
- Lua側：`nil` 値を受け入れる（Luaではニル可能なパラメータが標準的）
- 動作：`nil` 時はグローバルスコープのみで検索、値指定時は段階的フォールバック戦略を実行

**根拠**:
- Lua言語設計との統合性（ニル値は Lua の標準的な欠落値表現）
- mlua の引数検証で容易に実装可能（Option型との マッピング）
- 将来の拡張性を確保（新しいスコープレベルの追加時の互換性維持）
- Design段階での迷走を回避（実装方針が明確に決定）

### mlua-stdlib参照を用いた実装方法

Luaへの公開ライブラリの実装方法については、[mlua-stdlib](https://github.com/mlua-rs/mlua-stdlib) が非常に参考になるため、以下の方針で実装する：

**参照理由**:
- pasta_lua は既に mlua-stdlib を依存クレートとして参照している（Cargo.toml）
- mlua-stdlib は複数のライブラリ関数をLuaにバインディングする際のベストプラクティスを実装
- モジュール登録、エラーハンドリング、引数検証等の実装パターンが参考になる

**実装方針**:
1. mlua-stdlib のソースコードを解析し、ライブラリ関数の登録パターンを理解する
2. 同様の実装手法（モジュール登録関数等）を採用する
3. pasta_core の型（SceneTable, WordTable, RandomSelector）をmlua経由でLuaに安全に公開する
4. エラーハンドリングと引数検証は mlua-stdlib と同じ品質レベルを維持する

**期待される構成**:
- pasta_lua/src/search/ ディレクトリに検索モジュールを実装
- `loader(lua: &Lua) -> Result<Table>` - `@pasta_search` モジュールテーブル生成
- `register(lua: &Lua) -> Result<Table>` - モジュール登録（mlua-stdlib パターン）
- SceneTable/WordTable 参照の管理方法は Design段階で詳細決定
- mlua-stdlib のコード生成・登録パターンに従う

#### 複数ランタイムインスタンス対応 ✅ 要件確認

pasta_lua は複数の独立した Lua ランタイムインスタンスをサポートする必要があります。

**制約**:
- Static 変数で SceneTable/WordTable を保持することは禁止
- 各ランタイムインスタンスは独立した SceneTable/WordTable を持つ必要がある
- スレッドローカル（TLS）でも複数インスタンス対応には不十分

**選択肢**:
1. **UserData ラッピング**（推奨候補）: SceneTable/WordTable を UserData として Lua に登録し、各ランタイムが独立管理
2. **Arc<Mutex<>> + Lua Globals**: Arc でランタイムごとに参照を保持
3. **mlua UserData Registry**: mlua 提供の UserData registry パターン

**Decision pending**: Design 段階で最適なパターンを決定（Requirement 4, Criteria 6 での詳細決定項目）
## Out of Scope

- Lua側モジュール（pasta.act, pasta.actor）の実装（pasta_lua_implementation 仕様）
- code_generator.rs の修正（pasta_lua_transpiler 仕様）
- シーン/単語の登録機能（pasta_core で既存）
- WordTable/SceneTable の構築ロジック（pasta_core で既存）
- Level 1/2 検索（アクターfield, SCENEfield）- Lua側で処理

## Appendix: 検索レベル参照表

### PROXY:word(name) - 4レベル検索

| Level | 検索対象 | 実装場所 | 本仕様スコープ |
|-------|---------|---------|--------------|
| 1 | アクターfield | Lua (pasta.actor) | ❌ |
| 2 | SCENEfield | Lua (pasta.scene) | ❌ |
| 3 | `:global_scene_name:name` | Rust (search_word with param) | ✅ |
| 4 | `name` (全体) | Rust (search_word with nil) | ✅ |

### ACT:word(name) - 3レベル検索

| Level | 検索対象 | 実装場所 | 本仕様スコープ |
|-------|---------|---------|--------------|
| 1 | SCENEfield | Lua (pasta.scene) | ❌ |
| 2 | `:global_scene_name:name` | Rust (search_word with param) | ✅ |
| 3 | `name` (全体) | Rust (search_word with nil) | ✅ |
