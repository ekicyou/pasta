# リサーチ & 設計判断ログ

## サマリー
- **機能**: dynamic-call-variable（`＞(id | expr)` コールターゲット拡張）
- **ディスカバリースコープ**: Extension（既存システムの拡張）
- **主要な知見**:
  1. `id` と `expr` の先頭文字集合は完全に素集合 — PEG `(id | expr)` に**曖昧性なし**
  2. `Expr` AST・`generate_expr()`・`try_parse_expr()` は全て完全実装済み — **新規ロジック最小**
  3. `ACT_IMPL.call` は任意文字列 key を既に受け付ける — **Lua ランタイム構造変更不要**

## リサーチログ

### PEG 文法の (id | expr) 曖昧性分析

- **調査動機**: `call_scene = { call_marker ~ (id | expr) ~ s ~ args? }` に変更した場合、PEG パーサーが `id` と `expr` を正しく区別できるか確認
- **情報源**: `grammar.pest` 内の `id`, `expr`, `term` ルール定義
- **知見**:
  - `id` の先頭: Unicode XID_START（日本語文字、英字、`_`）
  - `expr` → `term` の先頭候補:
    - `var_ref`: `＄` / `$`（U+FF04 / U+0024）— XID_START に含まれない
    - `fn_call`: `＠` / `@`（U+FF20 / U+0040）— XID_START に含まれない
    - `paren_expr`: `（` / `(`（U+FF08 / U+0028）— XID_START に含まれない
    - `number_literal`: `0-9`, `-`（数字・ハイフン）— XID_START に含まれない
    - `string_literal`: `「`（U+300C）, `"`（U+0022）— XID_START に含まれない
  - **結論**: `id ∩ expr = ∅` — Pest PEG の ordered choice で**常に一意に決定**
- **設計への影響**: `(id | expr)` の ordered choice で問題なし。`id` を先にマッチさせ、失敗時に `expr` にフォールバック

### CallScene.target 型変更の影響範囲

- **調査動機**: `CallScene.target: String` → `CallScene.target: CallTarget` 型変更の影響箇所を特定
- **情報源**: `grep_search` による `CallScene` 参照検索
- **知見**: 影響箇所は以下の6箇所:
  1. `parse_action.rs:6-23` — `parse_call_scene()`: `target = inner.as_str()` を分岐に変更
  2. `element_gen.rs:52-98` — `generate_call_scene()`: `&call_scene.target` を `match` に変更
  3. `scope_gen.rs:274` — `LocalSceneItem::CallScene(call_scene)`: 参照のみ（変更不要）
  4. `scope_gen.rs:251` — `is_callable_item()`: パターンマッチのみ（変更不要）
  5. `visitors.rs:745-750` — LSP `visit_call_scene()`: `cs.span` のみ参照（`target` 不使用・変更不要）
  6. `ast/scene.rs:165` — `LocalSceneItem::CallScene(CallScene)`: 型ラップのみ（変更不要）
- **設計への影響**: 実質的にコード変更が必要なのは `parse_action.rs` と `element_gen.rs` の2ファイルのみ。残りはコンパイラが検出するが変更不要

### generate_expr() の再利用可能性

- **調査動機**: 動的コールのターゲット式を Lua に変換する際、既存の `generate_expr()` がそのまま使えるか確認
- **情報源**: `element_gen.rs:208-270` の `generate_expr()` 実装
- **知見**:
  - 全 `Expr` バリアント（Integer, Float, String, BlankString, VarRef, FnCall, Paren, Binary）を処理済み
  - `VarRef::Local` → `var.{name}`, `VarRef::Global` → `save.{name}`, `VarRef::Args(i)` → `args[i+1]`
  - `generate_expr_to_buffer()` ヘルパーも存在（バッファへの出力用）
  - **動的コールのターゲット**: `tostring(generate_expr(expr))` でラップすれば完了
- **設計への影響**: `generate_call_scene()` の Dynamic 分岐で `generate_expr_to_buffer()` → `tostring()` ラップのみ。新規コード生成ロジック不要

### ACT_IMPL.call の nil ガード設計

- **調査動機**: 式評価結果が nil の場合（未定義変数等）、`ACT_IMPL.call` でどう処理するか
- **情報源**: `act.lua:391-407` の `ACT_IMPL.call` 実装
- **知見**:
  - 現状: `key` が nil の場合、`self:find_scene(nil, ...)` が呼ばれる → 意図しない挙動
  - `tostring(nil)` = `"nil"` — 文字列 `"nil"` でシーン検索される（バグのもと）
  - **修正案**: 関数先頭で `if key == nil then log.warn(...); return nil end` ガードを追加
  - 3行の追加のみで完結
- **設計への影響**: R3-AC5 の実装はランタイム側の最小変更（3行追加）で完結

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク/制限 | 備考 |
|-----------|------|------|------------|------|
| A: 既存拡張 | `CallScene.target` を `CallTarget` 列挙型に変更、既存の `Expr`/`generate_expr()` を再利用 | 最小変更(5ファイル)、型安全、既存ロジック再利用 | `CallScene.target` 型変更の影響波及 | **推奨** — コンパイラが影響範囲を検出 |
| B: 別 AST ノード | `DynamicCallScene` 新規追加 | 既存 CallScene 変更なし | コード重複、受入リスト拡張大 | gap-analysis で評価済み・非推奨 |
| C: 文字列保持 | `CallScene.target` は String のまま、プレフィックスで区別 | 型変更なし | 型安全性喪失 | gap-analysis で評価済み・非推奨 |

## 設計判断

### 判断: `(id | expr)` PEG 順序

- **文脈**: `call_scene = { call_marker ~ (id | expr) ~ s ~ args? }` で `id` と `expr` のどちらを先に試すか
- **検討した代替案**:
  1. `(id | expr)` — `id` 優先
  2. `(expr | id)` — `expr` 優先
- **採用アプローチ**: `(id | expr)` — `id` を先にマッチ
- **理由**: `id` は単一ルール（XID_START で始まる識別子）でマッチ/不マッチが即座に判定可能。`expr` は複合ルール（`term ~ bin*`）で失敗時のバックトラックが深い。`id` 先行で静的コール（大多数のケース）を高速パスにする
- **トレードオフ**: 静的コールが支配的なユースケースで最適。動的コールはバックトラック後の `expr` マッチになるが、先頭文字の素集合性により即座に `id` 不一致 → `expr` へ遷移するため性能影響は無視可能

### 判断: CallTarget 列挙型の設計

- **文脈**: `CallScene.target` を文字列から構造化された型に変更
- **検討した代替案**:
  1. `CallTarget::Static(String) | Dynamic { name, scope: VarScope }` — var_ref 特化
  2. `CallTarget::Static(String) | Dynamic(Expr)` — 任意の式
- **採用アプローチ**: `CallTarget::Static(String) | Dynamic(Expr)`
- **理由**: `Expr` 型を直接保持することで、変数参照だけでなく関数呼び出し・算術式・括弧式等の全てのターゲット式を型安全に表現。将来の式拡張にも自動的に対応
- **トレードオフ**: `Expr` が大きい分、`CallTarget` のメモリサイズが増加するが、AST ノードはトランスパイル中の一時データであり問題なし

### 判断: Lua コード生成の tostring() ラップ

- **文脈**: 動的コールのターゲット式を Lua コードに変換する方法
- **検討した代替案**:
  1. `tostring(expr)` でラップ — 安全だが冗長な場合あり
  2. 式の型に応じて文字列化を分岐 — 最適だが複雑
- **採用アプローチ**: `tostring(generate_expr(expr))`
- **理由**: `tostring()` は Lua の標準関数で、数値・文字列・nil 全てを安全に文字列化。`var_ref` の場合は `tostring(var.name)` となり、変数が文字列の場合も数値の場合も正しく動作。シンプルさを優先
- **トレードオフ**: `var_ref` が既に文字列の場合、`tostring()` は冗長だが Lua ランタイムが最適化するため性能影響なし

## リスク & 軽減策

- `CallScene.target` 型変更によるコンパイルエラー — Rust コンパイラの `exhaustive match` で全漏れ検出。影響箇所は2ファイルのみ（調査済み）
- `(id | expr)` の PEG 解析優先順序ミス — 先頭文字の素集合性を検証済み。`id` が先行しても `expr` ターゲットは正しくパースされる（`id` は XID_START 先頭文字でのみマッチ）
- 既存テスト回帰 — 950+ テスト全パスを確認するだけで十分（既存パスは影響なし）
- `tostring(nil)` = `"nil"` 問題 — `ACT_IMPL.call` の先頭 nil ガードで解決（R3-AC5）

## 参照
- [doc/spec/04-call-spec.md](../../../../doc/spec/04-call-spec.md) — Call 仕様 §4.1 パターン2
- [doc/spec/01-grammar-model.md](../../../../doc/spec/01-grammar-model.md) — 行指向文法・式サポート
- [Pest PEG 公式ドキュメント](https://pest.rs/book/) — 文法ルール記法
- [Unicode XID_START](https://www.unicode.org/reports/tr31/) — 識別子先頭文字の定義
