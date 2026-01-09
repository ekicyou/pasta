# Requirements Document

## Introduction

本仕様は `pasta_lua_design_refactor` 親仕様から派生するトランスパイラー子仕様である。

`code_generator.rs` の出力形式を Act-first アーキテクチャに変更し、親仕様の設計に準拠したLuaコードを生成するよう修正する。

### 親仕様との関係

- **親仕様**: `.kiro/specs/completed/pasta_lua_design_refactor/`
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
| アクター発話 | `act.アクター:talk("テキスト")` | `act.アクター:talk("テキスト")` |
| 単語参照 | `act.アクター:word("name")` (単語参照) | `act.アクター:talk(act.アクター:word("name"))` (talk()で包む) |
| さくらスクリプト | `act.アクター:talk(\s[0])` | `act:sakura_script(\s[0])` |

### 出力形式の変更例

**現状の出力:**
```lua
do
    local SCENE = PASTA.create_scene("モジュール名_1")
    
    function SCENE.__start__(ctx, ...)
        local args = { ... }
        PASTA.clear_spot(ctx)
        PASTA.set_spot(ctx, "さくら", 0)
        local act, save, var = PASTA.create_session(SCENE, ctx)
        
        act:talk("こんにちは")
    end
end
```

**設計が求める出力:**
```lua
do
    local SCENE = {}
    
    function SCENE.__start__(act, ...)
        local args = { ... }
        act:clear_spot()
        act:set_spot("さくら", 0)
        local save, var = act:init_scene(SCENE)
        
        act.さくら:talk("こんにちは")
        act:sakura_script("\\s[0]")
    end
    
    PASTA.create_scene("モジュール名_1", "__start__", SCENE.__start__)
end
```

## Requirements

### Requirement 1: シーン関数シグネチャの変更
**Objective:** As a トランスパイラ開発者, I want シーン関数がactを第1引数で受け取る, so that Act-firstアーキテクチャに準拠する

**親仕様参照**: Requirement 5.6, 8.4

#### Acceptance Criteria
1. The code_generator shall シーン関数シグネチャを `function SCENE.__start__(act, ...)` 形式で出力する
2. The code_generator shall ローカルシーン関数シグネチャを以下の形式で出力する
   - エントリーポイント（シーン名なし）: `function SCENE.__start__(act, ...)`
   - 名前付きローカルシーン: `function SCENE.__<sanitized_name>_<counter>__(act, ...)` （例：`__scene_1__`, `__会話_2__`）
3. The code_generator shall 引数変数を `local args = { ... }` で定義する
4. The code_generator shall 第1引数の名前を `ctx` から `act` に変更する

### Requirement 2: init_scene呼び出しパターンの変更
**Objective:** As a トランスパイラ開発者, I want シーン関数冒頭でact:init_scene()を呼び出す, so that save/var参照を取得できる

**親仕様参照**: Requirement 5.7, 8.5

#### Acceptance Criteria
1. The code_generator shall シーン関数冒頭に `local save, var = act:init_scene(SCENE)` を出力する
2. The code_generator shall `PASTA.create_session(SCENE, ctx)` 呼び出しを生成しない
3. The code_generator shall save, var 変数をinit_scene()の戻り値から取得する形式で出力する
4. The code_generator shall スポット管理の後にinit_scene()呼び出しを配置する

### Requirement 3: スポット管理APIの変更
**Objective:** As a トランスパイラ開発者, I want スポット管理がact経由で行われる, so that 設計に準拠したコードを生成できる

**親仕様参照**: Requirement 6.1, 6.2, 6.3

#### Acceptance Criteria
1. The code_generator shall スポットクリアを `act:clear_spot()` 形式で出力する
2. The code_generator shall スポット設定を `act:set_spot("name", number)` 形式で出力する
3. The code_generator shall スポット管理をinit_scene()呼び出しの前に配置する
4. When アクターリストが存在する場合, the code_generator shall clear_spot()を最初に出力する
5. The code_generator shall 各アクターに対してset_spot()を順番に出力する

### Requirement 4: アクタープロキシ呼び出しパターン
**Objective:** As a トランスパイラ開発者, I want talk/wordがアクタープロキシ経由で呼び出される, so that 設計に準拠したコードを生成できる

**親仕様参照**: Requirement 5.8, 8.8

**議論背景**:
- word() は単語解決のみを行い、トークン化は行わない（word は search-only）
- 親仕様の設計パターンでは word() の結果を talk() で包む: `act.さくら:talk(act.さくら:word("笑顔"))`
- call() の動作は変わらない（すでに正しい形式で出力されている）

#### Acceptance Criteria
1. The code_generator shall 発話を `act.アクター:talk("テキスト")` 形式で出力する
2. The code_generator shall 単語参照を `act.アクター:talk(act.アクター:word("name"))` 形式で出力する（talk()で包む）
3. The code_generator shall 現在のアクターをコンテキストとして追跡する
4. When アクターが指定されていない場合, the code_generator shall デフォルトアクターを使用する
5. The code_generator shall `act:talk()` パターン（アクター指定なし）を生成しない（さくらスクリプト出力用の `act:sakura_script()` を除く）

### Requirement 5: シーン遷移APIの変更
**Objective:** As a トランスパイラ開発者, I want act:call()がsearch_resultを受け取る, so that Rust側検索結果を使用できる

**親仕様参照**: Requirement 8.7

#### Acceptance Criteria
1. The code_generator shall シーン呼び出しを `act:call(search_result, opts, ...)` 形式で出力する
2. The code_generator shall search_resultを `{"global_name", "local_name"}` テーブル形式で生成する
   - local_name パターン:
     - エントリーポイント（シーン名なし）: `"__start__"`
     - 名前付きローカルシーン: `"__<sanitized_name>_<counter>__"` （例：`"__scene_1__"`, `"__会話_2__"`）
3. When モジュール内シーン呼び出しの場合, the code_generator shall 現在のモジュール名をglobal_nameとして使用する
4. The code_generator shall optsを空テーブル `{}` として出力する
5. The code_generator shall 末尾呼び出し最適化（return付き）を維持する

### Requirement 6: さくらスクリプト出力の変更
**Objective:** As a トランスパイラ開発者, I want さくらスクリプトがact経由で出力される, so that 設計に準拠したコードを生成できる

**親仕様参照**: Requirement 3.7

**決定事項**:
- さくらスクリプトはアクター非依存（アクター固有ではない）
- 専用関数 `act:sakura_script(text)` として Lua側に実装される
- トークンタイプが分離される（親仕様 Requirement 7-3 参照）

#### Acceptance Criteria
1. The code_generator shall さくらスクリプトを `act:sakura_script("text")` 形式で出力する（アクタープロキシ経由ではなく）
2. The code_generator shall エスケープシーケンスを適切に処理する
3. The code_generator shall sakura_script を talk() で包まない（単独出力）

### Requirement 7: 変数アクセスパターンの維持 & 文字列リテラル処理
**Objective:** As a トランスパイラ開発者, I want save/var変数アクセスが維持される, so that 既存の変数展開ロジックとの互換性を保つ

**親仕様参照**: Requirement 8.6

**統一ルール**: すべての文字列リテラル出力は `StringLiteralizer::literalize()` を通す（例外なし）

#### Acceptance Criteria
1. The code_generator shall 永続変数アクセスを `save.変数名` 形式で出力する
2. The code_generator shall 作業変数アクセスを `var.変数名` 形式で出力する
3. The code_generator shall 変数代入を `save.変数名 = 値` 形式で出力する
4. The code_generator shall 既存の変数スコープ解決ロジックを維持する
5. The code_generator shall すべての文字列リテラルを `StringLiteralizer::literalize()` で処理する
6. The code_generator shall word()の単語名引数も `StringLiteralizer::literalize()` で処理する
7. The code_generator shall エスケープ文字も `StringLiteralizer::literalize()` で処理する

### Requirement 8: テスト互換性
**Objective:** As a トランスパイラ開発者, I want 既存テストが新出力形式に対応する, so that リグレッションを防止できる

#### Acceptance Criteria
1. The テスト修正 shall 既存の transpiler_integration_test.rs のアサーションを新出力形式に対応させる
2. The テスト修正 shall lua_specs/ 配下のLuaテストを新API形式に対応させる
3. The テスト修正 shall fixtures/ 配下の期待出力ファイルを更新する
4. When 全テストを実行した場合, the テスト shall 成功する
5. The テスト修正 shall 新出力形式を検証する追加テストケースを含む

### Requirement 9: ドキュメント更新
**Objective:** As a トランスパイラ開発者, I want code_generator.rsのドキュメントが更新される, so that 新設計が正しく文書化される

#### Acceptance Criteria
1. The code_generator.rs shall モジュールレベルのドキュメントコメントを更新する
2. The code_generator.rs shall 各メソッドのドキュメントコメントを新形式に合わせて更新する
3. The code_generator.rs shall 出力例のコードブロックを新形式に更新する

## Out of Scope

- Lua側モジュールの実装（pasta_lua_implementation 仕様で対応）
- Rust側検索モジュールの実装（pasta_search_module 仕様で対応）
- areka/shiori拡張モジュールの実装
- pasta_core パーサーの変更
- Luaランタイムの変更

## Technical Notes

### 文字列リテラル処理の統一ルール

**重要**: すべての文字列リテラル出力は `StringLiteralizer::literalize()` を通す（例外なし）

これにより以下を保証：
- Lua の特殊文字（`[`, `]`, `"`, `\` 等）が正しくエスケープされる
- 長い文字列は `[=[...]=]` 形式で出力される
- コード生成の一貫性が保たれ、バグが防止される

**適用対象**:
- `act.アクター:talk()` の文字列引数 ✅ 現在OK
- `act.アクター:word()` の単語名引数 ❌ 修正必要
- `act:sakura_script()` の文字列引数 ✅ 修正済み
- `Action::Escape` のエスケープ文字 ❌ 修正必要

### 影響を受けるファイル

- `crates/pasta_lua/src/code_generator.rs` - 主要な変更対象
- `crates/pasta_lua/tests/transpiler_integration_test.rs` - テスト更新
- `crates/pasta_lua/tests/fixtures/` - 期待出力の更新
- `crates/pasta_lua/tests/lua_specs/` - Lua単体テストの更新

### 変更の優先順位

1. **Phase 1**: シーン関数シグネチャ、init_scene（Req 1, 2）
2. **Phase 2**: スポット管理API（Req 3）
3. **Phase 3**: create_scene API（Req 4）
4. **Phase 4**: アクタープロキシ呼び出し（Req 5）
5. **Phase 5**: シーン遷移API（Req 6）
6. **Phase 6**: さくらスクリプト、変数アクセス（Req 7, 8）
7. **Phase 7**: テスト更新、ドキュメント（Req 9, 10）

### 後方互換性

本変更はLua側APIの破壊的変更を伴う。pasta_lua_implementation仕様と同時に実装し、整合性を保つ必要がある。
