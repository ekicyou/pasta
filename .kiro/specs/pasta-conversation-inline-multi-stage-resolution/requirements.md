# Requirements Document

## Introduction
pastaエンジンの会話行に含まれるインライン要素「＠単語」を拡張し、リテラルキー参照に加えて変数の値を単語辞書キーとして動的に解決できるようにする。多段展開（＠＠会話）は今回扱わず、変数ベースのキー解決と既存挙動との共存、エラーハンドリング、テスト検証を明確にする。

## Requirements

### Requirement 1: 変数ベース単語参照の構文
**Objective:** As a スクリプト作成者, I want 会話行で@$変数を使って単語を参照できる構文, so that 実行時に柔軟な単語選択を記述できる

#### Acceptance Criteria
1. When 会話行で@$に続く識別子が検出された場合, the Pasta Parser shall 変数ベース単語参照として解析する。
2. If @$の後に識別子が存在しないかRust/XID識別子規則に違反する場合, the Pasta Parser shall 構文エラーを生成し位置情報を含める。
3. The Pasta Parser shall 変数ベース単語参照をリテラル単語参照とは別のASTノードとして保持する。
4. When 会話行がリテラル参照と変数参照を混在させる場合, the Pasta Parser shall 元の並び順でコンテンツパーツをASTに格納する。

### Requirement 2: 変数値のランタイム解決と辞書検索
**Objective:** As a runtime, I want 変数値を単語辞書キーとして動的に検索できる, so that 実行時に前方一致選択を行える

#### Acceptance Criteria
1. When ランタイムが変数ベース単語参照ノードを評価する場合, the Pasta Runtime shall 変数スコープ規則に従ってVariableManagerから値を取得する。
2. If 変数が未定義の場合, the Pasta Runtime shall PastaError::UndefinedVariableを位置情報付きで記録し当該参照の解決を中断する。
3. When 変数値が取得された場合, the Pasta Runtime shall その値をキーに単語辞書へ前方一致検索を行い候補を列挙する。
4. When 前方一致で複数候補が見つかった場合, the Pasta Runtime shall RandomSelectorで一つを選択し会話に挿入する。
5. Where 単語辞書へのアクセスが失敗する場合, the Pasta Runtime shall エラーを記録し残りの会話処理を継続可能な形でフォールバックする。

### Requirement 3: リテラル参照との相互運用性
**Objective:** As a スクリプト作成者, I want リテラル参照と変数参照を同一行で混在させられる, so that 既存記法を保ったまま柔軟性を得られる

#### Acceptance Criteria
1. When 会話行にリテラル参照と変数参照が共存する場合, the Pasta Runtime shall 各参照を独立に解決し元の順序で出力に組み立てる。
2. When 参照の解決結果にランダム選択が含まれる場合, the Pasta Runtime shall 各参照ごとに独立してランダム選択を実行する。
3. The Pasta Runtime shall 変数参照の解決が失敗しても他のリテラル参照の解決を継続する。

### Requirement 4: エラーハンドリングと診断
**Objective:** As a system, I want 変数参照に起因する異常を検出・通知できる, so that スクリプト作成者が原因を特定できる

#### Acceptance Criteria
1. If 変数値が空文字の場合, the Pasta Runtime shall 警告を記録し対象部分を空文字で置換する。
2. If 前方一致検索で候補が存在しない場合, the Pasta Runtime shall 警告を記録し該当参照を出力に含めずスキップする。
3. The Pasta Runtime shall すべてのエラーおよび警告にファイル名と行番号を含めて報告する。
4. When エラーが発生する場合, the Pasta Runtime shall 上位レイヤーにエラー状態を伝搬するためのインターフェースを提供する。

### Requirement 5: テストと検証
**Objective:** As a development team, I want 変数参照機能の正常系・異常系を回帰テストで確認できる, so that 品質を継続的に保証できる

#### Acceptance Criteria
1. When 変数参照を含むPastaスクリプトを実行する場合, the Pasta Test Suite shall 変数値が辞書キーとして解決され単語が挿入されることを検証する。
2. If 変数が未定義の場合, the Pasta Test Suite shall エラー出力と位置情報が期待通りであることを検証する。
3. When 前方一致候補が複数ある場合, the Pasta Test Suite shall 複数回実行で異なる候補が選択され得ることを確認する。
4. When リテラル参照と変数参照を混在させたスクリプトを実行する場合, the Pasta Test Suite shall 参照が独立に解決され順序が保持されることを検証する。
5. When 変数値が空文字または未登録キーである場合, the Pasta Test Suite shall 警告ログの内容と出力結果を検証する。