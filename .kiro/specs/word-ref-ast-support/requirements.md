# Requirements Document

## Introduction

本仕様は、Pasta DSL文法において式（term）内で単語参照（`word_ref`）を使用可能にするためのAST対応を定義します。
これにより「`＄場所＝＠場所`」のような構文で、変数に単語参照の結果を代入できるようになります。

**背景**: grammar.pestの`term`規則に`word_ref`が追加され、式コンテキストで単語参照が許可されました。
しかし、AST（Expr型）およびパーサー実装にはこの対応がありません。

**段階的実装計画**:
- **Phase 1**: pasta_core AST追加 + パーサー実装（このリリース）
- **Phase 2**: pasta_rune + pasta_lua トランスパイラー最小実装（次のリリース）
- **Phase 3**: トランスパイラー完全実装 + E2Eテスト（将来）

## Project Description (Input)
文法「＄場所＝＠場所」を処理できるように、「grammar.pest」を変更しました。
termに「word_ref」を追加しています。ASTの対応をお願いします。

## Requirements

### Requirement 1: Expr型へのWordRefバリアント追加（Phase 1）

**Objective:** As a Pasta DSL開発者, I want 式内で単語参照（`@word`）を使用できること, so that 変数代入や関数引数に単語のランダム選択結果を渡せるようになる

#### Acceptance Criteria
1. When パーサーがterm内で`word_ref`規則を検出した場合, the Pasta Parser shall `Expr::WordRef`バリアントを生成する
2. The `Expr` enum shall `WordRef { name: String }` バリアントを持つ
3. When `＄変数＝＠単語名` 形式の変数代入をパースした場合, the Pasta Parser shall `VarSet { value: Expr::WordRef { name: "単語名" } }`として解析する

### Requirement 2: パーサーのtry_parse_expr関数対応（Phase 1）

**Objective:** As a Pasta Parser, I want `Rule::word_ref`を式として解析できること, so that grammar.pestの文法変更がAST変換に反映される

#### Acceptance Criteria
1. When `try_parse_expr`関数が`Rule::word_ref`を受け取った場合, the Pasta Parser shall `Some(Expr::WordRef { name })`を返す
2. When `word_ref`が`args`（関数引数）内に出現した場合, the Pasta Parser shall 正常に`Arg::Positional(Expr::WordRef { name })`として解析する
3. When `word_ref`が二項演算の項として出現した場合, the Pasta Parser shall 正常に`Expr::Binary`の`lhs`または`rhs`として解析する

### Requirement 3: 既存Action::WordRefとの共存（Phase 1）

**Objective:** As a Pasta DSL設計者, I want アクション行での`@word`とexpr内での`@word`が明確に区別されること, so that 両方のコンテキストで単語参照が適切に動作する

#### Acceptance Criteria
1. The `Action::WordRef` shall アクション行（`actor：@word テキスト`）のコンテキストで引き続き使用される
2. The `Expr::WordRef` shall 式コンテキスト（変数代入、関数引数、演算式）でのみ使用される
3. When 同一ファイル内でアクション行と式の両方で`@word`を使用した場合, the Pasta Parser shall 各コンテキストに応じて適切なAST型（`Action::WordRef`または`Expr::WordRef`）を生成する

### Requirement 4: トランスパイラー最小実装（Phase 2）

**Objective:** As a Pasta Compiler, I want `Expr::WordRef`がコンパイル時エラーを引き起こさないこと, so that 既存テストスイートが引き続き成功する

#### Acceptance Criteria
1. When pasta_rune/pasta_luaの`generate_expr`関数が`Expr::WordRef`を受け取った場合, the Transpiler shall コンパイルエラーを出さずに処理する
2. When `Expr::WordRef`に対して最小実装が適用された場合, the Transpiler shall 既存テストをすべて成功させる（回帰なし）
3. The `Expr::WordRef`ハンドラー shall TODO/最小コード（例: 未実装マーカー、stub実装）を含む

### Requirement 5: Phase 1 パーステストカバレッジ

**Objective:** As a Pasta開発チーム, I want 式内単語参照のパーサー動作を検証するテストが存在すること, so that 回帰を防止できる

#### Acceptance Criteria
1. The テストスイート shall `＄変数＝＠単語名` 形式のパーステストを含む
2. The テストスイート shall 関数引数内での`@単語名`のパーステストを含む
3. The テストスイート shall 二項演算との組み合わせ（例: `＄x＝＠数値 ＋ 10`）のパーステストを含む
