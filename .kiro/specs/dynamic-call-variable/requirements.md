# Requirements Document

## Project Description (Input)

call-spec.md（doc/spec/04-call-spec.md §4.1 パターン2）で仕様として文書化されている動的コール構文 `＞＄変数名` が、pasta_dsl パーサーで `expected id` エラーとなり使用できない。当初は変数参照（`var_ref`）のみを対象とした修正を検討していたが、技術分析の結果、`id`（静的シーン名）と `expr`（式）の先頭文字集合が完全に素集合であることが確認された。これにより、`＞(id | expr)` という統一的な文法拡張が可能であり、変数参照（`＄変数名`）はその自然な部分集合として解決される。本仕様はこの一般化を採用する。

## Introduction

本仕様は、Pasta DSL のコールターゲットを式（`expr`）まで拡張することを目的とする。`expr` は Pasta DSL で既に定義・実装済みの rule であり、`var_ref`（ローカル変数・グローバル変数参照）・`fn_call`（関数呼び出し）・`paren_expr`（括弧式）・`number_literal`・`string_literal`・`Binary`（二項演算）のいずれかを取る。これにより `＞＄変数名`（単純変数参照）は専用扱いなしに `expr` の `var_ref` として自然に解決される。修正対象は pasta_dsl（パーサー・AST）、pasta_lua（コード生成）の2クレートにまたがり、既存の `Expr` AST 型および `generate_expr()` コード生成関数を最大限に再利用する。

## Requirements

### Requirement 1: コールターゲットの式サポート

**Objective:** ゴースト辞書の作者として、`＞expr` 構文（式をコールターゲットとする動的コール）を `.pasta` ファイルに記述できるようにしたい。`＞＄変数名` のような変数参照はその自然な部分集合として、専用扱いなしに動作する。

#### Acceptance Criteria
1. When `＞expr` 構文（`expr` 部分に変数参照・関数呼び出し・括弧式・算術式等の任意の式）が `.pasta` ファイルに記述された場合, the pasta_dsl パーサー shall パースエラーを発生させず、動的コールとして `CallTarget::Dynamic(Expr)` AST ノードを生成する。
2. When `＞expr` の代表パターン（`＞＄ローカル変数`、`＞＄＊グローバル変数`、`＞（＠func（））` 等）が全角・半角を問わず使用された場合, the pasta_dsl パーサー shall すべて同一の `CallTarget::Dynamic(Expr)` として解析する（`call_marker`・`var_marker` が全角半角両対応済みのため自動対応）。
3. The pasta_dsl パーサー shall 動的コールの AST ノード（`CallTarget::Dynamic(Expr)`）と静的コールの AST ノード（`CallTarget::Static(String)`）を型レベルで区別可能に保つ。

### Requirement 2: 動的コールの Lua コード生成

**Objective:** pasta_lua トランスパイラーの開発者として、`CallTarget::Dynamic(Expr)` AST ノードを正しい Lua コードに変換したい。式の評価結果をシーン名として使用できる。

#### Acceptance Criteria
1. When `CallTarget::Dynamic(Expr)` AST ノードがトランスパイラーに渡された場合, the pasta_lua コードジェネレーター shall 既存の `generate_expr()` 関数を用いて式を評価し、その結果を `tostring()` で文字列化してシーン名として `ACT_IMPL.call` に渡す Lua コードを生成する。
2. The pasta_lua コードジェネレーター shall 動的コールにおいても、静的コールと同一の前方一致検索・ランダム選択セマンティクス（`ACT_IMPL.call`）を適用する Lua コードを生成する。

> **スコープ外**: フィルター構文（`＆key＝value`）は §4.2 により将来予約・現在無視。動的コールでも静的コールと同等に扱い、本仕様のスコープには含めない。

### Requirement 3: 動的コールのエンドツーエンド動作検証

**Objective:** ゴースト辞書の作者として、`＞expr` で式の評価結果をシーン名として解決し、該当シーンを呼び出したい。`＞＄変数名` はその最も基本的なユースケースとして含まれる。

> **注記**: ギャップ分析により、Lua ランタイム（`ACT_IMPL.call`）は任意文字列キーを既に受け付けており構造的変更は不要と確認済み。key=nil 時の早期リターン＋警告ログのガードのみ追加する（R3-AC5）。本要件の AC はパーサー → コード生成 → ランタイムを通したエンドツーエンド動作の検証基準を定義する。

#### Acceptance Criteria
1. When 動的コール `＞＄target`（`＄target` は `expr` の `var_ref` の特殊ケース）が実行され、変数 `＄target` の値が既存のシーン名と前方一致する場合, the pasta ランタイム shall 該当シーンを呼び出し、その出力を返す。
2. When 動的コール `＞＄target` が実行され、変数 `＄target` の値が複数のシーン名と前方一致する場合, the pasta ランタイム shall シャッフル＆順次消費方式で候補からシーンを選択する。
3. When 動的コール `＞＄target` が実行され、変数 `＄target` の値がどのシーン名とも一致しない場合, the pasta ランタイム shall 静的コールで候補が見つからない場合と同一の挙動をする（空文字列を返す）。
4. When Lua ブロックで変数に値を代入し、後続の `＞＄変数名` で動的コールした場合, the pasta ランタイム shall Lua ブロックで設定された変数値をシーン名として正しく解決する。
5. When 動的コール `＞expr` が実行され、式の評価結果が nil となった場合（未定義変数参照等）, the pasta ランタイム shall シーン検索を行わずに早期リターンし、警告ログを出力する。`ACT_IMPL.call` 内の key 引数 nil ガードとして実装する。

### Requirement 4: 既存機能との互換性

**Objective:** ゴースト辞書の作者として、動的コール機能の追加によって既存の静的コール（`＞シーン名`）やその他の DSL 機能が影響を受けないことを保証したい。

#### Acceptance Criteria
1. The pasta_dsl パーサー shall 既存の静的コール `＞シーン名` の解析動作を変更しない。
2. The pasta_lua トランスパイラー shall 既存の静的コールに対して生成される Lua コードを変更しない。
3. The pasta ランタイム shall 動的コール機能の追加後も、既存の全テスト（`cargo test --all`）をパスする。
