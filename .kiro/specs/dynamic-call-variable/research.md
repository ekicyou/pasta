# リサーチ & 設計判断

## サマリー
- **フィーチャー**: `dynamic-call-variable`（動的コール構文 `＞＄変数名`）
- **ディスカバリースコープ**: Extension（既存コンポーネント拡張）
- **主要ファインディング**:
  - PEG 文法の `var_ref` ルールは silent rule（`=_{}`）であり、`call_scene` に組み込むと `var_ref_local` / `var_ref_global` が直接展開される
  - `ACT_IMPL.call` は任意文字列の `key` を既に受け付けており、Lua ランタイムの構造的変更は不要
  - `CallScene` の参照箇所は 37 箇所（うちプロダクションコード 18 箇所）で、`target: String` → `target: CallTarget` への型変更はコンパイラが全検出する

## リサーチログ

### 拡張ポイント分析: `call_scene` PEG ルール
- **コンテキスト**: `call_scene = { call_marker ~ id ~ s ~ args? }` が `id` 固定のため `＞＄変数名` がパースエラーになる
- **調査対象**: grammar.pest の `var_ref` 関連ルール群
- **ファインディング**:
  - `var_ref =_{ var_ref_global | var_ref_local }` — silent rule
  - `var_ref_local = { var_marker ~ var_id ~ s }` — `s`（空白消費）を含む
  - `var_ref_global = { var_marker ~ global_marker ~ id ~ s }` — 同上
  - `call_scene` 自体にも `~ s ~` がある → `var_ref` 内の `~ s` と二重消費の可能性を確認
  - **結論**: `call_scene = { call_marker ~ (var_ref | id) ~ s ~ args? }` で問題ない。`var_ref` 内の `s` は silent rule 内の子ルール（`var_ref_local` / `var_ref_global`）に含まれており、Pest は PEG パーサーとして最長一致で消費する。`id` の場合は外側の `~ s ~` で消費。
- **影響**: PEG ルール1行のみの変更で R1-AC1, R1-AC2, R1-AC3 を同時に充足

### 既存パターン流用: `VarRef` / `VarScope`
- **コンテキスト**: 動的コールの AST 表現を設計するにあたり、既存の変数参照パターンを流用可能か
- **調査対象**: `ast/action.rs` の `Action::VarRef` と `VarScope` 列挙型
- **ファインディング**:
  - `VarScope` は `Local | Global | Args(u8)` の3バリアント
  - 動的コールで使うのは `Local` と `Global` のみ（`Args(u8)` は `$0`, `$1` 等の引数参照）
  - `parse_actions()` 内の `var_ref_local` / `var_ref_global` 処理パターンを `parse_call_scene()` にそのまま流用可能
  - `element_gen.rs` の `Action::VarRef` コード生成パターン（`var.{name}` / `save.{name}`）も流用可能
- **影響**: 新規型は `CallTarget` 列挙型のみ。`VarScope` をそのまま再利用

### `CallScene` 参照箇所の影響範囲
- **コンテキスト**: `CallScene.target` の型変更がコードベース全体に与える影響の定量化
- **調査対象**: プロジェクト全体の `CallScene` 参照37箇所
- **ファインディング**:
  - **直接修正必須** (6箇所):
    - `parse_action.rs`: `parse_call_scene()` — ルール分岐追加
    - `action.rs`: `CallScene` 構造体定義 — `target` 型変更
    - `element_gen.rs`: `generate_call_scene()` — コード生成分岐
    - `visitors.rs`: `visit_call_scene()` — トークン生成（LSP）
    - `scope_gen.rs`: `is_callable_item()` — パターンマッチ（変更不要、`CallScene(_)` のまま）
    - `parse_scene.rs`: `parse_call_scene()` 呼び出し箇所 — 変更不要（戻り値型は同じ）
  - **変更不要**: `scope_gen.rs` と `parse_scene.rs` は `CallScene` を不透明に扱うため影響なし
  - **テストファイル**: `scene_test.rs` 4箇所 — スナップショット更新のみ（出力形式が変わらない限り不要）
  - **gap-analysis.md, 他 spec の design.md**: ドキュメント参照のみ、影響なし
- **影響**: 実質修正は `action.rs`, `parse_action.rs`, `element_gen.rs` の3ファイル + `visitors.rs` の軽微修正。コンパイラが全箇所を検出

### Lua ランタイム: nil ガード設計
- **コンテキスト**: 未定義変数での動的コール時に `tostring(nil)` = `"nil"` が検索キーになる問題
- **調査対象**: `ACT_IMPL.call` の現在の実装
- **ファインディング**:
  - 現行コード: `key` に nil が渡されると `self:find_scene(nil, ...)` が実行され、`current_scene[nil]` → `SCENE.search(nil, ...)` と全段検索が走る
  - `SCENE.search` は nil キーでエラーにはならないが、意味のない検索が発生
  - **対策**: `ACT_IMPL.call` の先頭で `if key == nil then log.warn(...); return nil end` を追加
  - `log.warn` は既に `act.lua` で利用可能（ファイル冒頭で `local log = require("pasta.log")` 済み）
- **影響**: act.lua に3行の追加のみ

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク・制約 | 備考 |
|-----------|------|------|-------------|------|
| A: 既存拡張 | `CallScene.target` を `CallTarget` 列挙型に変換 | 型安全、最小変更、既存パターン流用 | 参照箇所の修正（コンパイラ検出） | **採用** |
| B: 新 AST ノード | `DynamicCallScene` を別途追加 | 既存 `CallScene` 無変更 | コード重複、`LocalSceneItem` 等の拡張大 | 不採用 |
| C: 文字列プレフィックス | `target` を `"$varname"` 形式の文字列で保持 | AST 型変更なし | 型安全性喪失、パターンマッチ脆弱 | 不採用 |

## 設計判断

### 判断 1: `CallTarget` 列挙型の設計
- **コンテキスト**: 動的コールと静的コールを AST レベルで区別する方法
- **検討した代替案**:
  1. `CallTarget::Static(String)` + `CallTarget::Dynamic { name, scope: VarScope }` — `VarScope` 全体を再利用
  2. `CallTarget::Static(String)` + `CallTarget::DynamicLocal(String)` + `CallTarget::DynamicGlobal(String)` — スコープ別バリアント
  3. `target: String` + `is_dynamic: bool` + `scope: Option<VarScope>` — フラグ方式
- **選択**: オプション 1
- **根拠**: `VarScope` は既存の確立された型であり、`Local` / `Global` のセマンティクスを正確に表現する。`Args(u8)` バリアントは動的コールでは使用されないが、型システムの制約として受容可能（パーサーが `Args` を生成しないことで保証）
- **トレードオフ**: `VarScope::Args(u8)` が CallTarget のコンテキストでは無意味だが、新規列挙型導入のコストと比較して許容範囲
- **フォローアップ**: パーサー実装時に `var_id` 内の `digit_id`（`$0`, `$1`）が動的コールのコンテキストで出現した場合のエラーハンドリングを検討

### 判断 2: Lua コード生成の動的ターゲット表現
- **コンテキスト**: 動的コールのトランスパイル先 Lua コードの形式
- **検討した代替案**:
  1. `tostring(var["name"])` — ローカル変数直接参照
  2. `act:resolve_var("name")` — 専用メソッド経由
  3. `var.name` — ドットアクセス
- **選択**: オプション 1 （`scope` に応じて `var["name"]` / `save["name"]` を使い分け）
- **根拠**: 既存の `Action::VarRef` コード生成パターン（`element_gen.rs` L126-131）と完全一致。`tostring()` で nil → `"nil"` 文字列変換を防ぐため、コード生成側では `tostring()` を使用し、nil ガードは `ACT_IMPL.call` 側で担当
- **トレードオフ**: `tostring(nil)` は `"nil"` 文字列を返すため、nil ガードがないと `"nil"` という名前のシーンを検索してしまうが、R3-AC5 の nil ガードで対策済み
- **修正**: コード生成は `tostring()` なしで `var["name"]` を直接使用し、nil がそのまま `ACT_IMPL.call` の `key` に渡るようにする。nil ガードが明示的に動作する設計とする

### 判断 3: `visitors.rs` への影響
- **コンテキスト**: LSP 用のトークン生成（`visit_call_scene`）が `CallTarget` 変更で影響を受けるか
- **選択**: `visitors.rs` は `CallScene` のスパン情報のみを使用しており、`target` の型に依存しない可能性が高い。実装時にコンパイラエラーが出た場合のみ対応する
- **根拠**: LSP トークン生成は構文ハイライト目的であり、ターゲットの静的/動的区別はトークンレベルでは不要

## リスクと緩和策
- **リスク 1**: `CallScene.target` 型変更による既存テスト破壊 — コンパイラが全箇所を検出、スナップショットテストで回帰検証
- **リスク 2**: PEG 文法の `var_ref | id` 順序による曖昧性 — PEG は順序付き選択（`/`）であり、`var_ref` が失敗した場合のみ `id` にフォールバック。`＄` で始まる入力は必ず `var_ref` にマッチするため曖昧性なし
- **リスク 3**: `digit_id`（`$0`, `$1`）が動的コールで使用された場合 — パーサーが `VarScope::Args(u8)` として解析し、コード生成で `args[N]` を出力。意味的には「引数の値をシーン名として使う」となり、構文的に正しいが使用頻度は極めて低い

## 参照
- [doc/spec/04-call-spec.md](../../../doc/spec/04-call-spec.md) — §4.1 パターン2: 動的ターゲット仕様
- [doc/spec/09-variables.md](../../../doc/spec/09-variables.md) — 変数スコープ定義
- [gap-analysis.md](./gap-analysis.md) — ギャップ分析（Option A 推奨）
