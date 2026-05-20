# Requirements Document

## Introduction
SSPのプロパティシステムは、ベースウェアが保持する各種パラメータへの読み書きアクセスを提供する。本仕様は書き込み側（`\![set,property,name,value]`）に対応するLua APIをpasta actオブジェクトに追加し、ゴースト作者がさくらスクリプトのタグフォーマットを意識せずにプロパティを設定できるようにする。

## Boundary Context
- **In scope**: `act:set_property(name, value)` メソッドによるプロパティ書き込みタグの生成とさくらスクリプト出力への統合
- **Out of scope**: プロパティ読み取り（`get_property` — `shiori-async-talk` specの範囲）、`%property[name]` 環境変数展開、Pasta DSL構文（`property-dsl-extension`の範囲）、プロパティ名の妥当性検証（SSP側の責任）
- **Adjacent expectations**: 既存のactトークンバッファおよびさくらスクリプトビルドパイプラインが既存トークン型を正しく処理すること（変更なしで利用）

## Requirements

### Requirement 1: プロパティ書き込みメソッド
**Objective:** ゴースト作者として、actオブジェクトのメソッド呼び出しでSSPプロパティを書き込みたい。これにより、さくらスクリプトタグの手動組み立てが不要になる。

#### Acceptance Criteria
1. When ゴースト作者が `act:set_property(name, value)` を呼び出したとき, the pasta ランタイム shall 対応する `\![set,property,<name>,<value>]` さくらスクリプトタグをさくらスクリプト出力に含める
2. When `act:set_property(name, value)` を呼び出したとき, the pasta ランタイム shall actオブジェクト自身を返し、メソッドチェーンを可能にする
3. When 同一イベントハンドラ内で `act:set_property` を複数回呼び出したとき, the pasta ランタイム shall 呼び出し順にすべてのプロパティ設定タグをさくらスクリプト出力に含める
4. When `act:set_property` のみを呼び出し他のトーク（`act:talk` 等）を伴わないとき, the pasta ランタイム shall プロパティ設定タグを単独でさくらスクリプト出力に含める

### Requirement 2: 引数のバリデーションと変換
**Objective:** ゴースト作者として、不正な引数には明確なエラーを受け取り、省略可能な引数は意図通りに扱われてほしい。

#### Acceptance Criteria
1. If `name` 引数が `nil` または空文字列であるとき, the pasta ランタイム shall エラーを発生させる
2. When `value` 引数が `nil` であるとき, the pasta ランタイム shall 空文字列を値として `\![set,property,<name>,]` タグを生成する（SSPにおけるプロパティ定義削除として動作）
3. When `value` 引数が空文字列であるとき, the pasta ランタイム shall `\![set,property,<name>,]` タグを生成する（SSPにおけるプロパティ定義削除として動作）
4. When `value` 引数が渡されたとき, the pasta ランタイム shall 型によらず `tostring()` を適用して文字列に変換した値をタグに含める
5. When `tostring()` 後の文字列がSSPタグ構文上の特殊文字を含むとき, the pasta ランタイム shall 当該文字をエスケープしてタグの構造を破壊しない出力を生成する
