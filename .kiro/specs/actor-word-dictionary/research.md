# Research & Design Decisions: actor-word-dictionary

---

## Summary

- **Feature**: `actor-word-dictionary`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. grammar.pest は既に `actor_scope_item` に `code_scope` を含むが、パーサーで未処理
  2. `ActorScope` 構造体には `code_blocks` フィールドがないが、`GlobalSceneScope`/`LocalSceneScope` には存在
  3. actor.lua に `PROXY:word()` の4レベル検索骨格が存在し、Rustヘルパー呼び出しの TODO コメントあり

---

## Research Log

### 1. grammar.pest の既存サポート状況

- **Context**: actor_scope 内の code_block サポート状況を確認
- **Sources Consulted**: 
  - [grammar.pest#L220](crates/pasta_core/src/parser/grammar.pest#L220)
- **Findings**:
  ```pest
  actor_scope      =  { actor_line ~ actor_scope_item* }
  actor_scope_item = _{ global_scene_attr_line | global_scene_word_line | var_set_line | code_scope | blank_line }
  code_scope = _{ code_block ~ blank_line* }
  ```
  - grammar.pest では `code_scope` が `actor_scope_item` に含まれている ✅
  - DSL レベルでは既にサポートされている
- **Implications**: パーサー実装 (`parse_actor_scope`) のみ対応必要

### 2. ActorScope AST 構造の現状

- **Context**: ActorScope に code_blocks フィールドが必要かを確認
- **Sources Consulted**:
  - [ast.rs#L259-L266](crates/pasta_core/src/parser/ast.rs#L259-L266)
  - [ast.rs#L333-L344](crates/pasta_core/src/parser/ast.rs#L333-L344) (GlobalSceneScope比較)
- **Findings**:
  ```rust
  // ActorScope（現状）
  pub struct ActorScope {
      pub name: String,
      pub attrs: Vec<Attr>,
      pub words: Vec<KeyWords>,
      pub var_sets: Vec<VarSet>,
      pub span: Span,
      // code_blocks なし ❌
  }
  
  // GlobalSceneScope（参考）
  pub struct GlobalSceneScope {
      // ...
      pub code_blocks: Vec<CodeBlock>,  // あり ✅
      // ...
  }
  ```
- **Implications**: `ActorScope` に `code_blocks: Vec<CodeBlock>` を追加する必要あり

### 3. parse_actor_scope の現状

- **Context**: パーサーが code_block を処理しているか確認
- **Sources Consulted**:
  - [mod.rs#L231-L280](crates/pasta_core/src/parser/mod.rs#L231-L280)
- **Findings**:
  - コメント: `actor_scope_item = _{ ... | blank_line }` と古い情報
  - `Rule::code_block` のマッチ処理がない ❌
  - `GlobalSceneScope` の `parse_global_scene_scope` には `Rule::code_block` 処理あり ✅
- **Implications**: `parse_actor_scope` に `Rule::code_block` 処理を追加

### 4. code_generator.rs のアクター出力

- **Context**: Lua配列形式出力の現状を確認
- **Sources Consulted**:
  - [code_generator.rs#L106-L130](crates/pasta_lua/src/code_generator.rs#L106-L130)
- **Findings**:
  ```rust
  // 現状：first_word のみ使用
  if let Some(first_word) = word_def.words.first() {
      let literal = StringLiteralizer::literalize_with_span(first_word, &word_def.span)?;
      self.writeln(&format!("ACTOR.{} = {}", word_def.name, literal))?;
  }
  ```
  - 全 words をループせず、最初の値のみ出力 ❌
  - 配列形式 `{ [=[値1]=], [=[値2]=] }` ではなく単一値 ❌
- **Implications**: ループで全値を配列形式で出力するよう修正

### 5. actor.lua の PROXY:word メソッド

- **Context**: ランタイムのフォールバック検索実装状況を確認
- **Sources Consulted**:
  - [actor.lua#L57-L78](crates/pasta_lua/scripts/pasta/actor.lua#L57-L78)
- **Findings**:
  ```lua
  function PROXY:word(name)
      -- Level 1: アクターfield
      local actor_value = rawget(self.actor, name)
      if actor_value then
          return actor_value
      end
      -- Level 2: SCENEfield
      local scene = self.act.current_scene
      if scene and scene[name] then
          return scene[name]
      end
      -- Level 3: グローバルシーン名での検索（Rust関数呼び出し予定）
      -- Level 4: 全体検索（Rust関数呼び出し予定）
      return nil -- TODO: Rust search_word 統合
  end
  ```
  - 4レベル検索の骨格あり ✅
  - ランダム選択ロジックなし ❌
  - 関数/配列の区別なし ❌
  - 前方一致検索なし（TODO コメントあり）❌
- **Implications**: 
  - 関数（完全一致）→ 配列（前方一致）の検索ロジックを追加
  - Rustヘルパー関数で前方一致検索を実装

### 6. word.lua のグローバル/ローカル単語構造

- **Context**: 既存単語辞書パターンを確認
- **Sources Consulted**:
  - [word.lua#L1-L80](crates/pasta_lua/scripts/pasta/word.lua#L1-L80)
- **Findings**:
  ```lua
  -- グローバル単語レジストリ（key → values[][]）
  local global_words = {}
  
  -- ビルダーパターン
  function WordBuilder:entry(...)
      local values = { ... }
      table.insert(self._registry[self._key], values)
      return self
  end
  ```
  - `values[][]` 構造（エントリごとに配列）
  - アクター単語には別の構造が必要（フラット配列 `{ 値1, 値2, ... }`）
- **Implications**: アクター単語は `{ [=[値1]=], [=[値2]=] }` のフラット配列で十分

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| **A: 既存拡張** | ActorScope + parse_actor_scope + generate_actor + actor.lua を修正 | 影響範囲が明確、既存パターンに従う | AST構造変更による互換性リスク | 推奨 |
| B: 新規コンポーネント | actor_word_registry.rs + actor_word.lua を新規作成 | 責務分離が明確 | ファイル数増加、既存との整合性維持コスト | 過剰設計 |
| C: ハイブリッド | フェーズ分割で段階的実装 | 各フェーズで動作確認可能 | 3フェーズの整合性管理 | Option A をフェーズ分割で実装 |

**選択**: Option A（既存拡張）をフェーズ分割で実装

---

## Design Decisions

### Decision: ActorScope へのcode_blocks追加

- **Context**: アクター定義内にLuaコードブロックを含められるようにする
- **Alternatives Considered**:
  1. ActorScope に `code_blocks: Vec<CodeBlock>` 追加
  2. 別構造体 `ActorCodeScope` を新設
- **Selected Approach**: Option 1 - 既存構造体に追加
- **Rationale**: GlobalSceneScope/LocalSceneScope と同じパターンを踏襲し、一貫性を維持
- **Trade-offs**: AST構造変更だが、既存使用箇所は限定的
- **Follow-up**: 下流の code_generator.rs でコードブロック展開を実装

### Decision: Lua配列形式出力

- **Context**: アクター属性を配列形式で出力
- **Alternatives Considered**:
  1. `ACTOR.key = { [=[値1]=], [=[値2]=] }` フラット配列
  2. `ACTOR.key = { {[=[値1]=]}, {[=[値2]=]} }` ネスト配列（word.lua互換）
- **Selected Approach**: Option 1 - フラット配列
- **Rationale**: シンプルで `math.random(#array)` で直接アクセス可能
- **Trade-offs**: word.lua の `values[][]` とは異なる構造だが、用途が違うため問題なし
- **Follow-up**: ランタイムで配列長チェックとランダム選択を実装

### Decision: フォールバック検索のLua/Rust分担

- **Context**: 6レベル検索のロジック配置
- **Alternatives Considered**:
  1. 全てLuaで実装
  2. 全てRust FFIで実装
  3. ハイブリッド（検索フローはLua、個別アルゴリズムはRust）
- **Selected Approach**: Option 3 - ハイブリッド
- **Rationale**: 
  - フォールバックロジック（順序制御）はLuaの方が柔軟
  - 前方一致検索（Radix Trie等）はRustの方が高性能
- **Trade-offs**: Lua/Rust間のインターフェース設計が必要
- **Follow-up**: Rustヘルパー関数 `search_word_prefix(scope, key)` を設計

### Decision: コードブロック言語

- **Context**: actor_scope 内のコードブロック言語識別
- **Alternatives Considered**:
  1. 常にLuaと仮定
  2. 言語識別子（```lua, ```rune 等）を参照
- **Selected Approach**: Option 2 - 言語識別子を参照（ただし現状Luaのみ対応）
- **Rationale**: 将来の拡張性を維持しつつ、現状はLua限定で実装
- **Trade-offs**: パーサーで言語識別子を保持する必要あり（CodeBlock に既に `language` フィールドあり）
- **Follow-up**: generate_actor で `code_block.language == "lua"` をチェック

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| AST構造変更による互換性 | 下流コンポーネントへの影響 | 段階的テスト、既存テストでリグレッション確認 |
| 後方互換性 | 既存スクリプトの動作変更 | 単一値も配列として処理、ランダム選択で同一動作 |
| ランタイム性能 | ランダム選択のオーバーヘッド | O(1)アクセス（Luaテーブル）で問題なし |
| Lua/Rust インターフェース | FFI設計の複雑さ | 既存の mlua パターンを活用 |

---

## References

- [grammar.pest](crates/pasta_core/src/parser/grammar.pest) - DSL文法定義
- [ast.rs](crates/pasta_core/src/parser/ast.rs) - AST構造体定義
- [mod.rs (parser)](crates/pasta_core/src/parser/mod.rs) - パーサー実装
- [code_generator.rs](crates/pasta_lua/src/code_generator.rs) - Luaコード生成
- [actor.lua](crates/pasta_lua/scripts/pasta/actor.lua) - アクターランタイム
- [word.lua](crates/pasta_lua/scripts/pasta/word.lua) - 単語辞書ランタイム
