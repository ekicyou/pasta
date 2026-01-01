# Requirements Document

## Introduction

pasta_luaトランスパイラーにおいて、`SetValue::WordRef` 構文のコード生成を実装する。これにより「＄場所＝＠場所」形式のPasta DSL構文を「`var.場所 = act:word("場所")`」形式のLuaコードに変換できるようになる。

本機能は、pasta_core側で既に実装済みの `SetValue` 列挙型（`Expr(Expr)` / `WordRef { name: String }` バリアント）を利用し、pasta_lua側のコード生成層を拡張する。

## Requirements

### Requirement 1: WordRef代入のLuaコード生成

**Objective:** As a ゴースト開発者, I want 「＄変数＝＠単語」構文がLuaコードに正しくトランスパイルされること, so that 単語参照結果を変数に代入できる

#### Acceptance Criteria

1. When `SetValue::WordRef { name }` を含む `VarSet` がトランスパイルされる, the pasta_lua code_generator shall `var.変数名 = act:word("単語名")` 形式のLuaコードを生成する
2. When ローカル変数（`VarScope::Local`）への WordRef 代入がトランスパイルされる, the pasta_lua code_generator shall `var.変数名 = act:word("単語名")` を出力する
3. When グローバル変数（`VarScope::Global`）への WordRef 代入がトランスパイルされる, the pasta_lua code_generator shall `save.変数名 = act:word("単語名")` を出力する

### Requirement 2: 既存Expr処理との互換性維持

**Objective:** As a pasta_lua利用者, I want 従来の式代入（`SetValue::Expr`）が引き続き動作すること, so that 既存スクリプトが壊れない

#### Acceptance Criteria

1. The pasta_lua code_generator shall `SetValue::Expr` パターンで従来通りの式展開コードを生成する
2. The pasta_lua code_generator shall `SetValue::Expr` と `SetValue::WordRef` の両方を同一ファイル内で処理できる

### Requirement 3: テスト期待値の更新

**Objective:** As a 開発者, I want テストフィクスチャ `sample.lua` が新しい出力形式を反映すること, so that CI/CDテストが正しく通過する

**Note:** 本仕様はトランスパイル（Luaコード生成）のみを対象とする。Luaランタイム `act:word()` の実装は別仕様で対応。

#### Acceptance Criteria

1. The sample.expected.lua fixture shall 「`var.場所 = act:word("場所")`」形式の出力を期待値として含む
2. When pasta_luaのトランスパイルテストが実行される, the test suite shall `sample.pasta` から生成されたコードが `sample.expected.lua` の期待値と厳密一致することを検証する（文字列比較）
3. The test suite shall トランスパイル結果を `sample.generated.lua` として保存し、デバッグを容易にする
4. The test suite shall トランスパイル結果の正しさのみを検証し、Luaランタイムの実行は検証対象外とする

**Note**: sample.lua は参照実装として保持し、厳密一致テストには sample.expected.lua を使用。

### Requirement 4: エラーハンドリング

**Objective:** As a pasta_lua利用者, I want 引数変数（`VarScope::Args`）への WordRef 代入がエラーになること, so that 不正な操作が明確に報告される

#### Acceptance Criteria

1. If `VarScope::Args` への WordRef 代入がトランスパイルされる, then the pasta_lua code_generator shall `TranspileError` を返す

