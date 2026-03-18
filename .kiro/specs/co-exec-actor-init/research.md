# Research & Design Decisions

## Summary
- **Feature**: `co-exec-actor-init`
- **Discovery Scope**: Extension（既存システムの不具合修正＋インターフェース簡素化）
- **Key Findings**:
  - 根本原因は Rust 側の `toml_to_lua` が TOML テーブルキーを値テーブル内に `name` として注入しない点にある
  - `BUILDER.build` のインターフェースは直接変更方式に移行可能（テスト再現性は入力テーブル制御で担保）
  - Lua 側 `@pasta_log` モジュールが Rust `tracing` へのブリッジとして利用可能

## Research Log

### TOML → Lua 変換における `[actor]` セクションの構造

- **Context**: CONFIG由来アクターの `name` フィールド欠落の原因調査
- **Sources Consulted**: `module_registry.rs` L45-60, L138-162 (`toml_to_lua` 関数)
- **Findings**:
  - `register_config_module` は `custom_fields` 全体を `toml_to_lua` で再帰変換している
  - `toml_to_lua` は `Table(t)` に対して各 `(k, v)` ペアを `table.set(k, toml_to_lua(v))` で変換
  - `[actor."さくら"]` → Lua `CONFIG.actor["さくら"] = { spot = 0, default_surface = 0 }`
  - TOML キー「さくら」は Lua テーブルのキーとして使われるが、値テーブル内に `name` フィールドとして注入されない
- **Implications**: `register_config_module` 内で `[actor]` セクション専用の後処理を追加し、各サブテーブルに `name = キー名` を注入する必要がある。汎用的な `toml_to_lua` は変更しない。

### `@pasta_log` Lua → Rust tracing ブリッジ

- **Context**: Req 2（診断ログ）で使用するログ基盤の確認
- **Sources Consulted**: `runtime/log.rs`
- **Findings**:
  - `require "@pasta_log"` で取得可能
  - `log.trace/debug/info/warn/error(value)` の5段階
  - Lua コールスタック情報（ソースファイル、行番号、関数名）を自動キャプチャ
  - テーブル値は JSON 変換される（`MAX_TABLE_ELEMENTS = 1000`, `MAX_NESTING_DEPTH = 10`）
- **Implications**: `sakura_builder.lua` から直接 `require "@pasta_log"` して `log.warn()` で呼び出せる

### BUILDER.build のインターフェース簡素化

- **Context**: 議題1で合意済みの設計変更
- **Sources Consulted**: `sakura_builder.lua` L49-130, `shiori/act.lua` L68-80
- **Findings**:
  - 現行: シャローコピー → 内部 `actor_spots` 変更 → `(script, actor_spots)` 2値返却 → 呼び出し元で `STORE.actor_spots` に書き戻し
  - 変更後: `input_actor_spots` を直接変更 → `script` のみ返却 → 書き戻し不要
  - テスト再現性は入力テーブルの制御で担保（新テーブルを渡せば副作用なし）
- **Implications**: `sakura_builder.lua` の L56-60（コピーループ）削除、L130 の第2返却値削除、`shiori/act.lua` の書き戻し処理削除

## Design Decisions

### Decision: name 注入の実装箇所

- **Context**: CONFIG由来アクターに `name` フィールドが欠落している問題の修正箇所
- **Alternatives Considered**:
  1. Lua 側 `ACTOR.get_or_create` で既存エントリ正規化
  2. Lua 側 `store.lua` 初期化時に正規化
  3. Rust 側 `register_config_module` で TOML 変換時に注入
- **Selected Approach**: Option 3 — Rust 側で注入
- **Rationale**: データ提供側（Rust）の責務。Lua 側で後付け正規化するよりデータソースで正しいデータを生成するのが自然。`[actor]` セクション固有の対応で十分（汎用化不要）
- **Trade-offs**: Rust コード変更を伴うが、Lua 側のワークアラウンドが不要になる
- **Follow-up**: 既存テスト `config_actors_initialization_test.rs` で `name` フィールド存在を検証するテスト追加

### Decision: BUILDER.build 直接変更方式

- **Context**: `actor_spots` テーブルの更新方式
- **Selected Approach**: 入力テーブル直接変更、スクリプト文字列のみ返却
- **Rationale**: 「純粋性はテストにおける再現性が維持できれば良い」（開発者合意）。コピー + 書き戻しパターンは無駄な間接層
- **Trade-offs**: 関数の副作用が発生するが、Lua の慣例（テーブルは参照渡し）に合致

### Decision: 診断ログのトリガーポイント

- **Context**: スコープ未設定アクターの検出
- **Selected Approach**: `BUILDER.build` 内で `actor_spots[actor_name]` が `nil` のフォールバック発動時に warn
- **Rationale**: 実際の障害点に直結。`％` 省略時の毎回ログではノイズになる
- **Trade-offs**: `％` 省略の意図的なケースでもフォールバック時に warn が出るが、スコープ設定済みなら出ない

## Risks & Mitigations

- **Rust 側変更の影響範囲**: `toml_to_lua` は汎用関数であり変更しない。`register_config_module` 内に `[actor]` セクション専用の後処理を追加することで影響を局所化
- **既存テストへの影響**: `name` フィールド追加により既存テストの期待値が変わる可能性がある。`config_actors_initialization_test.rs` のテスト更新が必要
- **`BUILDER.build` インターフェース変更**: 返却値の変更により、呼び出し元 `shiori/act.lua` の書き戻し処理の同時修正が必要

## References

- `crates/pasta_lua/src/runtime/module_registry.rs` — CONFIG モジュール登録
- `crates/pasta_lua/src/runtime/log.rs` — `@pasta_log` モジュール
- `crates/pasta_lua/scripts/pasta/shiori/sakura_builder.lua` — BUILDER.build 実装
- `crates/pasta_lua/scripts/pasta/shiori/act.lua` — SHIORI_ACT_IMPL.build 実装
- `crates/pasta_lua/scripts/pasta/store.lua` — STORE 初期化
- `crates/pasta_lua/tests/loader/config_actors_initialization_test.rs` — 既存テスト
