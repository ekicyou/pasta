# Research & Design Decisions

## Summary
- **Feature**: `pasta-scene-dictionary-finalization`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. mluaの`create_function()`と`package.loaded`操作パターンは既存コードで確立済み
  2. `with_config()`から SearchContext 構築を削除し、`finalize_scene()`でのみ構築する方針
  3. Lua側モジュール（`pasta.scene`, `pasta.word`）のテーブル走査は標準 mlua API で実現可能

## Research Log

### Topic 1: Lua関数のRust置換メカニズム

- **Context**: `pasta/init.lua` で定義された `PASTA.finalize_scene()` スタブをRust関数で置き換える方法
- **Sources Consulted**: 
  - `crates/pasta_lua/src/runtime/enc.rs` - 既存の関数バインディング実装
  - `crates/pasta_lua/src/search/mod.rs` - モジュール登録パターン
  - mlua ドキュメント (Luaテーブル操作API)
- **Findings**:
  - `package.loaded["pasta"]` からテーブルを取得し、`finalize_scene` フィールドを Rust 関数で上書き可能
  - `lua.create_function()` で Rust クロージャを Lua 関数に変換
  - enc.rs で `table.set("key", lua.create_function(impl)?)` パターンが確立済み
- **Implications**: 
  - 既存パターンをそのまま適用可能
  - ランタイム初期化時（`with_config()` または `from_loader_with_scene_dic()` 直後）に関数置換を実行

### Topic 2: Luaテーブル走査によるデータ収集

- **Context**: `pasta.scene` および `pasta.word`（新規）レジストリからのデータ収集方法
- **Sources Consulted**:
  - `crates/pasta_lua/scripts/pasta/scene.lua` - 既存シーンレジストリ構造
  - mlua API ドキュメント（Table::pairs(), Table::get()）
- **Findings**:
  - `pasta.scene` 構造: `registry[global_name] = { __global_name__ = global_name, [local_name] = scene_func }`
  - `Table::pairs()` イテレータで全エントリ走査可能
  - `Table::get::<_, T>()` で型安全な値取得が可能
  - `Value::as_function()` で Lua 関数参照を取得
- **Implications**:
  - シーン収集: `pairs()` で `registry` テーブルを走査し、各グローバルテーブル内の関数を列挙
  - 単語収集: 同様のパターンで `pasta.word` レジストリを走査

### Topic 3: SearchContext 初期構築の削除

- **Context**: `with_config()` での SearchContext 構築を削除し、`finalize_scene()` でのみ構築する方針
- **Sources Consulted**:
  - `crates/pasta_lua/src/runtime/mod.rs` (lines 145-189) - `with_config()` 実装
  - `crates/pasta_lua/src/search/mod.rs` - `register()` 関数
- **Findings**:
  - 現在の `with_config()` は `crate::search::register()` を呼び出して SearchContext 構築
  - この呼び出しを削除すると `@pasta_search` モジュールは存在しない状態になる
  - `finalize_scene()` で `register()` を呼び出すことで、Lua側レジストリ収集後に構築可能
- **Implications**:
  - `with_config()` から `crate::search::register()` 呼び出しを削除
  - `finalize_scene()` 実装内で `crate::search::register()` を呼び出し
  - 既存テストには `finalize_scene()` 呼び出しを追加

### Topic 4: シーン名カウンタのLua側実装

- **Context**: Rust側 `SceneRegistry::increment_counter()` をLua側で実現する方法
- **Sources Consulted**:
  - `crates/pasta_core/src/registry/scene_registry.rs` - 既存カウンタロジック
  - `crates/pasta_lua/scripts/pasta/scene.lua` - Lua側シーンモジュール
- **Findings**:
  - 現在の `scene.lua` はカウンタ管理を持たない（単純な登録のみ）
  - Lua側で `local counters = {}` テーブルを追加し、`create_scene()` 時にインクリメント
  - 生成される名前: `base_name .. counter` (例: "メイン1", "メイン2")
- **Implications**:
  - `pasta/scene.lua` にカウンタロジックを追加
  - `pasta/init.lua` の `create_scene()` をカウンタ対応に変更
  - トランスパイラ出力は番号なしの `PASTA.create_scene("メイン")` 形式

### Topic 5: 単語辞書レジストリ設計

- **Context**: 新規 `pasta.word` モジュールの設計
- **Sources Consulted**:
  - `crates/pasta_core/src/registry/word_registry.rs` - Rust側単語レジストリ
  - `crates/pasta_core/src/parser/grammar.pest` - 単語値の文法（カンマ区切り）
- **Findings**:
  - Rust側 `WordDefRegistry` は `key → values[]` のエントリリスト
  - 同じキーで複数登録可能（別エントリとして保持）
  - Lua側もこれを再現: `global_words[key]` = `{values1, values2, ...}` (配列の配列)
  - ローカル単語: `local_words[scene_name][key]` = 同構造
- **Implications**:
  - `pasta/word.lua` を新規作成
  - ビルダーパターン API: `create_word(key):entry(v1, v2, ...)`
  - `finalize_scene()` で収集し `WordDefRegistry` 形式に変換

## Architecture Pattern Evaluation

| Option         | Description                 | Strengths                  | Risks / Limitations               | Notes                        |
| -------------- | --------------------------- | -------------------------- | --------------------------------- | ---------------------------- |
| Phase 1 Inline | `runtime/mod.rs` に直接実装 | 最小変更、既存パターン踏襲 | ファイル肥大化リスク（現在481行） | 推奨: 初期実装として適切     |
| New Module     | `runtime/finalize.rs` 分離  | 責務分離、テスト容易性     | 過剰な構造化                      | Phase 2 リファクタリング候補 |

**選択**: Phase 1 Inline → 実装安定後に必要に応じて分離

## Design Decisions

### Decision 1: SearchContext 初期構築の完全削除

- **Context**: 初期構築と `finalize_scene()` での再構築の二重化問題
- **Alternatives Considered**:
  1. 条件分岐フラグ (`skip_search_context`) で初期構築を制御
  2. `with_config()` から SearchContext 構築を完全削除
- **Selected Approach**: Option 2 - 完全削除
- **Rationale**: 
  - `finalize_scene()` で必ず構築する方針なら状態管理は不要
  - シンプルな実装（削除のみ、条件分岐なし）
- **Trade-offs**: 
  - ✅ コード簡潔化
  - ❌ 既存テストへの影響（`finalize_scene()` 呼び出し追加が必要）
- **Follow-up**: 既存テストを洗い出し、`finalize_scene()` 呼び出しを追加

### Decision 2: シーン名カウンタのLua側管理

- **Context**: トランスパイル時のRust側カウンタをどう置き換えるか
- **Alternatives Considered**:
  1. Lua側でカウンタ管理（`pasta.scene` モジュール拡張）
  2. Rust側で `finalize_scene()` 時にカウンタ再計算
  3. トランスパイラで事前番号付与（現状維持に近い）
- **Selected Approach**: Option 1 - Lua側カウンタ管理
- **Rationale**:
  - 積極的にLua実装に依存する方針に合致
  - パフォーマンス要求発生時にRust移行可能
- **Trade-offs**:
  - ✅ トランスパイラ簡素化
  - ❌ Lua側コードの複雑化（軽微）
- **Follow-up**: `pasta/scene.lua` にカウンタロジック実装

### Decision 3: 単語辞書のビルダーパターンAPI

- **Context**: トランスパイラ出力コードの可読性とLua API設計
- **Alternatives Considered**:
  1. 単純関数呼び出し: `PASTA.register_word(key, {v1, v2, ...})`
  2. ビルダーパターン: `PASTA.create_word(key):entry(v1, v2, ...)`
- **Selected Approach**: Option 2 - ビルダーパターン
- **Rationale**:
  - メソッドチェーンで可読性向上
  - 可変長引数で簡潔な記述
  - 将来的なメタデータ追加に対応可能
- **Trade-offs**:
  - ✅ 直感的なAPI
  - ❌ 実装が若干複雑（ビルダーオブジェクト返却）
- **Follow-up**: `pasta/word.lua` でビルダー実装

### Decision 4: 仕様分割禁止

- **Context**: スコープ拡大に伴う実装戦略
- **Alternatives Considered**:
  1. Phase分割（Phase 1: シーン収集、Phase 2: 単語収集）
  2. 単一サイクルで全要件実装
- **Selected Approach**: Option 2 - 単一サイクル
- **Rationale**:
  - テスト依存性: SearchContext は両辞書が必要
  - アーキテクチャ整合性: トランスパイラ変更とLua収集は一体
  - 部分実装では既存機能が破損
- **Trade-offs**:
  - ✅ テスト可能な状態を維持
  - ❌ 実装規模が大きい
- **Follow-up**: レイヤー単位での順次完成で対応

## Risks & Mitigations

| Risk                                 | Mitigation                                                    |
| ------------------------------------ | ------------------------------------------------------------- |
| Lua ↔ Rust連携の複雑性               | enc.rs の既存パターンを踏襲、プロトタイプで早期検証           |
| 既存テストへの影響                   | テスト一覧を作成し、`finalize_scene()` 呼び出しを計画的に追加 |
| `pasta.word` 新規実装の品質          | 単体テスト充実、既存 `pasta.scene` パターンを踏襲             |
| ファイルサイズ増加（runtime/mod.rs） | 500行超で Phase 2 分離検討                                    |

## References

- [mlua ドキュメント](https://docs.rs/mlua/) - Lua バインディング API
- [pasta_lua/scripts/pasta/scene.lua](../../crates/pasta_lua/scripts/pasta/scene.lua) - 既存シーンレジストリ
- [pasta_lua/src/search/mod.rs](../../crates/pasta_lua/src/search/mod.rs) - 検索モジュール登録
- [pasta_lua/src/runtime/enc.rs](../../crates/pasta_lua/src/runtime/enc.rs) - Rust関数バインディング例
