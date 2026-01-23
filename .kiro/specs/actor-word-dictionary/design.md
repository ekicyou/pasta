# Design Document: actor-word-dictionary

## Overview

**Purpose**: アクター単語辞書機能により、DSL作成者はアクター定義内に複数値属性とLua関数を記述し、ランダムトーク生成時に多様な出力を実現できる。

**Users**: Pasta DSLでデスクトップマスコットの会話スクリプトを記述するゴースト作成者。

**Impact**: 既存のアクター属性（1対1キー・バリュー）を配列形式に拡張し、ランタイムでのランダム選択とフォールバック検索を追加する。

### Goals

- アクター属性の複数値定義とLua配列形式出力を実現
- 6レベルのフォールバック検索（関数=完全一致、辞書=前方一致）を実装
- アクター定義内へのLua関数埋め込みをサポート
- 後方互換性を維持（既存スクリプトが変更なしで動作）

### Non-Goals

- シーン単語辞書の変更（既存機能）
- Rune言語対応（現状はLuaバックエンドのみ）
- 前方一致検索の完全Lua実装（Rustヘルパー使用）

---

## Architecture

### Existing Architecture Analysis

**現行アーキテクチャ**:
- `pasta_core`: 言語非依存層（PEG パーサー、AST定義）
- `pasta_lua`: Luaバックエンド（トランスパイラ、ランタイム）

**既存ドメイン境界**:
- パーサー/AST（`pasta_core/src/parser/`）
- コード生成（`pasta_lua/src/code_generator.rs`）
- ランタイム（`pasta_lua/scripts/pasta/`）

**維持すべき統合ポイント**:
- `ActorScope` → `generate_actor()` → `actor.lua` の変換フロー
- `GlobalSceneScope`/`LocalSceneScope` での `code_blocks` パターン

### Architecture Pattern & Boundary Map

```mermaid
graph TB
    subgraph pasta_core["pasta_core（言語非依存層）"]
        Grammar["grammar.pest<br/>actor_scope_item"]
        Parser["parse_actor_scope<br/>Rule::code_block処理追加"]
        AST["ActorScope<br/>+code_blocks追加"]
    end
    
    subgraph pasta_lua["pasta_lua（Luaバックエンド）"]
        CodeGen["code_generator.rs<br/>generate_actor()"]
        ActorLua["actor.lua<br/>PROXY:word()"]
        RustHelper["Rustヘルパー<br/>search_word_prefix()"]
    end
    
    Grammar --> Parser
    Parser --> AST
    AST --> CodeGen
    CodeGen --> ActorLua
    ActorLua --> RustHelper
```

**Architecture Integration**:
- 選択パターン: **既存拡張**（Option A）- `GlobalSceneScope` と同じ `code_blocks` パターンを踏襲
- ドメイン境界: パーサー/AST変更は `pasta_core`、コード生成/ランタイム変更は `pasta_lua`
- 既存パターン維持: `CodeBlock` 型、`generate_*` メソッド群、`PROXY:*` メソッド群
- Steering準拠: 2パス変換、Yield型出力、日本語フレンドリー

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| Parser | Pest 2.8 | `actor_scope_item` に `code_block` サポート | 既存対応済み |
| AST | Rust struct | `ActorScope.code_blocks` 追加 | 新規フィールド |
| Transpiler | code_generator.rs | 配列形式出力 + コードブロック展開 | 修正 |
| Runtime | Lua 5.4 (mlua) | フォールバック検索 + ランダム選択 | actor.lua 拡張 |
| FFI | mlua | 前方一致検索ヘルパー | 新規 |

---

## System Flows

### 単語置換フロー

```mermaid
sequenceDiagram
    participant Script as Pasta Script
    participant Transpiler as code_generator.rs
    participant Actor as actor.lua
    participant Rust as Rust Helper
    
    Note over Script,Transpiler: トランスパイル時
    Script->>Transpiler: ActorScope (words, code_blocks)
    Transpiler->>Transpiler: 配列形式でLua出力
    Transpiler->>Transpiler: code_blocks展開
    
    Note over Actor,Rust: ランタイム時
    Actor->>Actor: PROXY:word(key) 呼び出し
    Actor->>Actor: 1. アクター関数検索（完全一致）
    Actor->>Rust: 2. アクター辞書検索（前方一致）
    Rust-->>Actor: 候補配列
    Actor->>Actor: ランダム選択
    alt 見つからない場合
        Actor->>Actor: 3. シーン関数検索
        Actor->>Rust: 4. シーン辞書検索
        Actor->>Actor: 5. グローバル関数検索
        Actor->>Rust: 6. グローバル辞書検索
    end
    Actor-->>Script: 選択された値
```

---

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1 | DSL構文 - 複数値アクター属性 | grammar.pest (既存) | - | - |
| 2 | トランスパイル - Lua配列出力 | code_generator.rs | generate_actor | トランスパイル時 |
| 3 | ランタイム - ランダム選択 | actor.lua | PROXY:word | ランタイム時 |
| 4 | 単語置換優先順位とフォールバック検索 | actor.lua, Rustヘルパー | PROXY:word, search_word_prefix | ランタイム時 |
| 4.関数定義 | アクター内Lua関数定義 | ActorScope, parse_actor_scope, code_generator.rs | generate_actor | トランスパイル時 |
| 5 | グローバル単語定義 | word.lua (既存) | - | - |
| 6 | 後方互換性 | 全コンポーネント | - | - |

---

## Components and Interfaces

### コンポーネントサマリー

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|--------------|--------|--------------|------------------|-----------|
| ActorScope | pasta_core/AST | アクター定義のAST表現 | 4.関数定義 | CodeBlock (P0) | State |
| parse_actor_scope | pasta_core/Parser | アクタースコープのパース | 4.関数定義 | Rule::code_block (P0) | Service |
| generate_actor | pasta_lua/Transpiler | アクター定義のLua出力 | 2, 4.関数定義 | ActorScope (P0), StringLiteralizer (P1) | Service |
| PROXY:word | pasta_lua/Runtime | 単語置換とフォールバック検索 | 3, 4 | search_word_prefix (P1) | Service |
| search_word_prefix | pasta_lua/FFI | 前方一致検索ヘルパー | 4 | WordDefRegistry (P0) | Service |

---

### pasta_core/AST

#### ActorScope（修正）

| Field | Detail |
|-------|--------|
| Intent | アクター定義をAST構造体として保持し、コードブロックを含む |
| Requirements | 4.関数定義 |

**Responsibilities & Constraints**
- アクター名、属性、単語定義、変数設定、コードブロックを保持
- `GlobalSceneScope`/`LocalSceneScope` と同じ `code_blocks` パターンを踏襲

**Dependencies**
- Inbound: parse_actor_scope — パース結果の構築 (P0)
- Outbound: generate_actor — Lua出力生成 (P0)

**Contracts**: State [x]

##### State Management

```rust
/// アクター定義スコープ（修正版）
#[derive(Debug, Clone)]
pub struct ActorScope {
    /// アクター名
    pub name: String,
    /// アクターの属性
    pub attrs: Vec<Attr>,
    /// アクターの単語定義（表情など）
    pub words: Vec<KeyWords>,
    /// アクターの変数設定
    pub var_sets: Vec<VarSet>,
    /// コードブロック（Lua関数定義）【新規】
    pub code_blocks: Vec<CodeBlock>,
    /// ソース位置
    pub span: Span,
}
```

**Implementation Notes**
- 既存の `GlobalSceneScope.code_blocks` と同じ型・パターンを使用
- `new()` メソッドに `code_blocks: Vec::new()` 初期化を追加

---

### pasta_core/Parser

#### parse_actor_scope（修正）

| Field | Detail |
|-------|--------|
| Intent | actor_scope をパースし、code_block を含む ActorScope を構築 |
| Requirements | 4.関数定義 |

**Responsibilities & Constraints**
- `Rule::code_block` を処理し、`code_blocks` ベクタに追加
- 既存の `parse_global_scene_scope` と同じパターンを踏襲

**Dependencies**
- Inbound: parse_file — ファイルパース時に呼び出し (P0)
- Outbound: ActorScope — 構築結果 (P0)
- Internal: parse_code_block — コードブロックパース (P0)

**Contracts**: Service [x]

##### Service Interface

```rust
/// parse_actor_scope の修正箇所
fn parse_actor_scope(pair: Pair<Rule>) -> Result<ActorScope, ParseError> {
    // ...existing code...
    let mut code_blocks = Vec::new();  // 【新規】
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            // ...existing cases...
            Rule::code_block => {  // 【新規】
                code_blocks.push(parse_code_block(inner)?);
            }
            _ => {}
        }
    }
    
    Ok(ActorScope {
        name,
        attrs,
        words,
        var_sets,
        code_blocks,  // 【新規】
        span,
    })
}
```

- Preconditions: `pair` は `Rule::actor_scope` である
- Postconditions: `ActorScope` に全 `code_block` が含まれる
- Invariants: パースエラー時は `ParseError` を返す

---

### pasta_lua/Transpiler

#### generate_actor（修正）

| Field | Detail |
|-------|--------|
| Intent | ActorScope をLua配列形式 + コードブロック展開で出力 |
| Requirements | 2, 4.関数定義 |

**Responsibilities & Constraints**
- 単語定義を `{ [=[値1]=], [=[値2]=] }` 配列形式で出力
- コードブロックをアクター定義ブロック内に展開
- 文字列リテラルは `StringLiteralizer` を使用

**Dependencies**
- Inbound: transpile — トランスパイル時に呼び出し (P0)
- Outbound: Luaファイル — 生成結果 (P0)
- Internal: StringLiteralizer — 文字列リテラル化 (P1)

**Contracts**: Service [x]

##### Service Interface

```rust
/// generate_actor の修正箇所
pub fn generate_actor(&mut self, actor: &ActorScope) -> Result<(), TranspileError> {
    self.writeln("do")?;
    self.indent();
    
    self.writeln(&format!(
        "local ACTOR = PASTA.create_actor(\"{}\")",
        actor.name
    ))?;
    
    // 【修正】配列形式で全値を出力
    for word_def in &actor.words {
        let values: Vec<String> = word_def.words.iter()
            .map(|w| StringLiteralizer::literalize_with_span(w, &word_def.span))
            .collect::<Result<Vec<_>, _>>()?;
        
        if !values.is_empty() {
            let array_literal = format!("{{ {} }}", values.join(", "));
            self.writeln(&format!("ACTOR.{} = {}", word_def.name, array_literal))?;
        }
    }
    
    // 【新規】コードブロック展開
    for code_block in &actor.code_blocks {
        if code_block.language == "lua" {
            self.write_blank_line()?;
            for line in code_block.code.lines() {
                self.writeln(line)?;
            }
        }
    }
    
    self.dedent();
    self.writeln("end")?;
    self.write_blank_line()?;
    
    Ok(())
}
```

- Preconditions: `actor` は有効な `ActorScope`
- Postconditions: Lua配列形式 + コードブロックが出力される
- Invariants: 空の `words` は出力しない

**出力例**

```lua
do
    local ACTOR = PASTA.create_actor("さくら")
    ACTOR.通常 = { [=[\s[0]]=], [=[\s[100]]=] }
    ACTOR.照れ = { [=[\s[1]]=] }
    
    function ACTOR.時刻(act, ...)
        local hour = os.date("%H")
        if hour < 12 then return "おはよう"
        elseif hour < 18 then return "こんにちは"
        else return "こんばんは"
        end
    end
end
```

---

### pasta_lua/Runtime

#### PROXY:word（修正）

| Field | Detail |
|-------|--------|
| Intent | 6レベルフォールバック検索とランダム選択を実行 |
| Requirements | 3, 4 |

**Responsibilities & Constraints**
- 関数検索（完全一致）→ 辞書検索（前方一致）の順序を各スコープで実行
- アクター → シーン → グローバル の順でスコープ検索
- 配列からのランダム選択に `math.random()` を使用

**Dependencies**
- Inbound: シーン実行 — 単語参照時に呼び出し (P0)
- Outbound: search_word_prefix — 前方一致検索 (P1)

**Contracts**: Service [x]

##### Service Interface

```lua
--- 単語置換（6レベルフォールバック検索）
--- @param name string 単語名（＠なし）
--- @return string|nil 見つかった単語、またはnil
function PROXY:word(name)
    -- Level 1: アクター関数（完全一致）
    local actor_fn = rawget(self.actor, name)
    if type(actor_fn) == "function" then
        return actor_fn(self.act)
    end
    
    -- Level 2: アクター辞書（前方一致）
    local actor_words = PASTA.search_word_prefix("actor", self.actor.name, name)
    if actor_words then
        return actor_words[math.random(#actor_words)]
    end
    
    -- Level 3: シーン関数（完全一致）
    local scene = self.act.current_scene
    if scene then
        local scene_fn = rawget(scene, name)
        if type(scene_fn) == "function" then
            return scene_fn(self.act)
        end
        
        -- Level 4: シーン辞書（前方一致）
        local scene_words = PASTA.search_word_prefix("scene", scene.name, name)
        if scene_words then
            return scene_words[math.random(#scene_words)]
        end
    end
    
    -- Level 5: グローバル関数（完全一致）
    local global_fn = PASTA.get_global_function(name)
    if global_fn then
        return global_fn(self.act)
    end
    
    -- Level 6: グローバル辞書（前方一致）
    local global_words = PASTA.search_word_prefix("global", nil, name)
    if global_words then
        return global_words[math.random(#global_words)]
    end
    
    return nil
end
```

- Preconditions: `name` は空でない文字列
- Postconditions: 見つかった場合は文字列、見つからない場合は nil
- Invariants: 検索順序は常に 1→2→3→4→5→6

---

### pasta_lua/FFI

#### search_word_prefix（新規）

| Field | Detail |
|-------|--------|
| Intent | 前方一致で単語辞書を検索し、候補配列を返す |
| Requirements | 4 |

**Responsibilities & Constraints**
- スコープ（actor/scene/global）と名前で辞書を特定
- キーの前方一致で候補を検索
- 全候補の値をフラット配列で返す

**Dependencies**
- Inbound: PROXY:word — Luaから呼び出し (P0)
- External: WordDefRegistry — 単語辞書参照 (P0)

**Contracts**: Service [x]

##### Service Interface

```rust
/// Luaから呼び出し可能なヘルパー関数
fn search_word_prefix(
    lua: &Lua,
    scope: String,      // "actor" | "scene" | "global"
    scope_name: Option<String>,  // actor名 or scene名（globalはNone）
    key_prefix: String, // 検索キー
) -> LuaResult<Option<Vec<String>>> {
    // 前方一致検索を実行
    // 見つかった場合は値の配列を返す
    // 見つからない場合は None
}
```

- Preconditions: `scope` は "actor", "scene", "global" のいずれか
- Postconditions: 前方一致した候補の値配列、または None
- Invariants: 検索はRadix Trieで効率的に実行

**Implementation Notes**
- 既存の `WordDefRegistry` または新規のアクター辞書構造を使用
- mlua の関数登録パターンに従う
- 設計フェーズでは詳細実装は決定しない（タスクフェーズで詳細化）

---

## Data Models

### Domain Model

```mermaid
classDiagram
    class ActorScope {
        +String name
        +Vec~Attr~ attrs
        +Vec~KeyWords~ words
        +Vec~VarSet~ var_sets
        +Vec~CodeBlock~ code_blocks
        +Span span
    }
    
    class KeyWords {
        +String name
        +Vec~String~ words
        +Span span
    }
    
    class CodeBlock {
        +String language
        +String code
        +Span span
    }
    
    ActorScope "1" *-- "*" KeyWords : contains
    ActorScope "1" *-- "*" CodeBlock : contains
```

### Logical Data Model

**Lua側のデータ構造**:

```lua
-- アクター属性（配列形式）
ACTOR.通常 = { [=[\s[0]]=], [=[\s[100]]=] }
ACTOR.照れ = { [=[\s[1]]=] }

-- アクター関数（完全一致で呼び出し）
function ACTOR.時刻(act, ...)
    return "こんにちは"
end
```

**検索用インデックス**（Rust側）:
- アクター辞書: `actor_name -> key_prefix -> values[]`
- シーン辞書: `scene_name -> key_prefix -> values[]`（既存）
- グローバル辞書: `key_prefix -> values[]`（既存）

---

## Error Handling

### Error Categories and Responses

| Category | Error | Response |
|----------|-------|----------|
| パースエラー | 不正なコードブロック構文 | `ParseError::InvalidCodeBlock` |
| トランスパイルエラー | 文字列リテラル化失敗 | `TranspileError::StringLiteral` |
| ランタイムエラー | 単語が見つからない | `nil` を返す（エラーではない） |

---

## Testing Strategy

### Unit Tests

| テスト | 対象 | 検証内容 |
|--------|------|----------|
| parse_actor_scope_with_code_block | parse_actor_scope | code_block を含む actor_scope が正しくパースされる |
| generate_actor_array_output | generate_actor | 複数値が配列形式で出力される |
| generate_actor_code_block | generate_actor | コードブロックが展開される |

### Integration Tests

| テスト | 対象 | 検証内容 |
|--------|------|----------|
| actor_word_random_selection | トランスパイル + ランタイム | 配列からランダム選択が動作する |
| actor_fallback_6_levels | PROXY:word | 6レベル検索が優先順位通りに動作する |
| actor_function_exact_match | PROXY:word | 関数が完全一致で呼び出される |
| actor_dictionary_prefix_match | PROXY:word + Rustヘルパー | 辞書が前方一致で検索される |

### 網羅テストケース（設計詳細）

以下のテストケースで6レベル検索を完全網羅する：

**テストフィクスチャ: `comprehensive_fallback_test.pasta`**

```pasta
# グローバル単語定義
＠天気：雨、雪、台風
＠挨拶：こんにちは、おはよう

# アクター定義
％さくら
　＠天気：晴れ、曇り
　＠表情：\s[0]、\s[1]
```lua
function ACTOR.時刻(act)
    return "朝"
end
```

％うにゅう
　＠表情：\s[10]、\s[11]

# シーン定義
＊テスト
　＠季節：春、夏
```lua
function SCENE.日付(ctx)
    return "1月1日"
end
```
　％さくら、うにゅう
　　さくら：テスト
```

**テストマトリクス**:

| テストID | 呼び出し | 期待レベル | 期待結果 |
|----------|----------|------------|----------|
| T1 | さくら.word("時刻") | L1 アクター関数 | "朝" |
| T2 | さくら.word("天気") | L2 アクター辞書 | "晴れ" or "曇り" |
| T3 | さくら.word("日付") | L3 シーン関数 | "1月1日" |
| T4 | さくら.word("季節") | L4 シーン辞書 | "春" or "夏" |
| T5 | さくら.word("挨拶") | L6 グローバル辞書 | "こんにちは" or "おはよう" |
| T6 | うにゅう.word("天気") | L6 グローバル辞書 | "雨" or "雪" or "台風" |
| T7 | うにゅう.word("表情") | L2 アクター辞書 | "\s[10]" or "\s[11]" |
| T8 | さくら.word("存在しない") | - | nil |
| T9 | さくら.word("天") | L2 前方一致 | "晴れ" or "曇り"（天気にマッチ） |

**注意**: テストの省略は禁止。全テストケースを実装完了すること。

---

## 改訂履歴

| 日付 | 内容 |
|------|------|
| 2026-01-23 | 設計ドキュメント初版作成 |
