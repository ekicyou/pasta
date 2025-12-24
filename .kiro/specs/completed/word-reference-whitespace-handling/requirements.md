# 要件書

## プロジェクト説明（入力）
comprehensive_control_flow.pastaのアクション文「さくら　：＠場所　の天気は＠天気　だね。」を用意したところ、トランスパイラ結果comprehensive_control_flow.transpiled.rnが想定と異なる。

間違い：
        yield Talk(pasta_stdlib::word("", "場所", []));
        yield Talk("　の天気は");
        yield Talk(pasta_stdlib::word("", "天気", []));
        yield Talk("　だね。");

正しい：
        yield Talk(pasta_stdlib::word("", "場所", []));
        yield Talk("の天気は");
        yield Talk(pasta_stdlib::word("", "天気", []));
        yield Talk("だね。");

＠単語　の後に空白区切りが入る場合、パーサーまたはトランスパイラーは空白を無視しなければなりません。

---

## 要件

### 導入

本要件書は、Pastaスクリプトの単語参照（`＠単語名`）および変数参照（`＄変数名`）直後の空白文字が正しく処理されていない問題を解決するための要件を定義します。現状、これらの参照の直後に半角または全角スペースが続く場合、その空白がテキストとして誤って出力されています。これにより、生成されるRuneコードに不要な空白を含むTalkステートメントが挿入され、ランタイム出力に意図しない空白が含まれてしまいます。

**対象要素**:
- `＠単語名`（WordExpansion）: 単語参照
- `＄変数名`（VarRef）: 変数参照

**対象外**:
- さくらスクリプト（`\n`, `\w8`など）: これらは意味を持つエスケープシーケンスであり、空白処理の対象外

### Requirement 1: インライン参照後の空白除去（パーサー層）

**目的**: パーサーは、単語参照（`＠単語名`）および変数参照（`＄変数名`）直後の空白文字を正しく処理し、ASTの`SpeechPart`要素として空白を含めないようにする。これらの空白はトークン区切りであり、出力テキストに含めるべきではない。

#### Acceptance Criteria

1. When パーサーがspeech_content内で`word_expansion`（`＠単語名`形式）または`var_ref`（`＄変数名`形式）をパースし、その直後に半角スペース（U+0020）、全角スペース（U+3000）、またはタブ（U+0009）が続く場合, the Pasta Parserは空白をテキストパート（`text_part`）として分離せず、次の意味のあるトークンまで空白をスキップしなければならない

2. When パーサーがspeech_content内で`word_expansion`または`var_ref`直後に複数の連続した空白文字（半角・全角・タブの任意の組み合わせ）を検出した場合, the Pasta Parserはそれらすべての空白を無視し、次の非空白トークンから`text_part`を開始しなければならない

3. When インライン参照（`word_expansion`または`var_ref`）直後の空白を除去した結果、別の`word_expansion`、`var_ref`、`func_call`、`sakura_script`、または改行が続く場合, the Pasta Parserは空のテキストパートを生成してはならない

4. When インライン参照が行末（改行の直前）に配置され、その間に空白が存在する場合, the Pasta Parserは空白を無視し、改行を正しく認識しなければならない

5. The Pasta Parserは、インライン参照の前にある空白文字を通常のテキストとして扱い、除去してはならない

6. The Pasta Parserは、さくらスクリプト（`sakura_script`）の直後に空白が続く場合でも、その空白を除去してはならない（さくらスクリプトは意味を持つエスケープシーケンスであり、空白処理の対象外）

### Requirement 2: トランスパイラ出力の正確性

**目的**: トランスパイラは、パーサーから受け取ったASTに基づき、インライン参照（単語参照・変数参照）後の空白が含まれていない正しいRuneコードを生成する。

#### Acceptance Criteria

1. When トランスパイラが`SpeechPart::WordExpansion`または`SpeechPart::VarRef`を処理する場合, the Transpilerはそれぞれ適切なRune呼び出しを含む単一の`yield Talk()`ステートメントを生成しなければならない

2. When トランスパイラが`SpeechPart::Text`をインライン参照（WordExpansionまたはVarRef）の直後に処理する場合, the Transpilerは空白で始まるテキストを別の`yield Talk()`に分離せず、先頭の空白を含まない形で出力しなければならない

3. When トランスパイラがspeech_content全体を処理し、インライン参照直後のテキストパートが存在する場合, the Transpilerは生成されるTalkステートメントが連続する空白を含まないことを保証しなければならない

4. The Transpilerは、インライン参照以外の箇所（通常のテキスト、関数呼び出し、さくらスクリプト）における空白文字の処理に影響を与えてはならない

### Requirement 3: テスト検証

**目的**: パーサーおよびトランスパイラの修正が期待通りに動作し、既存機能に回帰が発生しないことを検証する。

#### Acceptance Criteria

1. When 「さくら　：＠場所　の天気は＠天気　だね。」という入力をパースした場合, the Pasta Parserは4つのSpeechPart要素（WordExpansion("場所"), Text("の天気は"), WordExpansion("天気"), Text("だね。")）を生成しなければならない（注: 空白を含むTextパートが生成されてはならない）

2. When 「太郎：＄名前　さんこんにちは」という入力をパースした場合, the Pasta Parserは3つのSpeechPart要素（VarRef("名前", Local), Text("さんこんにちは")）を生成しなければならない（注: VarRef直後の空白は除去される）

3. When AC 1のパース結果をトランスパイルした場合, the Transpilerは以下の出力を生成しなければならない:
   ```
   yield Talk(pasta_stdlib::word("", "場所", []));
   yield Talk("の天気は");
   yield Talk(pasta_stdlib::word("", "天気", []));
   yield Talk("だね。");
   ```

4. When AC 2のパース結果をトランスパイルした場合, the Transpilerは以下の出力を生成しなければならない:
   ```
   yield Talk(`${ctx.local.名前}`);
   yield Talk("さんこんにちは");
   ```

3. When 既存のテストスイート（`cargo test --all`）を実行した場合, the Systemはすべてのテストが成功しなければならない（回帰がないことを確認）

4. When comprehensive_control_flow.pastaおよび類似のフィクスチャファイルを処理した場合, the Systemは単語参照後の空白を含まない正しい出力を生成しなければならない

### Requirement 4: 文法仕様との整合性

**目的**: パーサーの修正がSPECIFICATION.mdおよびgrammar.mdで定義された文法仕様と整合性を保つことを保証する。

#### Acceptance Criteria

1. When パーサーがspeech_contentルールを処理する場合, the Pasta Parserは`text_part`, `word_expansion`, `var_ref`, `func_call`, `sakura_script`の優先順位と境界を正しく認識しなければならない

2. The Pasta Parserは、`text_part`ルールが`＠`（at_marker）および`＄`（dollar_marker）を除外する既存の定義を維持しなければならない

3. When パーサーがWHITESPACE定義（半角・全角スペース、タブ、その他Unicode空白）を解釈する場合, the Pasta Parserは単語参照直後のコンテキストでこれらを明示的にスキップしなければならない

4. The Pasta Parserは、単語参照の構文（`＠単語名`形式、括弧なし）と関数呼び出しの構文（`＠関数名()`形式、括弧あり）を明確に区別し続けなければならない

