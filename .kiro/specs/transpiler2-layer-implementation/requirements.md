# Requirements Document

## Project Description (Input)
parser2-pest-migrationを完成させた後、トランスパイラー2層を実装する。新しいpasta2.pestベースのParser2が提供するAST構造に対応した効率的なトランスパイラーを構築し、従来のtranspilerコンポーネントを段階的に置き換える基盤を確立する。

## Introduction
本仕様は、既に完成した`parser2`モジュール（parser2-pest-migration完了後）を入力として、新たな`transpiler2`レイヤーを実装するための要件を定義します。transpiler2は、parser2が生成した新AST型に対応し、より保守性の高い実装設計を提供します。レガシーtranspilerと並存しながら、段階的に置き換える基盤を確立します。

**実装方針（ギャップ分析承認済み）**: SceneRegistry/WordDefRegistry/SceneTable/WordTableを共有モジュール`src/registry/`に統合し、transpiler/transpiler2/runtimeから共用します。これにより、コード重複を完全に排除し、既存Registry実装を100%再利用します。

## Requirements

### Requirement 1: Transpiler2 Module Independence
**Objective:** 開発者として、既存のtranspilerと独立した新しいtranspiler2モジュールを作成したい。これにより、新AST構造への対応と段階的移行を実現できる。

#### Acceptance Criteria
1. The Pastaプロジェクト shall 独立した名前空間を持つ新しいモジュール`src/transpiler2/`を作成する
2. The Transpiler2モジュール shall レガシーtranspilerと同じ公開API関数を公開する：`transpile_str`、`transpile_file`（モジュールパス`pasta::transpiler2::*`経由で名前空間分離）
3. When lib.rsが公開APIを公開する、the Pastaクレート shall `transpiler2`モジュールをpublicとして公開する（`pub mod transpiler2;`）、`pasta::transpiler2::*`経由で使用可能にする
4. The Transpiler2モジュール shall 完全な独立性を保証するため、レガシーtranspilerモジュールとコンテキスト型定義を共有しない
5. The Transpiler2 shall Parser2（`pasta::parser2::*`）が生成したAST型のみを入力として受け取る
6. The Transpiler2 shall レガシーtranspilerモジュールとのコンパイルエラーやランタイム競合を引き起こさない

### Requirement 2: AST-to-Rune Code Generation
**Objective:** 開発者として、Parser2 AST構造を完全に解析し、高品質なRune仮想機械コードを生成したい。これにより、スクリプト実行の基盤を確立できる。

#### Acceptance Criteria
1. The Transpiler2 shall Parser2 AST（PastaFile）の3層スコープ構造（FileScope ⊃ GlobalSceneScope ⊃ LocalSceneScope）を完全に処理する
2. The Transpiler2 shall すべてのグローバルシーン定義（`global_scene_scope`）をRuneのグローバル関数として生成する
3. The Transpiler2 shall ローカルシーン定義（`local_scene_scope`）をRuneのネストされた関数として生成する
4. The Transpiler2 shall ActionLine（会話行）をRuneの`yield`文へ変換し、`ScriptEvent::Talk`イベントを生成する
5. The Transpiler2 shall 継続行（ContinueAction）を正しく処理し、会話コンテンツを連結する
6. The Transpiler2 shall 単語参照（`＠word_name`）をRuneの単語函数呼び出しに展開する
7. The Transpiler2 shall 変数参照・代入（`＄var_name`、`＄var_name：value`）をRuneの変数操作に変換する
8. The Transpiler2 shall Runeコードブロック（` ```rune ... ``` `）をそのままRuneコード領域に埋め込む

### Requirement 3: Call Scene Resolution
**Objective:** 開発者として、Call文（シーンの呼び出し）を正しく解決し、Rune関数呼び出しコードに変換したい。これにより、会話の分岐と階層構造を実現できる。

#### Acceptance Criteria
1. The Transpiler2 shall Call行（`＞scene_name`）を検出し、対応するシーン関数呼び出しに変換する
2. The Transpiler2 shall ローカルシーン呼び出し（同親内）と外部シーン呼び出し（グローバル）を区別する
3. The Transpiler2 shall シーン名が存在しない場合、コンパイルエラーを生成する
4. The Transpiler2 shall 呼び出し先シーンのyield結果をそのまま呼び出し元にyieldする（generator継続）
5. When シーンが呼び出される、the Transpiler2 shall 同名の複数シーン定義（重複シーン）に対応するランダム選択メカニズムを実装する

### Requirement 4: Symbol Resolution and Scope Management
**Objective:** 開発者として、Pasta DSLの識別子（シーン名、単語名、変数名）をRuneシンボルに正しく解決したい。これにより、スコープルールを正確に実装できる。

#### Acceptance Criteria
1. The Transpiler2 shall FileScope内の属性定義（`＆attribute：value`）を処理し、グローバルメタデータとして管理する
2. The Transpiler2 shall すべてのグローバルシーン定義を1回目のパス（Phase 1）で登録し、シンボルテーブルを構築する
3. The Transpiler2 shall すべてのローカルシーン定義を各グローバルシーンの処理時に登録する
4. The Transpiler2 shall すべての単語定義（`＠word_name：...`）を1回目のパス（Phase 1）で登録する
5. The Transpiler2 shall 未定義のシーン・単語参照を検出し、TranspileErrorを生成する
6. The Transpiler2 shall ローカルシーン内の単語参照を適切にバインディングする（同一スコープ・親スコープ検索）

### Requirement 5: Variable Scope and Lifetime Management
**Objective:** 開発者として、Pasta DSLの変数スコープ（ローカル変数`＄var`、グローバル変数`＄＊var`、システム変数`＄＊＊var`）をRuneコードで正しく実装したい。これにより、データ永続化を実現できる。

#### Acceptance Criteria
1. The Transpiler2 shall ローカル変数（`＄var_name`）をRune関数内のローカル変数として生成する
2. The Transpiler2 shall グローバル変数（`＄＊var_name`）をRuneの共有状態（extern関数経由、またはコンテキストパラメータ）として生成する
3. The Transpiler2 shall システム変数（`＄＊＊var_name`）に専用マーカー（例：メタデータタグ）を付与し、Engine/Runtime層で永続化可能にする（具体的な永続化メカニズム実装はEngine/Runtime層の責任）
4. The Transpiler2 shall 変数代入（`＄var：value`）のRHS値をPasta式として評価する
5. The Transpiler2 shall 変数参照（`＄var`）をRune値として埋め込む（例：`let msg = "Current: {$count}";`→`let msg = format!("Current: {}", count);`）

### Requirement 6: Expression Evaluation and Type Inference
**Objective:** 開発者として、Pasta DSL式（整数、浮動小数点、文字列、演算）をRuneコードで評価可能にしたい。これにより、動的スクリプト機能を実現できる。

**Note:** 式の結果として「Data型」を提供するかは設計フェーズで決定する。実装方針として、(1) 型情報を含むData構造体、(2) 直接Rune型（i64/f64/String）の両案を評価し、既存Runtime層との互換性で決定する。

#### Acceptance Criteria
1. The Transpiler2 shall 整数リテラル（全角`０`〜`９`、半角`0`〜`9`）をRune `i64`値に変換する
2. The Transpiler2 shall 浮動小数点リテラル（小数点含む）をRune `f64`値に変換する
3. The Transpiler2 shall 文字列リテラル（括弧括り：`「text」`、`「「nested」」`等）をRune文字列として処理する
4. The Transpiler2 shall 二項演算式（`+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`）をRune式に変換する
5. The Transpiler2 shall 関数呼び出し（`＠fn_name(arg1, arg2, ...)`）をRune関数呼び出しに展開する
6. The Transpiler2 shall 式の結果を正しく型判定し、Runeコードへ適切に埋め込む

### Requirement 7: Error Handling and Diagnostics
**Objective:** 開発者として、transpiler2のエラーを統一的に処理し、意味のある診断メッセージを提供したい。これにより、デバッグを容易にできる。

**Note:** TranspileError型の具体的な実装形式（error.rsへの追加 vs. transpiler2/error.rs独立モジュール）は設計フェーズで決定する。既存PastaErrorとの統合戦略も設計で詳細化する。

#### Acceptance Criteria
1. The Transpiler2 shall すべてのトランスパイル操作で`Result<T, TranspileError>`を返す（TranspileErrorは新規定義）
2. When Parser2パース結果が無効である、the Transpiler2 shall TranspileError::InvalidAstを返す（通常は発生しない）
3. When シーン・単語が未定義である、the Transpiler2 shall TranspileError::UndefinedSymbolを返す
4. When 式評価がタイプ不正である、the Transpiler2 shall TranspileError::TypeMismatchを返す
5. The Transpiler2エラーメッセージ shall ソース位置（ファイル名、行番号、列番号）を含む
6. The Transpiler2 shall 診断情報としてAST Spanを保有し、エラーメッセージに組み込む

### Requirement 8: Rune Code Output and Runtime Compatibility
**Objective:** 開発者として、生成されたRuneコードが既存Rune VMで正常に実行されることを保証したい。これにより、既存Runtimeとの互換性を確保できる。

#### Acceptance Criteria
1. The Transpiler2 shall Rune 0.14 VMで実行可能な構文として生成する
2. The Transpiler2 shall すべてのyield文を`yield ScriptEvent::Talk { ... }`形式で生成する
3. The Transpiler2 shall generator関数として出力Runeコードを構造化する（`yield`の継続性対応）
4. The Transpiler2 shall Pasta標準ライブラリ（stdlib）の関数シグネチャと一致するコードを生成する
5. The Transpiler2 shall 生成コードがRuntime層（`ScriptGenerator`等）で実行可能であることをユニットテストで検証する

### Requirement 9: Two-Pass Transpilation Architecture
**Objective:** 開発者として、既存の2パストランスパイル設計（Phase 1: 登録、Phase 2: 生成）を正確に実装したい。これにより、フォワードリファレンス対応と順序依存性回避を実現できる。

#### Acceptance Criteria
1. The Transpiler2 shall Phase 1（登録フェーズ）でファイル全体をスキャンし、全シーン・単語・属性を登録する
2. The Transpiler2 shall Phase 2（生成フェーズ）でそれぞれのシーン内容をRuneコードに変換する
3. When Phase 1が完了する、the Transpiler2 shall 全シンボルが解決可能であることを保証する
4. The Transpiler2 shall Phase 1でエラーが発生した場合、Phase 2をスキップする
5. The Transpiler2 shall Phase 1・Phase 2の分離を明確にコードで表現する（`Phase1Context`、`Phase2Generator`等の構造体分離）
6. The Transpiler2 shall レガシーtranspiler（2パス実装）との互換性を保ちながら、新AST型に対応する

### Requirement 10: Full Test Coverage
**Objective:** 開発者として、transpiler2の全機能を網羅的にテストしたい。これにより、品質保証と将来の保守性を確保できる。

#### Acceptance Criteria
1. The Pastaプロジェクト shall 全スコープ構造（FileScope、GlobalSceneScope、LocalSceneScope）の変換を検証するテストを作成する
2. The テストスイート shall シーン呼び出し（Call文）のランダム選択メカニズムを検証する
3. The テストスイート shall 変数スコープ（ローカル・グローバル・システム）の正確性を検証する
4. The テストスイート shall 単語参照の展開を検証する
5. The テストスイート shall 継続行の連結処理を検証する
6. The テストスイート shall Runeコードブロックの埋め込み処理を検証する
7. The テストスイート shall エラーケース（未定義シーン・単語、型不正）を検証する
8. The テストスイート shall `tests/fixtures/`の既存fixtureとpasta2.pestベースの新fixtureを使用する
9. The テストスイート shall 生成されたRuneコードがRuntime層で実行可能であることをE2E検証する（統合テスト）
10. The テストスイート shall transpiler2がparser2 AST出力に対して、レガシーtranspiler相当の結果を生成することを検証する
