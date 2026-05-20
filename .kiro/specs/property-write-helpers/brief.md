# Brief: property-write-helpers

## 問題
ゴースト作者がSSPのプロパティを書き換えたい場合（シェルメニュー非表示化、ツールチップ変更、カーソル変更など）、さくらスクリプトの `\![set,property,name,value]` タグを手動で組み立てる必要がある。現在のpasta ランタイムにはプロパティ書き込み用のAPIが存在しない。

## 現状
- `act:raw_script(text)` でさくらスクリプトを直接注入することは技術的に可能だが、エスケープやフォーマットをゴースト作者が自力で管理する必要がある
- プロパティ関連のヘルパーは一切実装されていない
- `\![set,property,...]` はSSPが処理するfire-and-forget型のタグであり、SHIORIコールバックは発生しない（yield不要）

## 期待する成果
- ゴースト作者が `act:set_property("currentghost.mousecursor", "my_cursor.cur")` のようにLuaメソッド一発でプロパティを書き込めること
- さくらスクリプトのタグフォーマットの詳細を意識せずにプロパティ操作ができること

## アプローチ
`act` オブジェクトに `set_property(name, value)` メソッドを追加する。内部では `\![set,property,name,value]` さくらスクリプトタグを `raw_script` トークンとしてトークンバッファに追加する。既存のトークンバッファ → build → さくらスクリプト生成パイプラインをそのまま利用するため、インフラ変更は不要。

## スコープ
- **対象**: 
  - `act:set_property(name, value)` メソッド
  - SETが有効なプロパティ（`[SET有効]` マーク付き）への書き込み
  - 値のエスケープ処理（カンマ等の特殊文字）
- **対象外**: 
  - プロパティの読み取り（`get_property` は `shiori-async-talk` specの範囲）
  - `%property[name]` 環境変数展開（不要と判断）
  - プロパティ名のバリデーション（SSP側の責任）
  - DSL構文（`property-dsl-extension` の範囲）

## 境界候補
- act オブジェクトへのメソッド追加（pasta_lua/pasta_scripts/pasta/act.lua または shiori/act.lua）
- さくらスクリプトタグフォーマット生成ロジック

## 対象外
- yield/resumeメカニズムの変更
- イベントディスパッチャーの変更
- SHIORIプロトコルレベルの変更

## 上流 / 下流
- **上流**: 既存の act トークンバッファシステム（`act:raw_script()`）、さくらスクリプトビルダー
- **下流**: `shiori-async-talk`（get_property実装時にset_propertyと対になるAPI設計を参照）、`property-dsl-extension`（DSLトランスパイルのターゲットAPI）

## 既存Specとの接点
- **拡張**: なし（新規メソッド追加のみ）
- **隣接**: `act-sakura-script-method`（完了済み、`act:sakura_script()` raw注入の先行実装）

## 制約
- 既存の `act` APIデザインに準拠（メソッドチェーン、トークンバッファ方式）
- LuaJIT 2.1互換
- テストは `lua_test` BDDフレームワークで記述
