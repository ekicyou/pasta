# 要件ドキュメント

## プロジェクト説明（入力）
pasta_dslクレートはPest PEGパーサーを用いたDSL解析層で約2500行に成長している。パーサーは外部入力（.pastaファイル）を直接処理するため、入力検証・エラーハンドリングの堅牢性が重要。parser/mod.rs（300+行）を含むパーサーモジュール全体の複雑度検証、デッドコード除去、冗長表現削減を実施し、外部振る舞い（公開API・AST型）を不変に保つ。

## 境界コンテキスト
- **対象内**: pasta_dsl/src/ 全ファイル（13ファイル、約2500行）の脆弱性調査、パーサーの堅牢性検証、デッドコード除去、冗長表現削減
- **対象外**: Pest文法定義（grammar.pest）の変更、AST型の公開インターフェース変更、新しいDSL構文の追加、pasta_coreへの変更
- **隣接期待**: pasta_core（上流レジストリ型の参照のみ、変更なし）、pasta_lua・pasta_lsp（下流依存、公開APIが不変であれば影響なし）

## 要件

### 要件1: 入力検証の堅牢性
**目的:** 開発者として、外部入力（.pastaファイル）の解析において悪意ある入力や不正な入力に対して安全な動作を保証したい。パーサーがクラッシュや無限ループなしにエラーを報告できるようにするため。

#### 受け入れ基準
1. When 不正なUTF-8シーケンスを含む入力が与えられた場合, the pasta_dsl shall パニックせずに適切なParseErrorを返す
2. When 極端に長い行（10,000文字以上）を含む入力が与えられた場合, the pasta_dsl shall 妥当な時間内にパース結果またはエラーを返す
3. When 深くネストされた構造を含む入力が与えられた場合, the pasta_dsl shall スタックオーバーフローなしに処理を完了する
4. If パーサー内部で予期しない状態が発生した場合, the pasta_dsl shall unwrap()/expect()によるパニックではなくResult型でエラーを伝播する
5. The pasta_dsl shall 外部入力パスにおいて`unwrap()`、`expect()`、`panic!()`の直接使用を排除する（テストコード除く）

### 要件2: エラーハンドリングの一貫性
**目的:** 開発者として、パースエラーの情報が一貫した形式で提供され、上流・下流での診断が容易であってほしい。

#### 受け入れ基準
1. When パースエラーが発生した場合, the pasta_dsl shall ファイル名、行番号、列番号を含むエラー情報を提供する
2. The pasta_dsl shall エラー型（ParseError）の全バリアントがDisplay/Debugトレイトを適切に実装する
3. When 複数のパースエラーが発生した場合, the pasta_dsl shall MultipleErrorsバリアントで全エラーを集約して報告する

### 要件3: デッドコード除去
**目的:** メンテナーとして、使用されていないコード（未使用関数、未到達分岐、不要なimport）を除去し、コードベースの可読性と保守性を向上させたい。

#### 受け入れ基準
1. The pasta_dsl shall 未使用のpub関数・メソッドを含まない（クレート外部から参照されないpub項目の可視性を最小化）
2. The pasta_dsl shall 未使用のimport文（use宣言）を含まない
3. The pasta_dsl shall 到達不能なmatch分岐やif分岐を含まない
4. When デッドコード除去が完了した場合, the pasta_dsl shall 既存テストスイート全体がパスする

### 要件4: 冗長表現の削減
**目的:** メンテナーとして、同一ロジックの重複や不必要に冗長なパターンを簡素化し、コード量を削減したい。

#### 受け入れ基準
1. The pasta_dsl shall 同一の変換ロジックが複数箇所に重複しない（共通パターンの抽出）
2. The pasta_dsl shall 不要な中間変数や冗長なクロージャを含まない
3. The pasta_dsl shall match式における冗長なパターン（同一処理の複数アーム）を統合する
4. When 冗長表現の削減が完了した場合, the pasta_dsl shall 既存テストスイート全体がパスする

### 要件5: パーサー複雑度の削減
**目的:** メンテナーとして、パーサーモジュール全体の認知的複雑度を下げ、将来の機能追加や修正を容易にしたい。

#### 受け入れ基準
1. The pasta_dsl shall 各関数の行数が概ね50行以下となるよう適切に分割する
2. The pasta_dsl shall ネストの深い処理（3段階以上のインデント）を早期リターンやヘルパー関数で簡素化する
3. When パーサーの複雑度削減が完了した場合, the pasta_dsl shall 既存テストスイート全体がパスし外部振る舞いが不変である

### 要件6: 外部振る舞いの不変性
**目的:** 下流クレート利用者として、監査によるリファクタリングが公開API・AST型・パース結果に影響しないことを保証したい。

#### 受け入れ基準
1. The pasta_dsl shall 公開API（`parse_str`、`parse_file`、`parse_str_partial`）のシグネチャを変更しない
2. The pasta_dsl shall AST型（`PastaFile`、`FileItem`、`SceneScope`等）の公開フィールド・バリアントを変更しない
3. The pasta_dsl shall エラー型（`ParseError`、`PartialParseError`）の公開インターフェースを変更しない
4. When 全監査作業が完了した場合, the pasta_dsl shall ワークスペース全体のテストスイート（`cargo test --workspace`）がパスする
5. When 全監査作業が完了した場合, the pasta_dsl shall 性能劣化がない（既存のパース速度を維持する）

### 要件7: partial.rsのパーシャルパース堅牢性
**目的:** LSP利用者として、不完全なソースコードに対するパーシャルパースが安全かつ堅牢に動作してほしい。

#### 受け入れ基準
1. When 空文字列が入力された場合, the pasta_dsl shall パニックせずに空のPartialParseResultを返す
2. When 構文的に不完全な入力（閉じ括弧なし等）が与えられた場合, the pasta_dsl shall パース成功部分をitemsに、失敗部分をerrorsに分離して返す
3. If パーシャルパースの行推論（infer_rule_from_line）で対応ルールが見つからない場合, the pasta_dsl shall Noneを返し処理をスキップする
