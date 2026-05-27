# Brief: property-dsl-extension

## Problem
ゴースト作者がSSPプロパティシステムにアクセスするには、現在Luaブロック内で `act:set_property()` / `act:get_property()` を直接呼び出す必要がある。DSLレベルの構文がないため、プロパティの読み書きという頻出操作のたびにLuaコンテキストスイッチが発生し、辞書の可読性と記述効率が低下している。

## Current State
- `act:set_property(name, value)` — Lua APIとして実装済み（`property-write-helpers` spec完了）
- `act:get_property(name)` — Lua APIとして実装済み（`shiori-async-talk` spec完了）
- Pasta DSLからはLuaブロック経由でのみ利用可能。専用のDSL構文は存在しない

## Desired Outcome
Pasta DSL内で `＄％` スコープ修飾子を使い、プロパティの読み書きをローカル/グローバル変数と同じ直感性で記述できる。トランスパイラが既存のLua APIコールに変換する。

## Approach
**＄％ スコープ修飾方式（C'案）** — 既存の変数スコープ修飾パターン（`＄` = ローカル、`＄＊` = グローバル）を拡張し、`＄％` を「共有プロパティスコープ」として追加する。

スコープ階層:
```
＄var        → ローカル（セッション内）   → var.name
＄＊var      → グローバル（永続・自ゴースト）→ save.name
＄％prop.path → 共有プロパティ（外部・SSP）  → act:get/set_property(...)
```

選定理由:
- `＄＊` との対称構造が文法的根拠を与える（場当たり的でない）
- `＄％` が明示的なパース入口。既存の `＄` 分岐にパスを1つ追加するだけ
- ドット区切りパスは `＄％` の後でのみ有効 → 一般の変数パースに影響なし
- ゴースト作者の学習コスト最小（「％付き＝外のプロパティ」だけ覚えればよい）

## Scope
- **In**:
  - `＄％prop.path＝value` によるプロパティSET（同期、さくらスクリプトタグ発行）
  - `＄var＝＄％prop.path` によるプロパティGET（非同期、yield/resume）
  - アクション行内でのインラインGET（`さくら：＄％prop.path　テキスト`、プリフェッチ方式）
  - 1アクション行内の複数 `＄％` 参照のバッチ最適化（`get_property({...})` 1回に集約）
  - `＄％「string」` による文字列リテラルフォールバック（`scope(0)` 等を含む複雑パス対応）
  - Pest文法拡張（`property_ref`, `property_set`, `property_path` ルール）
  - AST拡張（`PropertyRef`, `PropertySet` ノード型）
  - トランスパイラ拡張（プロパティノード → Lua APIコール生成）
- **Out**:
  - 式中でのプロパティ参照（`＄result＝＄％a ＋ ＄％b`）— 将来拡張
  - プロパティ値の型変換（文字列として返す、既存API準拠）
  - `%property[name]` 環境変数展開
  - 新しいLua APIの追加（既存の `set_property`/`get_property` にトランスパイル）
  - LSP対応（プロパティ名補完等）— 別spec

## Boundary Candidates
- Pest文法 + AST拡張（パーサー層）
- トランスパイラのコード生成（Lua生成層）
- プリフェッチ＋バッチ最適化（トランスパイラ最適化層）

## Out of Boundary
- Lua API層の変更（既存APIをそのまま利用）
- ランタイム層の変更（yield/resume基盤は `shiori-async-talk` で完成済み）
- LSPのプロパティ名補完やバリデーション

## Upstream / Downstream
- **Upstream**: `property-write-helpers`（SET用Lua API）、`shiori-async-talk`（GET用Lua API + 非同期基盤）
- **Downstream**: LSPプロパティ補完（将来）、式中プロパティ参照（将来拡張）

## Existing Spec Touchpoints
- **Extends**: `property-write-helpers`, `shiori-async-talk` のLua APIにトランスパイル
- **Adjacent**: `pasta-transpiler-variable-expansion`（変数スコープの実装パターン）、`pasta-cue-dsl-extension`（DSLパース拡張の実装パターン）

## Constraints
- Pest 2.8.6 PEGパーサーの制約内で文法拡張
- 既存の `＄` / `＄＊` パースパスへの影響ゼロ
- `％` マーカーの行頭用途（アクター辞書）との衝突なし（`＄％` は `＄` が先行するため曖昧性ゼロ）
- プロパティ名のドット `.` は XID_Continue に含まれないため、識別子パースとの衝突なし

## 構文サンプル

```pasta
＃ SET（プロパティ書き込み — 同期）
＄％system.name＝new_value

＃ GET（プロパティ読み取り → 変数代入 — 非同期）
＄名前＝＄％currentghost.name
さくら：ゴースト名は＄名前　です

＃ GET（アクション行インライン — プリフェッチ方式）
さくら：ゴースト名は＄％currentghost.name　です

＃ 複数プロパティのバッチ取得（1 yield に集約）
さくら：名前は＄％currentghost.name　で幅は＄％currentghost.width　です

＃ 複雑なプロパティ名（文字列リテラルフォールバック）
＄幅＝＄％「currentghost.balloon.scope(0).validwidth.initial」

＃ 式文（SET、副作用のみ）
＄＝＄％system.name
```

## トランスパイル例

```pasta
＄％system.name＝new_value
```
→
```lua
act:set_property("system.name", "new_value")
```

```pasta
＄名前＝＄％currentghost.name
```
→
```lua
var["名前"] = act:get_property("currentghost.name")
```

```pasta
さくら：名前は＄％currentghost.name　で幅は＄％currentghost.width　です
```
→
```lua
local _p1, _p2 = act:get_property({"currentghost.name", "currentghost.width"})
act.sakura:talk("名前は" .. tostring(_p1) .. "で幅は" .. tostring(_p2) .. "です")
```
