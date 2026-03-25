# Research & Design Decisions

## Summary
- **Feature**: `fn-call-expr-stmt`
- **Discovery Scope**: Extension（既存システムの機能拡張）
- **Key Findings**:
  1. `FnScope::Global` は element_gen.rs の3箇所で `"SCENE."` にハードコードされており、全て `"GLOBAL."` への変更が必要
  2. PEG 文法の `set` ルール内に `id` があるため、`var_set_none` 追加には `id` の位置移動が必要だが、`set` はサイレントルール（`=_{ }`）のため pest の子ノード構造への影響なし
  3. `VarSet.name` を `Option<String>` にする最小変更で AST・コード生成・LSP の全層を対応可能

## Research Log

### PEG 文法の `set` ルール構造分析

- **Context**: `var_set_none = { var_marker ~ set }` を追加するには、現在 `set` 内にある `id` を親ルールに移動する必要がある
- **Sources**: `crates/pasta_dsl/src/parser/grammar.pest` L85-89
- **Findings**:
  - 現行: `set =_{ id ~ s ~ set_marker ~ s ~ ( expr | word_ref ) }`
  - `set` はサイレントルール（`=_{ }`）: 子ノードは親に昇格される
  - `parse_var_set()` は `pair.into_inner()` で `Rule::id` を探索するが、`id` が `set` 由来でも `var_set_local` 由来でも、サイレントルール展開により同一視される
  - `set_marker` も `equals` もサイレントルール → 出力に影響なし
- **Implications**:
  - `id` を `set` → `var_set_local`/`var_set_global` に移動しても、`parse_var_set()` のロジックは変更不要
  - `var_set_none` では `Rule::id` が出現しないため、`name` フィールドが空のままになる

### parse_var_set() の互換性検証

- **Context**: `id` の位置移動による既存パース処理への影響を確認
- **Sources**: `crates/pasta_dsl/src/parser/parse_elements.rs` L110-175
- **Findings**:
  - `parse_var_set(pair)` は `pair.into_inner()` のイテレータで各子ノードの `Rule` を判別
  - `Rule::id` → 最初の出現を変数名として取得
  - `Rule::word_ref` → `SetValue::WordRef` を生成
  - その他 → `try_parse_expr()` で式として解析
  - `set` がサイレントルールのため、子の `Rule::id` は `var_set_local`/`var_set_global` の直接の子として見える
  - 文法変更後も同じ `Rule::id` が同じ位置に出現 → **処理は完全互換**
- **Implications**: `parse_var_set()` は `var_set_local`/`var_set_global` に対して無変更で動作する

### FnScope::Global のコード生成パス

- **Context**: `SCENE.` → `GLOBAL.` への変更範囲を特定
- **Sources**: `crates/pasta_lua/src/code_gen/element_gen.rs` L183, L244, L312
- **Findings**:
  - **L183-184**: `Action::FnCall` — アクション行内の関数呼び出し（`act.actor:talk(tostring(PREFIX.fn(act)))`）
  - **L244-245**: `Expr::FnCall` in `generate_expr()` — `self.write_raw()` で直接出力（変数代入の右辺など）
  - **L312-313**: `Expr::FnCall` in `generate_expr_to_buffer()` — バッファに書き込み（`tostring()` ラッパー内等）
  - 3箇所すべてが同一パターン: `let prefix = match scope { Local => "SCENE.", Global => "SCENE." }`
  - コメント "Same for now" あり — 実装予定のプレースホルダー
- **Implications**: 3箇所の `FnScope::Global => "SCENE."` を `"GLOBAL."` に変更するのみ

### write_header() とスナップショット影響

- **Context**: `local GLOBAL = require "pasta.global"` ヘッダー追加の影響範囲
- **Sources**: `crates/pasta_lua/src/code_gen/mod.rs` L84-88, `tests/transpiler/snapshots/*.snap`
- **Findings**:
  - 現行ヘッダー: `local PASTA = require "pasta"` + 空行
  - 全スナップショットの L5 に `local PASTA = require "pasta"` が存在
  - GLOBAL ヘッダー追加後: `local PASTA = require "pasta"\nlocal GLOBAL = require "pasta.global"\n` + 空行
  - `insta` で `cargo test -- --ignored` + `cargo insta review` で一括更新可能
  - `.cache_version` 機構により Lua 中間コードの後方互換性は不要（議題1クローズ済み）
- **Implications**: 全スナップショット（20+件）の一括更新が必要だが、`insta` ワークフローで自動化

### LSP セマンティックトークン影響

- **Context**: `var_set_none` に対する LSP のトークン化処理
- **Sources**: `crates/pasta_lsp/src/analysis/visitors.rs` L145-210
- **Findings**:
  - `visit_var_set()` → `tokenize_var_set_text()` で以下を出力:
    1. マーカートークン (`＄`/`$`, optionally `＄＊`/`$*`)
    2. 変数名トークン (`vs.name`)
    3. 代入演算子トークン (`＝`/`=`)
    4. 値トークン（式 or 単語参照）
  - `var_set_none` では変数名がない → ステップ 2 をスキップ
  - マーカー `＄` の直後に `＝` が続く → ステップ 1, 3, 4 のみ
- **Implications**: `vs.name` が `None` の場合にステップ 2 をスキップする条件分岐を追加

### TextMate 文法影響

- **Context**: VSCode 拡張のシンタックスハイライトへの影響
- **Sources**: `editors/vscode/syntaxes/pasta.tmLanguage.json` L94
- **Findings**:
  - 行レベルパターン: `^(\s*)([＄$])(.+)$` — `＄` で始まる行全体をキャプチャ
  - `＄＝＠fn()` はこのパターンにマッチする（`＄` の後に `＝＠fn()` がキャプチャされる）
  - インラインパターン: `[＄$][＊*]?([^＠@＄$\\\\\\s]+)` — 変数参照用（`var_set_none` には無関係）
- **Implications**: TextMate 文法の変更は不要。既存パターンが `＄＝expr` を自然に認識する

## Architecture Pattern Evaluation

| Option                                                    | Description                                            | Strengths                                                             | Risks / Limitations                                                  |
| --------------------------------------------------------- | ------------------------------------------------------ | --------------------------------------------------------------------- | -------------------------------------------------------------------- |
| A: `VarSet.name` を `Option<String>` に変更               | 既存 `VarSet` 構造体の `name` フィールドを `Option` 化 | 最小変更、`LocalSceneItem` 不変、全層で `name.is_none()` チェックのみ | `name` 使用箇所の全探索が必要                                        |
| B: 新 `VarSetNone` 構造体 + `LocalSceneItem` 新バリアント | 専用の構造体とバリアントを追加                         | 型安全、不正な組み合わせ不可                                          | `LocalSceneItem` 変更（R2-AC6 に反する）、match arm 追加が全層に波及 |
| C: `VarSet.name` を空文字列のまま使用                     | `name: ""` の場合を expr stmt として扱う               | コード変更最小                                                        | 暗黙的な規約、型レベルでの保証なし、空文字列変数名との区別不可       |

## Design Decisions

### Decision: `VarSet.name` を `Option<String>` に変更（Option A 採用）

- **Context**: `var_set_none`（変数名なし・式文）を既存 `VarSet` 型で表現する方法
- **Alternatives Considered**:
  1. Option A: `name: Option<String>` — 型レベルで「名前なし」を表現
  2. Option B: 新構造体 `VarSetNone` — 型安全だが `LocalSceneItem` 変更が必要
  3. Option C: 空文字列規約 — 暗黙的で危険
- **Selected Approach**: Option A
- **Rationale**:
  - R2-AC6 の「`local_scene_item` への変更不要」を満たす唯一の型安全な選択肢
  - `name.is_none()` で式文判定 → 明確で安全
  - `scope` フィールドは `name.is_none()` 時に参照されないため、既存の `VarScope::Local` デフォルトで問題なし
- **Trade-offs**: `name` 使用箇所（code_gen, LSP）での `Option` ハンドリングが必要
- **Follow-up**: LSP `tokenize_var_set_text()` の変数名トークン スキップロジック実装

### Decision: ヘッダーは常時出力（GLOBAL 使用有無に関わらず）

- **Context**: `local GLOBAL = require "pasta.global"` を毎回出力するか、使用時のみにするか
- **Alternatives Considered**:
  1. 常時出力 — シンプル、キャッシュ機構で互換性問題なし
  2. 使用時のみ出力 — AST 解析で GLOBAL 使用を検出する必要あり
- **Selected Approach**: 常時出力
- **Rationale**: 議題1クローズ済み。`.cache_version` でキャッシュ全クリアされるため中間コード互換性は不要。Lua の `require` は結果をキャッシュするため未使用でもパフォーマンス影響は無視できる。
- **Trade-offs**: 全スナップショットの更新が必要（`insta review` で自動化可能）

## Risks & Mitigations

- **Risk 1: 全スナップショット更新によるレビュー負荷** — `cargo insta review` で一括承認。差分は `local GLOBAL` 行の追加のみで機械的に確認可能
- **Risk 2: `VarSet.name: Option<String>` の変更がコンパイラ全体に波及** — `name` の使用箇所はコード生成（element_gen.rs）と LSP（visitors.rs）の2箇所のみ。型チェッカが未対応箇所を検出
- **Risk 3: `set` ルールからの `id` 移動が予期しないパース結果を生む** — サイレントルール展開により visible children は不変。既存テスト（950+件）でリグレッション検出

## References

- [pest 2.x サイレントルール](https://pest.rs/book/grammars/syntax.html#silent-and-atomic-rules) — `=_{ }` ルールの子ノード昇格動作
- [insta スナップショットテスト](https://insta.rs/) — `cargo insta review` での一括更新フロー
- 議題1クローズ: `.cache_version` 機構（`crates/pasta_lua/src/loader/cache.rs`）
- 議題2クローズ: `word_ref` 許容（パターンB）
- 追加議題クローズ: `＄＝expr` 構文確定（`var_marker` 全バリアント統一）
