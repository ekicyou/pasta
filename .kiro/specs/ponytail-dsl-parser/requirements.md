# Requirements Document

## Project Description (Input)
ゴーストの作者と、辞書を読むすべての道具（pasta.dll・`pasta_check`・VSCode 拡張）が、辞書の解析で困っている（2026-10-19 の `/ponytail:ponytail-audit` の指摘より）。大きい辞書ほど読み込みが行数の 2 乗で遅くなり、型に入らない大きな数は書いた場所ごとに違う形で黙って化け、括弧を深く入れ子にした辞書はスタックがあふれて SSP ごと落ちる。加えて、作られない誤りの型と、誤って計算されテストだけが読む `PastaFile.span` が残っている。`crates/pasta_dsl` のパーサ（`parser/ast/span.rs`・`parse_scene.rs`・`parse_action.rs`・`parse_elements.rs`・`parser/mod.rs`・`error.rs`・`lib.rs` の再輸出。`partial.rs` は除く）の監査の指摘を残らず直し、解析の時間が辞書の大きさに比例し（span の値は今と同じ）、大きな数はどこに書いても同じ規則で値になり、深い入れ子はプロセスを落とさず位置つきの構文の誤りになり、作られない誤りの型と誤った `PastaFile.span` が無くなるようにする。

## Introduction

この spec は、2026-10-19 の監査が `pasta_dsl` のパーサで見つけた指摘の 1〜5 番と、ギャップ分析（research.md）で同じ領域に増えた指摘を、残らず直す。直す中身は 2 種類ある。

- **挙動を変える直し**：大きな数の値の決め方、深い入れ子と長い式を構文の誤りにすること、シーン引数の番号とアクターの番号の範囲外を構文の誤りにすること。
- **挙動を変えない直し**：解析の速さ（span の値は変えない）、作られない誤りの型・誤った `PastaFile.span`・小物の撤去。

要件は「その領域の指摘が残らず直っている」ことと、挙動を変える直しの決めごとである。どう直すか（関数・ファイルの形）は設計で決める。

> brief.md だけでは決めきれない論点は、推奨案を【前提】として書いた。research.md の「要件ディスカッションへの持ち越し」に、案と推奨の理由を並べてある。要件ディスカッションで確定させる。

## Boundary Context

- **In scope**
  - 監査の 1〜5 番（解析が 2 乗に遅い・大きな数の字句が化ける・作られない誤りの型・誤った `PastaFile.span`・深い括弧の入れ子でスタックがあふれる）。
  - ギャップ分析で増えた、同じ領域の指摘（要件 2.5・2.8・3.2・6.1〜6.4）。名前つきの引数・Call の動的ターゲットの大きな数（要件 2.4 の例）は、監査の 2 番と根が同じなので 2 番に含める。
  - 直した挙動に合わせた、マニュアルの頁・`crates/pasta_dsl/README.md`・`lib.rs` の説明の直しと、スキルの参照の作り直し。
- **Out of scope**
  - 部分解析（`crates/pasta_dsl/src/partial.rs`）と、その誤りの出し方・位置の出し方。`ponytail-editor-tooling` が持つ。ただし `partial.rs` は内部で `parse_str` を呼ぶので、要件 3 の守りは部分解析にも効く（`partial.rs` は直さない）。
  - 文法（`grammar.pest`）の意味の変更。受け付ける字句の形は変えない。範囲外の値を構文の誤りにするのは、字句の形でなく値の検査である。
  - `pasta_core` の型（`ponytail-core-registry`）。
  - `pasta_lua` のコード生成（`crates/pasta_lua/src/code_gen/`。`ponytail-lua-transpiler`）。research.md に書いた申し送り（`＄２５５` の番号の桁あふれ・`!select` の大きな小数）は、その spec が直す。
  - `review-improvement-loop` の 3.3 の既知のバグ（括弧式の第 1 項だけを包む）。2026-10-04 の `14be2195` で直っており、拾う作業は無い。回帰テストを保つことだけを要件 8.2 に書く。
- **Adjacent expectations**
  - `parse_str` の誤りの形（`ParseError::SyntaxError` の `file`・`line`・`column`・`message`）を保つ。`ponytail-editor-tooling` の診断と `pasta_check validate` の `syntax` の報告が、これを読む。
  - `pasta_lua` のトランスパイラとデバッガのソースの対応表は、AST の span の値を使う。この spec は span の値を変えない（`ponytail-lua-transpiler` も「値は変わらない」を前提にしている）。
  - `pest-2-9-4-upgrade` とは同じウェーブに置かない（pest の行の索引を使うため）。

## Requirements

### Requirement 1: 解析の時間を辞書の大きさに比例させる

**Objective:** As a ゴーストの作者, I want 大きい辞書でも、読み込みの時間が行数に比例してほしい, so that 辞書を育てるほど起動・検査・編集が遅くなる、ということが無い。

#### Acceptance Criteria

1. When 1 シーン 4 行の辞書 8,000 行を、release の構成で解析する, the Pasta Parser shall 1 秒未満で解析を終える（2026-11-02 の測定で、今は 2.26 秒。brief の測定では 10.6 秒）。
2. When 同じ形の辞書を 8,000 行から 32,000 行に増やす, the Pasta Parser shall 解析の時間の伸びを 8 倍以内に収める（行数に比例すれば 4 倍、今の 2 乗なら 16 倍）。
3. The Pasta Parser shall すべての AST の節の span（始まりと終わりの行・桁、始まりと終わりのバイトの位置）を、直す前と同じ値にする。改行が CRLF の辞書、先頭に BOM のある辞書、全角の文字や絵文字を含む辞書も同じとする。
4. The Pasta Parser shall 文法の照合で見つかる構文の誤りの位置と文言を、直す前と同じにする。

### Requirement 2: 大きな数の字句を、どこに書いても同じ規則で値にする

**Objective:** As a ゴーストの作者, I want 大きな数を書いたとき、書いた場所によらず同じ値になってほしい, so that 値が黙って空や 0 に化けたり、引数や演算子が消えたりしない。

#### Acceptance Criteria

1. When 小数点の無い数の字句が 64 ビットの符号つき整数（-9223372036854775808〜9223372036854775807）に収まる, the Pasta Parser shall 今と同じく、その整数の値にする。
2. When 小数点の無い数の字句が 64 ビットの符号つき整数に収まらない, the Pasta Parser shall その数にいちばん近い小数（64 ビットの浮動小数点数）の値にする（Lua と同じ）。例：`＄x＝10000000000000000000` の値は `1e19`（今は空の文字列）。
3. The Pasta Parser shall 1 と 2 の規則を、式（変数の代入の右辺・位置の引数・名前つきの引数・Call の動的ターゲット・括弧の中）、属性値、キューコマンドの引数のどこでも同じに使う。
4. When 式・引数の並びに大きな数を含む, the Pasta Parser shall 項・演算子・引数を 1 つも落とさず、並びの位置もずらさない。例：`＄y＝1＋10000000000000000000` は `1 ＋ 1e19`（今は `1`）、`＞t（10000000000000000000、2）` の引数は 2 つ（今は 1 つで、`＄０` が 2 にずれる）、`!wait(99999999999999999999)` の引数は `1e20`（今は `0`）、`＆n：99999999999999999999` は小数の `1e20`（今は整数の `0`）。
5. If 数の字句（小数点の有無を問わない）が、有限の 64 ビットの浮動小数点数に収まらない（およそ 1.8×10^308 を超える）, then the Pasta Parser shall その字句の位置の構文の誤りにする（【前提】持ち越し 4。今は空の文字列か、Lua で `nil` になる `inf` になる）。
6. The Pasta Parser shall 小数点のある数の字句を、今と同じく小数の値にする。
7. If シーンのアクター指定行の番号（`％名前＝数字`）が 0〜4294967295 に収まらない, then the Pasta Parser shall その番号の位置の構文の誤りにする（【前提】持ち越し 1。今は番号の指定が黙って無視され、自動の番号になる。例：`％a、b＝4294967296` は今、番号が 0・1 になる）。範囲の中の番号と、番号を省いたときの採番は、今と同じとする（4294967295 の次の省いた番号が 4294967295 に留まる扱いも含む）。
8. If シーン引数の番号（`＄２５６` など、数字だけの変数名）が 0〜255 に収まらない, then the Pasta Parser shall その番号の位置の構文の誤りにする（【前提】持ち越し 5。今はマニュアルのとおり `＄０` として扱われる）。範囲の中の番号は、今と同じとする。
9. When 2・5・7・8 の誤りか値の変化が起きる辞書を、`pasta_check validate` で検査する, the `pasta_check validate` shall pasta.dll と同じ結果（5・7・8 は行・桁つきの `syntax` の誤り、2 は誤り無し）を報告する。

### Requirement 3: 深い入れ子と長い式で、プロセスを落とさない

**Objective:** As a ゴーストの作者, I want 括弧を深く入れ子にした辞書や、極端に長い式の辞書を読んでも、SSP や VSCode が落ちないでほしい, so that 書き間違いや悪意のある辞書で、ベースウェアごと異常終了しない。

#### Acceptance Criteria

1. If 1 行の中で、閉じていない開き括弧（`（`・`(`）の重なりが 64 段を超える, then the Pasta Parser shall スタックをあふれさせずに、65 段目の開き括弧の行と桁を位置にした構文の誤りにする（【前提】持ち越し 2。数えるのは、台詞・文字列・コメント・Lua ブロックの中を含む、その行のすべての括弧とする）。
2. If 1 つの式の二項演算子が 256 個を超える, then the Pasta Parser shall 257 個目の演算子の行と桁を位置にした構文の誤りにする（【前提】持ち越し 3。ギャップ分析で増えた指摘。今は `＋1` を 1 万個つないだ 1 行で、解析の後の片付けがスタックをあふれさせる）。
3. While 括弧の重なりが 64 段以内で、1 つの式の二項演算子が 256 個以内である, the Pasta Parser shall 今と同じ AST を作る。リポジトリのすべての辞書・テストの辞書・マニュアルの例は、1 と 2 の誤りにならない（2026-11-02 の測定で、いちばん深いものは 8 段）。
4. When 1 か 2 の誤りになる辞書を含むゴーストを pasta.dll が読み込む, the Pasta Parser shall プロセスを異常終了させず、ほかの構文の誤りと同じ経路の読み込みエラーとして作者へ届ける。
5. When 1 か 2 の誤りになる文書を VSCode 拡張の解析（全体の解析と、それに続く部分解析）に渡す, the Pasta Parser shall 拡張の解析の処理を異常終了させない（部分解析は `parse_str` を呼ぶので、`partial.rs` を直さずに守られる）。
6. The Pasta Parser shall 1 と 2 の上限を、release と debug のどちらの構成でも、1 MiB のスタックのスレッドの解析・AST の片付け・Lua への変換があふれない値にする。

### Requirement 4: 誤りの形を保ち、作られない誤りの型を無くす

**Objective:** As a pasta の開発者, I want 本番で作られない誤りの型が無くなり、新しい誤りも今の誤りと同じ形で出てほしい, so that 誤りを読む道具（診断・`pasta_check`）が、型や文言の違いで取りこぼさない。

#### Acceptance Criteria

1. The pasta_dsl crate shall `ParseError::PestError`・`ParseError::MultipleErrors`・`ParseErrorInfo`・`ParseResult`、コンストラクタの `syntax_error`・`pest_error`・`io_error` を提供しない。
2. The pasta_dsl crate shall `ParseError::SyntaxError`（`file`・`line`・`column`・`message`）と、`parse_file` が読めないファイルに返す `ParseError::IoError` を、今と同じ形で提供する。
3. When 要件 2.5・2.7・2.8・3.1・3.2 の誤りになる, the Pasta Parser shall `ParseError::SyntaxError` を返し、`line` と `column` を 1 から数えた行と桁（桁は文字の数）にし、`message` を文法の照合の誤りと同じく `Parse error in {ファイル名} at {行}:{桁}: ` で始める。文の残りには、何が範囲を超えたかと上限の値を書く。
4. When `pasta_check validate` が 3 の誤りを報告する, the `pasta_check validate` shall 今の `syntax` の報告と同じく、行・桁と、前置きを除いた説明を出す（`pasta_check` のコードは直さずに済む）。

### Requirement 5: 誤って計算される `PastaFile.span` を無くす

**Objective:** As a pasta の開発者, I want ファイル全体の span という、誤って計算されテストだけが読む値が無くなってほしい, so that 日本語の辞書で誤った桁を返す値に、誰かが頼り始めない。

#### Acceptance Criteria

1. The pasta_dsl crate shall `PastaFile` のファイル全体の span を提供しない（【前提】持ち越し 6。今は終わりの桁がバイト長で、日本語では誤る。本番で読む所は無い）。
2. When 1 の撤去で、ファイル全体の span を確かめていたテスト（`error_api_test.rs`・`span_byte_offset_test.rs`）を直した, the pasta_dsl test suite shall 同じ観点（空の辞書・コメントだけの辞書の解析が成功すること、各節のバイトの位置が元の文に合うこと）を、各節の span で確かめる。

### Requirement 6: 使われない部品と、隠れた拾い方を無くす

**Objective:** As a pasta の開発者・読み手, I want パーサから、本番で使われない部品と、扱っていない規則を黙って拾う書き方が無くなってほしい, so that 読む手間が減り、次の文法の変更で同じ種類の不具合（扱い忘れの規則が別の値に化ける）が起きない。

#### Acceptance Criteria

1. If 式の項として扱っていない規則が、項の位置に来る, then the Pasta Parser shall その規則の中の最初の値を、項として黙って拾わない（今は、扱いの無い規則の中をたどり、最初に見つかった値を項にする。動的な単語参照・動的な関数呼び出しで、すでに 2 回この形の不具合が出ている）。文法が今作る入力について、AST は今と同じとする（ギャップ分析で増えた指摘）。
2. The pasta_dsl crate shall `Span::byte_len` を提供しない（テストだけが使う。ギャップ分析で増えた指摘）。
3. The Pasta Parser shall キューコマンドのスコープ（`!cmd@アクター：名前`）の分け方を、起きない場合のための守りと届かない戻り値を持たない形にし、結果は今と同じにする（ギャップ分析で増えた指摘）。
4. The Pasta Parser shall AST を組み立てるときの、同じ扱いを 2 度書いた分岐（ファイルの終わりの規則と、それ以外の規則の両方を何もせずに読み飛ばす分岐）を 1 つにする（ギャップ分析で増えた指摘）。

### Requirement 7: マニュアルと説明を、直した挙動に合わせる

**Objective:** As a ゴーストの作者・pasta の開発者, I want マニュアルと crate の説明が、直したあとの挙動と型のとおりに書かれていてほしい, so that 書いてあることと動きが食い違わない。

#### Acceptance Criteria

1. When 要件 2 と 3 の挙動を変える直しが入る, the pasta manual shall 同じ変更で、その挙動を書いた頁を直す。少なくとも次の所である。
   - `grammar/literals.md` の「数値（i64 / f64）」（64 ビットの整数に収まらない整数は小数になり、下の桁が丸められること。有限の小数に収まらない数は構文の誤りになること）
   - `grammar/variables.md` のシーン引数の「番号は 0〜255」の行（256 以上は構文の誤りになること）と、「優先順位と括弧」の節（括弧の重なりと演算子の数の上限）
   - `grammar/actor-dictionary.md` の `％` 行の番号の行（0〜4294967295 を超える番号は構文の誤りになること）
   - `internals/transpiler.md` の「パースと AST 構築時の正規化」（数の値の規則・範囲の検査・深さの守り）
2. When 要件 4〜6 の撤去が入る, the change shall `crates/pasta_dsl/README.md` の誤りの型の一覧と、`lib.rs` の説明（`ParseErrorInfo` の記述）を、今の型に書き直す。
3. When スキルの参照の元になる頁を直した, the pasta manual shall `node book/tools/gen-skill-refs.mjs` を走らせ、生成物を同じ変更に入れる（`--check` で差分が無い）。
4. The change shall README と `.kiro/steering/tech.md` に、公開クレートの pub API の断り書きを足さない（`ponytail-lua-loader` が書いた。`README.md` 88 行・`tech.md` 178 行）。

### Requirement 8: 直していない挙動を保ち、品質の関門を通す

**Objective:** As a pasta の開発者, I want 直しのほかの解析の挙動が変わらず、決まった関門を通ってほしい, so that 整理のついでに別の不具合が入らない。

#### Acceptance Criteria

1. The Pasta Parser shall 要件 2・3 で決めた変化のほかは、正しい辞書から今と同じ AST を作る。リポジトリのすべての `.pasta`（サンプルゴースト・テストの辞書）で、直す前と直した後の AST を比べて確かめる。
2. The pasta_dsl test suite shall 括弧式の中の演算を全体で包むことの回帰テスト（`expr_parse_test.rs` の `test_paren_binary_keeps_all_terms`）を保ち、通す。
3. When 挙動を変える直し・変えない直しを入れる, the change shall 振る舞いを固めるテスト（span の値・AST・誤りの形）を先に置き、1 つ直すごとに確かめて 1 コミットにする。挙動を変える直しは、直す前の誤りを再現するテストを先に置き、直した後にそのテストが通る。
4. The change shall `cargo fmt --check`、`cargo clippy --all-targets`（警告 0）、`cargo test -p pasta_dsl -p pasta_lua -p pasta_check -p pasta_lsp` を通る。
5. The change shall `crates/pasta_dsl/src/partial.rs` を変えない。
