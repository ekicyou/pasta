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

**Fixture戦略（承認済み）**: parser2で既にテスト済みのfixtureを最大限流用する。`tests/fixtures/parser2/`の3ファイル（basic_syntax.pasta、string_and_numbers.pasta、escape_sequences.pasta）および`comprehensive_control_flow2.pasta`をベースとし、transpiler固有機能（変数スコープ、シーン呼び出し等）で追加が必要な場合のみ新規fixtureを作成する。

#### Acceptance Criteria
1. The Pastaプロジェクト shall 全スコープ構造（FileScope、GlobalSceneScope、LocalSceneScope）の変換を検証するテストを作成する
2. The テストスイート shall シーン呼び出し（Call文）のランダム選択メカニズムを検証する
3. The テストスイート shall 変数スコープ（ローカル・グローバル・システム）の正確性を検証する
4. The テストスイート shall 単語参照の展開を検証する
5. The テストスイート shall 継続行の連結処理を検証する
6. The テストスイート shall Runeコードブロックの埋め込み処理を検証する
7. The テストスイート shall エラーケース（未定義シーン・単語、型不正）を検証する
8. The テストスイート shall parser2テスト済みfixture（`tests/fixtures/parser2/*.pasta`、`comprehensive_control_flow2.pasta`）を流用し、必要最小限の追加fixtureのみ作成する
9. The テストスイート shall 生成されたRuneコードがRuntime層で実行可能であることをE2E検証する（統合テスト）
10. The テストスイート shall transpiler2がparser2 AST出力に対して、期待されるRune出力を生成することを検証する

### Requirement 11: FileScope Attribute Inheritance
**Objective:** 開発者として、ファイルレベル属性（FileScope.attrs）をすべてのグローバルシーンに継承させたい。これにより、共通属性の一括定義と属性フィルタリングを実現できる。

**背景（parser1→parser2のAST変更）**: parser1にはFileScope自体が存在せず、file-level attributesは処理されなかった。parser2ではFileScope { attrs, words }が導入され、ファイル全体に共通する属性を定義可能になった。

#### Acceptance Criteria
1. The Transpiler2 shall FileScope.attrsを解析し、HashMap<String, String>形式に変換する
2. When グローバルシーンが登録される、the Transpiler2 shall file-level attributesをグローバルシーン属性とmergeする
3. The Transpiler2 shall 属性merge時に、シーンレベル属性を優先する（同一キーの場合、シーン属性がfile属性を上書き）
4. When シーンに属性が定義されていない、the Transpiler2 shall file-level attributesをそのまま継承する
5. The Transpiler2 shall mergeされた属性をSceneRegistry.register_globalの`attributes`引数として渡す
6. The テストスイート shall file-level attributes継承パターン（継承のみ、上書きあり）を検証する

**実装例**:
```pasta
＆天気：晴れ
＆季節：冬

＊会話＆時間：夜＆季節：夏
```
→ シーン「会話」の最終属性: `{天気: "晴れ", 時間: "夜", 季節: "夏"}` (季節はシーンレベルで上書き)

### Requirement 12: Scene and Local Attributes Processing
**Objective:** 開発者として、シーンレベル・ローカルシーンレベルの属性を正しく処理したい。これにより、シーン選択フィルタリング機能の基盤を確立できる。

**背景（旧transpilerとのギャップ）**: 旧transpilerでは`transpile_attributes_to_map()`が常に空HashMap `#{}`を返し、属性機能はP0スコープ外として未実装だった。transpiler2では属性処理を完全実装する。

#### Acceptance Criteria
1. The Transpiler2 shall GlobalSceneScope.attrsを解析し、HashMap<String, String>に変換する
2. The Transpiler2 shall LocalSceneScope.attrsを解析し、HashMap<String, String>に変換する
3. The Transpiler2 shall 属性値の文字列リテラルとエスケープシーケンスを正しく処理する
4. The Transpiler2 shall 属性をSceneRegistry.register_global/register_localの引数として渡す
5. The Transpiler2 shall 属性情報をSceneTable生成時に保持し、Runtime層のフィルタリング機能で利用可能にする
6. The テストスイート shall 属性付きシーン（グローバル・ローカル）の登録を検証する

### Requirement 13: CodeBlock Embedding
**Objective:** 開発者として、Runeコードブロック（` ```rune ... ``` `）を適切な位置にそのまま埋め込みたい。これにより、Pasta DSLで表現できない高度なロジックをRuneで直接記述できる。

**背景（parser1→parser2のAST変更）**: parser1にはcode_blocks機能が存在せず、Runeブロックは処理できなかった。parser2ではGlobalSceneScope/LocalScopeに`code_blocks: Vec<CodeBlock>`が追加され、明示的に扱える。

#### Acceptance Criteria
1. The Transpiler2 shall GlobalSceneScope.code_blocksを検出し、グローバルモジュールレベルにRune codeを出力する
2. The Transpiler2 shall LocalSceneScope.code_blocksを検出し、ローカルシーン関数内にRune codeを出力する
3. The Transpiler2 shall code_blocksの出力位置を正しく制御する（関数定義の前 vs. 後、他statements/itemsとの順序）
4. The Transpiler2 shall code_blocks内のRune構文を一切加工せず、そのまま出力する（transpiler2は構文検証しない）
5. When code_blocksに不正なRune構文が含まれる、the Rune VMのコンパイルエラー shall Transpiler2の責任外として扱う
6. The テストスイート shall code_blocks埋め込みパターン（global/local scope）を検証する

### Requirement 14: Explicit ContinueAction Processing
**Objective:** 開発者として、継続行（ContinueAction）を明示的な`：`prefixで処理したい。これにより、pasta2.pest文法仕様の変更に対応できる。

**背景（pasta.pest→pasta2.pestの文法変更）**: pasta.pestでは継続行に明示的なprefixがなかったが、pasta2.pestでは`continue_action_line`ルールとして`：`または`:`による明示的prefix付きで定義された。parser2 ASTでは`LocalSceneItem::ContinueAction(ContinueAction { actions })`として独立型になった。

#### Acceptance Criteria
1. The Transpiler2 shall LocalSceneItem::ContinueAction型を認識し、ActionLineと別処理する
2. The Transpiler2 shall ContinueAction.actionsを直前のActionLineに連結する（同一yield文として出力）
3. When ContinueActionが最初のitemである（直前にActionLineがない）、the Transpiler2 shall TranspileError::InvalidContinuationを返す
4. The Transpiler2 shall ContinueActionの連結時に、話者（speaker）を継承しない（既存ActionLineの話者を使用）
5. The テストスイート shall 継続行の連結パターン（1行ActionLine + 複数ContinueAction）を検証する

### Requirement 15: FileScope Words Registration
**Objective:** 開発者として、ファイルレベル単語定義（FileScope.words）をグローバル単語として登録したい。これにより、ファイル全体で使用可能な単語セットを定義できる。

**背景（parser1→parser2のAST変更）**: parser1では`PastaFile.global_words`として単一フィールドで管理されたが、parser2では`PastaFile.file_scope.words`に移動した。旧transpilerでは`file.global_words`を処理していたため、同等の機能をfile_scope.wordsで実装する必要がある。

#### Acceptance Criteria
1. The Transpiler2 shall FileScope.words（Vec<KeyWords>）を解析し、すべての単語をWordDefRegistryに登録する
2. The Transpiler2 shall Phase 1（登録フェーズ）でfile_scope.wordsを最初に処理する（グローバルシーンwordsより前）
3. The Transpiler2 shall file_scope.wordsとglobal_scene.wordsの重複チェックを行い、重複時にWarningを発行する（エラーではない）
4. The Transpiler2 shall 登録された単語をRuneの単語選択関数として生成する（既存word_registry.rsのパターンを踏襲）
5. The テストスイート shall file-level words定義と参照を検証する
