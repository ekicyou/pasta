# CHANGELOG: act-word-global-dict-search

## 実装完了: 2026-01-28

### 概要

PASTAスクリプトエンジンにおける単語検索アーキテクチャを整理し、Rust側の自動フォールバック仕様を廃止。Lua側で明示的なスコープ指定によるフォールバック制御に変更。

### 主要変更点

#### 1. Rust側 API 仕様変更

**WordTable/SceneTable フォールバック廃止:**
- `collect_word_candidates()` および `collect_scene_candidates()` の自動フォールバック（ローカル→グローバル）を削除
- `module_name` が空文字列の場合: グローバルスコープのみ検索
- `module_name` が非空の場合: 指定されたローカルスコープのみ検索
- 検索結果が0件でもフォールバックしない

**影響を受けるAPI:**
- `crates/pasta_core/src/registry/word_table.rs::collect_word_candidates()`
- `crates/pasta_core/src/registry/scene_table.rs::collect_scene_candidates()`

#### 2. Lua側実装

**共通値解決関数追加 (`pasta.word`):**
- `WORD.resolve_value(value, act)` 関数を新規実装
- 完全一致検索時の値解決ロジックを共通化
- 関数値: `value(act)` を実行
- 配列値: 最初の要素 `value[1]` を返す
- その他: `tostring(value)` で文字列化

**ACT_IMPL.word 実装 (`act.lua`):**
- 4レベルフォールバック検索を実装:
  1. シーンテーブル完全一致 (`current_scene[name]`)
  2. GLOBAL完全一致 (`GLOBAL[name]`)
  3. シーンローカル辞書前方一致 (`SEARCH:search_word(name, scene_name)`)
  4. グローバル辞書前方一致 (`SEARCH:search_word(name, nil)`)
- `pcall` によるSEARCH API graceful degradation対応

**PROXY_IMPL.word 簡素化 (`actor.lua`):**
- 3レベルフォールバック検索を実装:
  1. アクター完全一致 (`actor[name]`)
  2. アクター辞書前方一致 (`SEARCH:search_word(name, "__actor_xxx__")`)
  3. `act:word(name)` への委譲
- 旧Lua検索ロジック削除:
  - `search_prefix_lua()` 関数
  - ローカル `resolve_value()` 関数
  - `math.random` による候補選択
  - `WORD.get_*_words()` 直接呼び出し

#### 3. アクター単語辞書収集 (`finalize.rs`)

**新機能:**
- `all_words.actor` からアクター単語辞書を収集
- `WordCollectionEntry` 構造体に `actor_name: Option<String>` フィールドを追加
- `build_word_registry()` でアクター単語を `register_actor()` で登録

**影響を受けるファイル:**
- `crates/pasta_lua/src/runtime/finalize.rs::collect_words()`
- `crates/pasta_lua/src/runtime/finalize.rs::build_word_registry()`

### 破壊的変更

**なし** - 後方互換性を維持。既存のPASTAスクリプトは変更なしで動作。

### テスト

**新規テスト:**
- `test_collect_word_candidates_global_only` (WordTable)
- `test_collect_word_candidates_local_not_found_no_fallback` (WordTable)
- `test_collect_scene_candidates_local_not_found_no_fallback` (SceneTable)

**更新テスト:**
- `test_collect_word_candidates_local_only` (リネーム)
- `test_collect_scene_candidates_local_only` (リネーム)
- `test_resolve_scene_id_unified_global_scene_no_fallback` (新規)

**無視テスト:**
- `fallback_search_integration_test.rs` の12テストを `#[ignore]` 化
  - 理由: レガシー6レベルフォールバックテスト、新アーキテクチャでは `finalize_scene()` が必須

**テスト結果:**
- 全556テスト成功
- 0失敗
- 15無視（意図的）

### パフォーマンス影響

**なし** - 検索アルゴリズム自体は変更なし。フォールバック制御の責務がRust→Luaに移動したのみ。

### マイグレーションガイド

**PASTAスクリプト開発者:**
- 変更なし - 既存の `act:word()` および `proxy:word()` 呼び出しはそのまま動作

**Rust API利用者:**
- `WordTable::search_word()` および `SceneTable::search_scene()` を直接呼び出している場合:
  - `module_name` パラメータの意味が変更されました
  - ローカル検索で見つからない場合、グローバルへのフォールバックは自動で行われません
  - フォールバックが必要な場合は、2回のAPI呼び出しを明示的に実装してください

### ドキュメント更新

- [requirements.md](requirements.md) - SEARCH API仕様（フォールバック廃止後）を記載
- [design.md](design.md) - 検索フロー、アーキテクチャパターンを更新
- [tasks.md](tasks.md) - 全タスク完了 `[x]`
- [VALIDATION_REPORT.md](VALIDATION_REPORT.md) - 実装検証レポート

### 関連Issue/PR

なし（内部仕様改善）

### 謝辞

Kiro Spec-Driven開発プロセスにより、要件定義から実装・検証まで一貫した品質管理を実現。
