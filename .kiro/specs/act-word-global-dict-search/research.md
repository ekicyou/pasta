# Research & Design Decisions

## Summary
- **Feature**: `act-word-global-dict-search`
- **Discovery Scope**: Extension（既存システムの修正）
- **Key Findings**:
  1. `WordTable::collect_word_candidates` と `SceneTable::collect_scene_candidates` の両方に自動フォールバック（ローカル→グローバル）が実装されている
  2. フォールバック廃止に伴い、既存テストケース（`test_collect_*_fallback_*`）の修正が必要
  3. `finalize.rs::collect_words()` にアクター単語辞書の収集処理が欠落している

## Research Log

### Rust側フォールバック実装の現状分析
- **Context**: 要件1でRust側フォールバック廃止が必要
- **Sources Consulted**: 
  - [word_table.rs#L90-200](../../../crates/pasta_core/src/registry/word_table.rs)
  - [scene_table.rs#L355-420](../../../crates/pasta_core/src/registry/scene_table.rs)
- **Findings**: 
  - `collect_word_candidates()`: module_name非空の場合、ローカル検索→グローバル検索のフォールバック実装
  - `collect_scene_candidates()`: 同様のフォールバック実装
  - フォールバック条件: ローカル検索結果が0件の場合のみグローバル検索を実行
- **Implications**: 
  - `module_name` の値によって検索スコープを切り替えるシンプルな修正で対応可能
  - 既存テストケース6件程度の修正が必要

### Lua側現行実装の分析
- **Context**: 要件2, 3でLua側実装を修正
- **Sources Consulted**:
  - [act.lua#L88-96](../../../crates/pasta_lua/scripts/pasta/act.lua)
  - [actor.lua#L160-220](../../../crates/pasta_lua/scripts/pasta/actor.lua)
- **Findings**:
  - `ACT_IMPL.word()`: 現在は `scene[name]` の完全一致のみ実装、TODO コメントでRust統合予定
  - `PROXY_IMPL.word()`: 6レベルフォールバック実装、`search_prefix_lua()` と `math.random` 使用
  - `PROXY_IMPL.word()` 内の L2-L6 が削除対象
- **Implications**:
  - `ACT_IMPL.word()` は新規実装に近い
  - `PROXY_IMPL.word()` は大幅削除・簡略化

### アクター単語辞書収集の欠落
- **Context**: 要件4でfinalize.rs修正が必要
- **Sources Consulted**:
  - [finalize.rs#L85-150](../../../crates/pasta_lua/src/runtime/finalize.rs)
  - [word.lua#L85-95](../../../crates/pasta_lua/scripts/pasta/word.lua)
- **Findings**:
  - `WORD.get_all_words()` は `{ global, local, actor }` を返す
  - `collect_words()` は `global` と `local` のみ処理、`actor` は未処理
  - `build_word_registry()` に `register_actor()` 呼び出しを追加する必要あり
- **Implications**:
  - `WordCollectionEntry` 構造体に `actor_name: Option<String>` フィールド追加
  - または `is_actor: bool` フラグ追加

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A. Lua側フォールバック | Lua側で個別にSEARCH API呼び出し | 柔軟なフォールバック制御、Rust側シンプル化 | Lua側のコード量増加 | 採用 |
| B. Rust APIオプション | `search_word(key, scope, fallback: bool)` | 後方互換性維持 | API複雑化、内部ロジック分岐 | 不採用 |
| C. 別API追加 | `search_word_local()`, `search_word_global()` | 明示的、テスト容易 | API数増加 | 不採用 |

## Design Decisions

### Decision: フォールバック制御をLua側に移行

- **Context**: 現在Rust側で自動フォールバック（ローカル→グローバル）を実装しているが、アクター辞書検索時に意図しないグローバルフォールバックが発生する問題
- **Alternatives Considered**:
  1. Rust APIにオプション追加
  2. 別APIを追加
- **Selected Approach**: Lua側で個別にスコープ指定してAPI呼び出し
- **Rationale**: 
  - フォールバックロジックがLua側で明示的に制御可能
  - Rust側APIがシンプルに保たれる
  - 将来的なフォールバック順序変更が容易
- **Trade-offs**: 
  - Lua側のコード量が若干増加
  - 複数回のFFI呼び出しによる微小なオーバーヘッド
- **Follow-up**: パフォーマンステストでFFIオーバーヘッドを検証

### Decision: アクター辞書キー形式の維持

- **Context**: アクター辞書の検索キー形式
- **Selected Approach**: 既存の `:__actor_xxx__:key` 形式を維持
- **Rationale**: 
  - `WordDefRegistry::register_actor()` API既存
  - `WordTable` がこのキー形式に対応済み
- **Trade-offs**: Lua側でキー形式変換が必要

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| 既存テスト失敗 | フォールバック関連テストを新仕様に合わせて修正 |
| 後方互換性破壊 | 既存の動作を保つようLua側フォールバックを正確に実装 |
| パフォーマンス低下 | FFI呼び出し回数増加は最大3回程度、許容範囲内 |

## References
- [WordTable実装](../../../crates/pasta_core/src/registry/word_table.rs)
- [SceneTable実装](../../../crates/pasta_core/src/registry/scene_table.rs)
- [SearchContext実装](../../../crates/pasta_lua/src/search/context.rs)
- [finalize.rs実装](../../../crates/pasta_lua/src/runtime/finalize.rs)
