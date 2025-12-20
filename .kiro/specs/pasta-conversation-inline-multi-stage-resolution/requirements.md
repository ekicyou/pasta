# Requirements Document

## Project Description (Input)
pastaエンジンに関する仕様。会話行中のインライン要素「＠会話」について。当面は多段展開（＠＠会話など）は実施しない。代わりに変数をキーにした単語検索に対応し、ランタイムで単語辞書を動的に参照できる機能を実装。これにより、変数の値を単語定義キーとして使用し、柔軟な単語解決が可能になる。

## Introduction

Pasta DSLの会話行中にはインライン要素として＠単語が記述できます。本要件は、この単語参照機構を拡張し、リテラル単語キーの参照に加えて、変数をキーとした単語参照をサポートするものです。

これにより、スクリプト実行時に変数の値を単語辞書のキーとして動的に解決することで、より柔軟なスクリプト記述が可能になります。

## Requirements

### Requirement 1: 変数ベース単語参照の構文

**Objective:** As a スクリプト作成者, I want 変数の値をキーに単語辞書を検索できる構文, so that スクリプト実行時に柔軟に単語を選択できる

#### Acceptance Criteria

1. When パーサーが会話行内で＠$識別子形式を検出した場合, the Pasta Parser shall 変数ベース単語参照として解析する
2. The Pasta Parser shall 変数名がRust識別子規則に従うことを検証する（ASCII: a-z, A-Z, 0-9, _、Unicode: XID_Start/XID_Continue、数字始まり禁止）
3. The Pasta Parser shall ＠$の後に直続く識別子を変数名として抽出し、AST内でExpr::VariableWordRef { variable_name }として表現する
4. When パーサーが＠$の後に識別子が続かない場合（例：＠$ ）, the Pasta Parser shall 構文エラーとして検出し、エラーメッセージに位置情報を含める
5. The Pasta Parser shall 既存の＠単語（リテラル参照）と＠$変数（変数参照）を別のASTノードで区別する

### Requirement 2: ランタイム変数解決

**Objective:** As a runtime, I want 変数の値を単語辞書キーに変換して検索できる機構, so that 実行時に動的に単語選択できる

#### Acceptance Criteria

1. When ランタイムが＠$変数ノードを処理する場合, the Pasta Runtime shall 変数マネージャーから変数値を取得する
2. If 指定された変数が未定義の場合, the Pasta Runtime shall エラーハンドリング戦略に従い、適切なエラーを記録する（実行停止 or スキップ or デフォルト値）
3. When 変数の値が取得された場合, the Pasta Runtime shall その値を単語定義キーとして単語辞書から検索を行う
4. The Pasta Runtime shall 変数値をキーとした前方一致検索を実行し、一致するすべての単語を候補として列挙する（既存の前方一致ロジックを再利用）
5. When 変数値に基づく検索で複数の候補が見つかった場合, the Pasta Runtime shall ランダムセレクターで一つを選択し、会話に挿入する

### Requirement 3: 通常参照との相互運用性

**Objective:** As a スクリプト作成者, I want リテラル参照と変数参照を自由に混在させられる, so that 柔軟な会話制御ができる

#### Acceptance Criteria

1. The Pasta Parser shall リテラル単語参照＠単語と変数参照＠$変数を同じ会話行内に混在させることを許可する
2. When 同じ会話行に両方の参照がある場合, the Pasta Runtime shall 各参照を独立して解決し、文字列に挿入する
3. The Pasta Runtime shall リテラル参照のランダム選択と変数参照のランダム選択を独立して行う

### Requirement 4: エラーハンドリング

**Objective:** As a system, I want 変数参照に関する不正な使用法を検出できる, so that スクリプト作成者に明確なエラー情報を提供できる

#### Acceptance Criteria

1. If 変数が未定義のまま参照された場合, the Pasta Runtime shall エラー型PastaError::UndefinedVariable { variable_name, location }を生成する
2. If 変数値が空文字列の場合, the Pasta Runtime shall 対応する単語候補がないことをログに記録し、その部分を空文字列で置換する
3. When 変数値が単語定義キーとしての前方一致で候補なしの場合, the Pasta Runtime shall 警告をログに記録し、該当部分を置換せずスキップする
4. The Pasta Runtime shall すべてのエラーおよび警告に対して、対象の会話行の位置情報（ファイル、行番号）を含める

### Requirement 5: パーサー側構文変更の最小化

**Objective:** As a parser, I want 既存のパーサー定義への影響を最小化する, so that 既存テストが破壊されず保守性を向上させる

#### Acceptance Criteria

1. When パーサーがExpr::WordRef型を処理する場合, the Pasta Parser shall 既存の型を変更せず、新しいExpr::VariableWordRefバリアントを追加する
2. The Pasta Parser shall pest文法のword_refルールに@$ identifierパターンを追加する
3. The Pasta Parser shall ASTコンバーター（ast.rs）内で新しいバリアントへのマッピング処理を追加する

### Requirement 6: テスト・検証

**Objective:** As a development team, I want 変数参照機能が正しく動作することを確認できる, so that 回帰テストで品質を保証できる

#### Acceptance Criteria

1. When ＠$変数参照を含むPastaスクリプトが実行される場合, the Pasta Test Suite shall 変数が正しく解決され、対応する単語が選択されることを検証する
2. When 変数が未定義の場合, the Pasta Test Suite shall エラーが正しく記録されることを検証する
3. When 変数値に基づく前方一致で複数の単語が候補である場合, the Pasta Test Suite shall ランダム選択が複数回実行で異なる単語を選択することを検証する
4. The Pasta Test Suite shall リテラル参照と変数参照の混在シナリオをテストスクリプトで実行し、期待通りの出力を検証する