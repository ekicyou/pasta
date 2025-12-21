# Requirements Document

## Introduction
本仕様は、パスタスクリプトにおける変数代入（$変数: 値／$*変数: 値）と、変数参照の各行タイプ（アクション行の$変数／@$変数、コール行の>$変数）について、トランスパイラー層が生成すべきIR（振る舞い）を定義する。

### 憲法レベル決定
**変数スコープAPI仕様**（全ての要件・設計・実装はこれに従う）:
- **ローカル変数** (`$変数`) → `ctx.local` に保持・参照
- **グローバル変数** (`$*変数`) → `ctx.global` に保持・参照
- 現行の `ctx.var.*`、`get_global()`、`set_global()` は全て変更対象

## Requirements

### 1. 変数スコープ管理
**Objective:** スクリプト作者として、ローカル／グローバルの変数参照が期待通りに解決されてほしい。そうすることで、会話ロジックが決定的に動作する。

#### Acceptance Criteria
1. The Pasta Transpiler shall ローカル変数`$変数`はローカルスコープから参照する。
2. Where `$*変数`（グローバル記法）が使用される, the Pasta Transpiler shall グローバルスコープから値を参照する。
3. If 同名の変数がローカルとグローバルに存在する, the Pasta Transpiler shall `$変数`はローカル値を優先して解決する。
4. Where 変数名が日本語などUNICODE識別子である, the Pasta Transpiler shall 解析・参照をサポートする。

### 2. 変数代入（アサイン）
**Objective:** スクリプト作者として、`$変数: 値`／`$*変数: 値`の代入によって正しいスコープに値が保存されてほしい。そうすることで、後続の参照が一貫性を保つ。

#### Acceptance Criteria
1. When スクリプトが`$変数: 値`を含む, the Pasta Transpiler shall ローカルスコープへ値を保存するIRを生成する。
2. When スクリプトが`$*変数: 値`を含む, the Pasta Transpiler shall グローバルスコープへ値を保存するIRを生成する。
3. The Pasta Transpiler shall 代入の値は文字列リテラルとして扱い、後続参照に利用可能にする。
4. If 代入対象の識別子が無効（文法外）である, the Pasta Transpiler shall 診断エラーを生成しトランスパイルを失敗として報告する。

### 3. アクション行での`$変数`参照
**Objective:** スクリプト作者として、アクション行内の`$変数`が会話出力へ展開されてほしい。そうすることで、変数の値をユーザーに提示できる。

#### Acceptance Criteria
1. When アクション行に`$変数`が現れる, the Pasta Transpiler shall 参照値を用いた会話出力イベント（Talk）IRを生成する。
2. If 参照変数が未定義である, the Pasta Transpiler shall 診断エラーを生成し、その行の変換を失敗として報告する。

### 4. アクション行での`@$変数`参照
**Objective:** スクリプト作者として、アクション行内の`@$変数`が単語検索（word検索）へ展開されてほしい。そうすることで、変数値をキーにした語句解決が可能になる。

#### Acceptance Criteria
1. When アクション行に`@$変数`が現れる, the Pasta Transpiler shall 変数値を検索キーとする単語検索IRを生成する。
2. If 検索キーが空または未定義である, the Pasta Transpiler shall 診断エラーを生成し、その行の変換を失敗として報告する。

### 5. コール行での`>$変数`参照
**Objective:** スクリプト作者として、コール行の`>$変数`がシーン呼び出しへ展開されてほしい。そうすることで、変数値をラベルとして場面遷移できる。

#### Acceptance Criteria
1. When コール行に`>$変数`が現れる, the Pasta Transpiler shall 変数値をラベルキーとするシーン呼び出しIRを生成する。
2. If ラベルキーが空または未定義である, the Pasta Transpiler shall 診断エラーを生成し、その行の変換を失敗として報告する。

### 6. 一般品質要件
**Objective:** トランスパイル結果がテスト可能で一貫した品質を維持する。

#### Acceptance Criteria
1. The Pasta Transpiler shall すべての上記要件をユニット／統合テストで検証可能なIRとして出力する。
2. While トランスパイルが進行中である, the Pasta Transpiler shall エラー検出時に適切な診断情報（位置・原因）を付与する。


