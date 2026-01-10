# Research & Design Decisions: pasta_search_module

---
**Feature**: pasta_search_module  
**Discovery Scope**: Extension（既存 pasta_core コンポーネントへの mlua バインディング追加）  
**Discovery Type**: Light Discovery  
**作成日**: 2026-01-10  
**言語**: ja（日本語）
---

## Summary

- **Feature**: pasta_search_module - Rust 側検索モジュールの Lua バインディング実装
- **Discovery Scope**: Extension（mlua 経由で既存 pasta_core の SceneTable/WordTable を Lua に公開）
- **Key Findings**:
  1. pasta_core の SceneTable, WordTable, RandomSelector が完全実装済み（要件 100% カバー）
  2. mlua-stdlib の loader/register パターンが参照実装として利用可能
  3. MockRandomSelector が `#[cfg(test)]` 限定 → 公開化が必要
  4. 複数 Lua ランタイムインスタンス対応のため UserData ラッピングを採用

---

## Research Log

### Topic 1: pasta_core 既存 API の確認

**Context**: Requirement 1-5 の実現可能性を検証

**Sources Consulted**:
- `crates/pasta_core/src/registry/scene_table.rs` (791 行)
- `crates/pasta_core/src/registry/word_table.rs` (599 行)
- `crates/pasta_core/src/registry/random.rs` (157 行)

**Findings**:

| API | 機能 | 要件カバレッジ |
|-----|------|---------------|
| `SceneTable::from_scene_registry()` | SceneRegistry → SceneTable 変換 | Req 1 ✅ |
| `SceneTable::resolve_scene_id()` | 前方一致検索 + キャッシュ選択 | Req 2 ✅ |
| `SceneTable::resolve_scene_id_unified()` | ローカル + グローバル統合検索 | Req 2 ✅ |
| `WordTable::from_word_def_registry()` | WordDefRegistry → WordTable 変換 | Req 1 ✅ |
| `WordTable::search_word()` | 2段階プレフィックスマッチ + キャッシュ | Req 3 ✅ |
| `DefaultRandomSelector` | 本番用ランダム選択 | Req 5 ✅ |
| `MockRandomSelector` | 決定的テスト用選択 | Req 8 ⚠️ |

**Implications**:
- 検索ロジックは pasta_core で完全実装済み
- Lua 側は単純なラッパー呼び出しで済む
- **Issue**: MockRandomSelector は `#[cfg(test)]` で限定されている → 公開化が必要

---

### Topic 2: MockRandomSelector の公開化要件

**Context**: Requirement 8 で Lua 側から `set_scene_selector()` で MockRandomSelector に切り替える必要

**Sources Consulted**:
- `crates/pasta_core/src/registry/random.rs` (Lines 70-100)

**Findings**:
```rust
// 現在の実装（#[cfg(test)] 限定）
#[cfg(test)]
pub struct MockRandomSelector { ... }
```

**Implications**:
- **Design 決定**: MockRandomSelector を `#[cfg(test)]` から除外し、公開 API として提供
- 代替案として pasta_lua 内で独自の MockSelector を実装する選択肢もあるが、DRY 原則に反する
- **推奨**: pasta_core で `MockRandomSelector` を公開化（features フラグで制御可能）

---

### Topic 3: mlua-stdlib 実装パターン

**Context**: mlua バインディングの実装パターンを確認

**Sources Consulted**:
- mlua-stdlib GitHub リポジトリ（http, task, regex, json モジュール）
- gap_analysis.md の mlua-stdlib 分析セクション

**Findings**:

**パターン A: 単純関数群**
```rust
fn loader(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;
    t.set("func1", lua.create_function(func1)?)?;
    Ok(t)
}
```

**パターン B: UserData + 関数群**
```rust
impl UserData for LuaType {
    fn add_methods(...) { ... }
}
fn loader(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;
    t.set("Type", lua.create_proxy::<LuaType>()?)?;
    Ok(t)
}
```

**Implications**:
- pasta_search_module は **パターン B** を採用（SearchContext UserData）
- `add_method()` で不変メソッド、`add_method_mut()` で可変メソッドを実装
- メタテーブル設定で `SEARCH:func()` / `SEARCH.func()` 両形式対応

---

### Topic 4: 複数 Lua ランタイムインスタンス対応

**Context**: pasta_lua は複数の独立した Lua ランタイムをサポートする必要

**Sources Consulted**:
- REVIEW_FINDINGS.md 議題 2 の決定
- gap_analysis.md Option A 分析

**Findings**:
- ❌ Static 変数は禁止（インスタンス間で状態が汚染される）
- ✅ UserData ラッピングで各インスタンスが独立した SearchContext を持つ
- ✅ mlua の add_method_mut() で exclusive access が自動確保される

**Implications**:
- SearchContext を UserData として実装
- 各 Lua インスタンスが独立した SceneTable/WordTable を保有
- Interior Mutability（Arc<Mutex<>>）は不要

---

### Topic 5: SceneInfo 直接返却の設計

**Context**: Requirement 2.3 で `(global_name, local_name)` タプルを返す必要

**Sources Consulted**:
- REVIEW_FINDINGS.md 議題 1 の決定
- SceneTable API 分析

**Findings**:
- 現在の `resolve_scene_id()` は `SceneId` のみを返す
- SceneInfo 復元には追加メソッドが必要

**決定事項**:
- pasta_core に `resolve_scene()` メソッドを追加（SceneId ではなく SceneInfo を直接返却）
- または既存の `resolve_scene_id_unified()` の返り値を拡張
- Lua 側で 2 step 呼び出し（resolve → get_info）を避け、1 step で完結

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| **A: UserData ラッピング** | SearchContext を UserData として公開 | 複数インスタンス対応、Static 不要、&mut self 可能 | 学習コスト（mlua API） | ⭐ **採用** |
| B: Arc<Mutex<>> | 共有状態を Mutex でラップ | 参照共有可能 | Deadlock リスク、overhead | ❌ 非採用 |
| C: mlua Registry | mlua 高度レジストリ機能 | 柔軟性 | ドキュメント少、学習コスト大 | ❌ 非採用 |

---

## Design Decisions

### Decision 1: SearchContext を単一 UserData として公開

- **Context**: Lua 側インターフェース設計（議題 2）
- **Alternatives Considered**:
  1. Pattern A: SearchContext 単一 UserData → `SEARCH:search_scene()`
  2. Pattern B: SceneTable/WordTable 別々 UserData → `SEARCH.scene:search()`
- **Selected Approach**: Pattern A
- **Rationale**: シンプル、Lua 側で内部テーブルを露出しない、最初の API 提案と一致
- **Trade-offs**: 内部構造の露出なし（利点）、細粒度アクセス不可（許容）
- **Follow-up**: メタテーブル設定で両形式（`:` / `.`）呼び出し対応

### Decision 2: &mut self 制御に mlua add_method_mut() を使用

- **Context**: Requirement 8 の Selector 切り替え（議題 3）
- **Alternatives Considered**:
  1. Interior Mutability（Arc<Mutex<>>）
  2. mlua add_method_mut()
- **Selected Approach**: add_method_mut()
- **Rationale**: シンプル、Rust 的設計、mlua が exclusive access を保証
- **Trade-offs**: mlua 依存（許容）、Interior Mutability 不要（利点）
- **Follow-up**: なし

### Decision 3: MockRandomSelector の公開化

- **Context**: Requirement 8 で Lua から MockSelector を利用
- **Alternatives Considered**:
  1. pasta_core で MockRandomSelector を公開化
  2. pasta_lua 内で独自 MockSelector 実装
- **Selected Approach**: pasta_core で公開化
- **Rationale**: DRY 原則、既存実装の再利用
- **Trade-offs**: pasta_core API 変更が必要
- **Follow-up**: Design で pasta_core 変更を明記

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| mlua API 学習曲線 | Medium | Low | mlua-stdlib 参照実装を活用 |
| MockRandomSelector 公開化 | Low | Medium | feature フラグで制御（オプション） |
| メタテーブル設定の複雑さ | Low | Low | mlua ドキュメント参照 |

---

## References

- [mlua-stdlib GitHub](https://github.com/mlua-rs/mlua-stdlib) — バインディング実装パターン
- [mlua Documentation](https://docs.rs/mlua/latest/mlua/) — UserData, add_method_mut API
- gap_analysis.md — 既存コード詳細分析
- REVIEW_FINDINGS.md — 設計決定の記録

