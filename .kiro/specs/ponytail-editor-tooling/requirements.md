# Requirements Document

## Project Description (Input)
VSCode で辞書を書くゴーストの作者が困っている（2026-10-19 の `/ponytail:ponytail-audit` の指摘より）。書き間違いが 1 つあるだけで、シーンの全行に、ずれた位置で波線が付き、保存時の正しい誤りは隠れてしまう。単語参照の箱、行末のコメント、Lua のフェンスの色は、本当の文法とずれる。加えて、`pasta_lsp` は本番では言語サーバとして使われていないのに、tower-lsp のサーバ（約 850 行と依存 10 個ほど）を抱えている。`crates/pasta_lsp`・`editors/vscode`・`crates/pasta_dsl/src/partial.rs`・`crates/pasta_check/src/validate/{syntax,report}.rs` の監査の指摘を残らず直し、構文の誤りは本当の行と桁に 1 件だけ保存時と同じ文言で出て、誤りの後ろのシーンも色が残り、箱・コメント・属性・フェンスの色が本当の文法と一致し、`pasta_lsp` は WASM の解析器だけになり、`launch.json` の `host` は廃止されるようにする。

## Introduction

この spec は、2026-10-19 の監査が VSCode まわり（`crates/pasta_lsp`・`editors/vscode`・`crates/pasta_dsl/src/partial.rs`・`crates/pasta_check/src/validate/{syntax,report}.rs`）で見つけた 13 の指摘と、ギャップ分析で同じ領域に増えた指摘を、残らず直す。直す中身は 2 種類ある。

- **挙動を変える直し**：作者の手元で見える困りごと。構文の誤りの波線、色（単語参照の箱・コメント・属性・フェンス・エスケープ）、`launch.json` の `host`、古い色が残るキャッシュ。
- **挙動を変えない撤去**：使われない言語サーバ・死んだコード・同語反復・テストだけが使う部品。

要件は「その領域の指摘が残らず直っている」ことと、挙動を変える直しの決めごとである。どう直すか（関数・ファイル・正規表現の形）は設計で決める。

> brief.md だけでは決めきれない論点は、【前提】と書いて最善の案で書いた。要件ディスカッションで確かめる（research.md の「要件ディスカッションへの持ち越し」）。

用語:

- **入力のたびの診断**：開いている `.pasta` を編集するたびに、拡張が WASM の解析器で出す診断。
- **保存時の診断**：保存などで走るゴースト全体の検査（`pasta_check validate` と同じ核）の診断。
- **意味の色**：解析器が返すセマンティックトークンの色。
- **TextMate の色**：`pasta.tmLanguage.json` の色。WASM が読めないときと、意味の色が無い所に出る。マニュアルのハイライトも同じ文法を読む。

## Boundary Context

- **In scope**
  - 監査の 1〜13 番と、ギャップ分析で増えた同じ領域の指摘（research.md の N1〜N12）。
  - 直しに合わせた、テスト（`crates/pasta_lsp/tests/*`・`crates/pasta_dsl/tests/partial_parse_test.rs`・`crates/pasta_dsl/tests/cue_cmd_parse_test.rs` の `infer_rule_from_line` の 4 つのテスト（350〜379 行）・`editors/vscode/src/test/*`）の直し。
  - 直しに合わせた、マニュアル（`book/src/debug/{source-level,troubleshooting,vscode-setup}.md` の `host`、`debug/validate.md` の「エディタの診断」、`internals/talk-output.md` の写しの記述）・`crates/pasta_lsp/README.md`・`TEST_COVERAGE.md`・`.kiro/steering/tech.md`・`structure.md` の直し。
  - ルートの `Cargo.toml`・`Cargo.lock`（tower-lsp の行。このウェーブではこの spec が席を持つ）。
- **Out of scope**
  - 本文の解析（`pasta_dsl/src/parser/**`。`ponytail-dsl-parser`）。文法そのものは変えない。色と診断を文法に合わせる。
  - デバッガの本体（`pasta_lua/src/debug`。`ponytail-lua-debug`）。待ち受けが 127.0.0.1 に固定であることは、あちらの持ち物で、この spec は前提にするだけである。
  - マニュアルのハイライトの CI（`ponytail-manual-ci`）。文法の変更でマニュアルを再公開する path filter は、あちらで足す。
  - 選択肢の行の意味の色（今は「将来実装」で、色を足すのは機能の追加になる）。TextMate の色のずれだけを直す（要件 5.7）。
  - `pasta_lsp` のクレート名の変更。
- **Adjacent expectations**
  - `ponytail-dsl-parser` は、`parse_str` の誤りの形（行・桁・文）を保つ。入力のたびの診断は、その形を読む（要件 11.1）。`partial.rs` はこの spec が持つ。
  - `ponytail-core-registry` とは、ルートの `Cargo.toml`・`Cargo.lock` を共有する。消す行が別（tower-lsp と fast_radix_trie）なので、後から入る側が `Cargo.lock` を作り直す。
  - `ponytail-lua-debug` とは、`book/src/debug/*.md` の別の節を触る。後から入る側が rebase で合わせる。`ponytail-manual-ci` とは、`tech.md` の別の行を触る。
  - 「公開クレートの pub API は予告なく消すことがある」の断り書きは、`README.md:88`・`tech.md:178` に既にある（`ponytail-lua-loader` が書いた）。この spec は足さない。

## Requirements

### Requirement 1: 構文の誤りを、本当の位置に 1 件だけ出す

**Objective:** As a ゴーストの作者, I want 書き間違いが 1 つなら、波線がその場所に 1 つだけ付いてほしい, so that どこを直せばよいかが一目で分かり、正しい行に惑わされない。

#### Acceptance Criteria

1. When 開いている `.pasta` に構文の誤りがある, the VSCode Extension shall 入力のたびの診断として、構文の誤りを 1 件だけ出す。
2. When 次の 5 行（4 行目だけが誤り）を開いた, the VSCode Extension shall 4 行目の 4 文字目（2 つ目の `＞`）に、誤りを 1 件だけ出す。2・3・5 行目には誤りを出さない。
   ```
   ＊挨拶
     さくら：やあ
     うにゅう：おう
     ＞＞
     さくら：ね
   ```
3. The VSCode Extension shall 入力のたびの診断の位置を、`parse_str` が返す行と桁（1 文字を 1 と数える）に当たるエディタの位置にする。サロゲートペアの文字（例 `𠮷`）を含む行でも、桁がずれない。
4. The VSCode Extension shall 入力のたびの診断の文を、同じ内容のファイルに対する `pasta_check validate` の syntax の理由と同じにする（例 `構文エラー: expected id or call_target_expr`）。`<partial>`・`-->`・pest の複数行の枠は出さない。
5. When 誤りがファイルの頭（1 行目）・最後の行・改行で終わらない最後の行・CRLF の行にある, the VSCode Extension shall 1〜4 と同じく、本当の位置に 1 件だけ出す。
6. When 作者が誤りを直した, the VSCode Extension shall 次の解析で、その診断を消す。
7. While `.pasta` に構文の誤りが無い, the VSCode Extension shall 構文の診断を出さない（今と同じ）。
8. 【前提】The VSCode Extension shall 入力のたびの診断を、保存時の診断と同じく、幅の無い点の範囲で出す（OQ-2）。

### Requirement 2: 入力のたびの診断と保存時の診断を、重ねず食い違わせない

**Objective:** As a ゴーストの作者, I want 開いている辞書でも閉じた辞書でも、同じ誤りが同じ所に同じ文で 1 件だけ出てほしい, so that 問題パネルの二重の報告や、正しい報告が隠れることに悩まない。

#### Acceptance Criteria

1. While `.pasta` が開いている, the VSCode Extension shall そのファイルの構文の誤りを、入力のたびの診断の 1 件だけで出す（保存時の診断の syntax は出さない。今の規則を保つ）。
2. When 構文の誤りがある `.pasta` を閉じた, the VSCode Extension shall 保存時の診断の syntax を出す（今の規則を保つ）。
3. When 開いている `.pasta` の中身が保存した中身と同じである, the VSCode Extension shall 入力のたびの診断を、保存時の診断と同じ行・桁・文にする。

### Requirement 3: 誤りがあっても、ほかの部分の色を残す

**Objective:** As a ゴーストの作者, I want 書きかけの誤りがあっても、ほかのシーンの色がそのまま付いていてほしい, so that 書いている途中で辞書全体の見た目が崩れない。

#### Acceptance Criteria

1. When 1 つのシーンに構文の誤りがある, the Pasta Analyzer shall 誤りの前と後ろのシーン・アクター辞書・ファイルの単語定義・ファイルの属性に、誤りが無いときと同じ意味の色を付ける。
2. When 誤りのあるシーンが、字下げした `　％さくら、うにゅう`・`　＠挨拶：こんにちは`・`　＠？はい「はい」`・`　＆朝：1` を含む, the Pasta Analyzer shall それらの行をシーンの区切りとみなさない。そのシーンのアクター・単語・項目を、ほかのシーンへ散らさない。
3. 【前提】While シーンに構文の誤りがある, the Pasta Analyzer shall そのシーン（区切りの行から次の区切りの行の手前まで）に意味の色を付けず、TextMate の色に任せる（OQ-1）。
4. The Pasta Analyzer shall 誤りが無い `.pasta` の意味の色を、今と同じにする（要件 4〜5 で直す所を除く）。

### Requirement 4: コメント・属性・名前の色を、文法に合わせる

**Objective:** As a ゴーストの作者, I want 行末のコメントと属性が、シーン名の色でなく、それぞれの色で見えてほしい, so that マニュアルが書いてよいとする書き方が、エディタで間違いのように見えない。

#### Acceptance Criteria

1. When グローバルシーン・ローカルシーン・アクター辞書の宣言の行に行末のコメントがある（例 `＊OnTest ＃ メモ`）, the Pasta Analyzer shall 名前だけを名前の色にし、`＃` から行末までをコメントの色にする。
2. When シーンの宣言の行に属性がある（例 `＊挨拶 ＆朝：1`・`・ローカル ＆朝：1`）, the Pasta Analyzer shall 属性を属性の色にし、名前の色に含めない。
3. When マニュアル（`book/src/grammar/block-structure.md` の「行末コメント」）が行末のコメントを書けるとする行に、`＃` か `#` からの行末のコメントがある, the Pasta Analyzer shall その部分をコメントの色にする。
4. When 文法が本文として読む `＃`・`#`（アクション行・継続行の台詞、値を囲んでいない単語定義の値）がある, the Pasta Analyzer shall そこをコメントの色にしない（今と同じ）。
5. While Lua ブロックの中である, the Pasta Analyzer shall `#`・`＃` で始まる行をコメントの色にしない（Lua の本文の色を消さない）。
6. The TextMate Grammar shall 1〜5 と同じ範囲を、同じ種類の色（名前・属性・コメント・Lua）にする（WASM が読めないときと、意味の色が無い所に出る色）。

### Requirement 5: 単語参照の箱とエスケープの色を、文法に合わせる

**Objective:** As a ゴーストの作者, I want 単語参照の箱が、本当に単語として引かれる所にだけ付いてほしい, so that 台詞のどこまでが単語の名前かを、箱を見て正しく判断できる。

#### Acceptance Criteria

1. When アクション行の台詞に単語参照がある（例 `＠挨拶。こんにちは`）, the VSCode Extension shall 文法が単語参照として読む範囲（`＠` と、識別子の文字が続く所までの名前。例 `＠挨拶`）にだけ箱を付ける。
2. If `＠` がエスケープの `＠＠` の一部である（例 `＠＠メール`）, then the VSCode Extension shall 箱を付けない。`＠＠＠メール` では、エスケープの後ろの `＠メール` にだけ箱を付ける。
3. The VSCode Extension shall 単語定義の行・コメント（行全体と行末）・Lua ブロックの中・キューコマンドのスコープ（`！コマンド名＠スコープ`）の `＠` に、箱を付けない（OQ-5）。
4. The TextMate Grammar shall 単語参照・動的な単語参照の色を、1〜2 と同じ範囲に付ける。
5. The TextMate Grammar shall 文法のエスケープ（`＠＠`・`＄＄`・`\\`・`\%`）だけをエスケープの色にする。`＄＄x` の `＄x` を変数参照の色にしない。文法が受け付けない `\＠`・`\＄`・`\＞` などをエスケープの色にしない（OQ-7）。
6. The TextMate Grammar shall 決して当たらない規則（`inline-fn-call`）を持たない。
7. 【前提】When 選択肢の行（例 `　＠？はい「はい」`）がある, the TextMate Grammar shall その行を単語定義の色にしない（OQ-6）。

### Requirement 6: Lua のフェンスの色を、文法に合わせる

**Objective:** As a ゴーストの作者, I want Lua ブロックの始まりと終わりが、パーサと同じ所で色分けされてほしい, so that 閉じたつもりのブロックの後ろが Lua の色のままにならない。

#### Acceptance Criteria

1. When 行頭から 3 つ以上続く `` ` `` と任意の言語名で Lua ブロックを開いた, the TextMate Grammar shall その次の行から、同じ数の `` ` `` の閉じのフェンスの手前までを Lua の色にする。
2. When 閉じのフェンスの後ろに空白か行末のコメントがある（例 ```` ``` ＃おわり ````）, the TextMate Grammar shall そこでブロックを閉じる。
3. When 4 つ以上の `` ` `` で開いた, the TextMate Grammar shall 中の 3 つの `` ` `` ではブロックを閉じない。
4. 【前提】If 文法が受け付けない開きのフェンスである（字下げした `` ``` ``・言語名の後ろの空白）, then the TextMate Grammar shall それを Lua ブロックの開きにしない（N5）。
5. The Pasta Analyzer shall 開きと閉じのフェンスの行にだけ意味の色を付け、本文を Lua の色に任せる（今と同じ）。

### Requirement 7: 意味の色の凡例を、2 か所で食い違わせない

**Objective:** As a pasta の開発者, I want 解析器と拡張のトークンの種類の並びが、食い違ったらすぐ分かってほしい, so that 片方だけ直して色が別の種類に化けることが起きない。

#### Acceptance Criteria

1. If 解析器と拡張のトークンの種類か修飾子の並びが食い違う, then the quality gate shall テストで落ちる。
2. The VSCode Extension shall `package.json` の `semanticTokenScopes` が、すべてのトークンの種類を覆うことを、今と同じく確かめる。

### Requirement 8: 古い版の色を返さない

**Objective:** As a ゴーストの作者, I want 編集した直後に、今の中身の色が付いてほしい, so that 前の中身の色が別の行にずれて残らない。

#### Acceptance Criteria

1. When 意味の色を求められた, the VSCode Extension shall 今の版の文書の解析結果を返す。前の版の結果や、別の文書の結果を返さない。
2. If 今の版の解析に失敗した, then the VSCode Extension shall 前の版の意味の色を返さない（TextMate の色に任せる）。
3. When 文書を閉じた, the VSCode Extension shall その文書の色と診断の控えを消す（今と同じ）。

### Requirement 9: `pasta_lsp` を WASM の解析器だけにする

**Objective:** As a pasta の開発者, I want 本番で使われない言語サーバと、その依存が無くなってほしい, so that 読むコード・ビルドの時間・依存の更新の手間が減る。

#### Acceptance Criteria

1. The pasta_lsp crate shall 言語サーバ（`server.rs`・`document.rs`・`error.rs`・native の空の入口）と、それらだけを確かめるテスト（`lsp_lifecycle_test.rs`・`document_sync_test.rs`・`crash_recovery_test.rs`）を持たない。
2. The pasta_lsp crate shall `tower-lsp` と `thiserror` に依存しない。ルートの `Cargo.toml` は `tower-lsp` の行を持たない（OQ-4）。
3. The pasta_lsp crate shall crates.io へ出さない設定（`publish = false`）にする。リリースの手順（`release.yml` の 5 つのクレート）は変えない。
4. The pasta_lsp crate shall WASM の 3 つの入口（`wasm_analyze`・`wasm_select_ghost_files`・`wasm_validate_ghost`）の名前と、JS へ返す形（トークンと診断の形・ゴースト全体の報告の形）を、今と同じに保つ。
5. If 解析器が panic した, then the VSCode Extension shall 拡張を落とさず、出力チャンネルに理由を書き、TextMate の色を残す（今の守り）。wasm32 で効かない `catch_unwind` の守りは持たない。
6. The pasta_check crate shall serde の名前と重なる `FindingKind::code()` と、テストだけが使う `FindingKind::ALL` を持たない。報告の `code`（kebab-case の文字列）は、今と同じに保つ。
7. The pasta_lsp crate shall テストだけが使う変換（`WasmAnalysisResult::from_analysis`）と、本番で出ない重さ（Warning・Information・Hint）の対応を持たない（N9）。

### Requirement 10: デバッグの `host` を廃止する

**Objective:** As a ゴーストの作者, I want 使い道の無い `host` の設定が無くなってほしい, so that 別の値を書いてもつながらない理由に迷わない。

#### Acceptance Criteria

1. The VSCode Extension shall アタッチの構成の属性・最初の構成・構成の雛形に、`host` を出さない。
2. When `launch.json` の `pasta` の構成に `host` が書いてある（値によらない）, the VSCode Extension shall 「許されない」と注意し、その値を使わない（OQ-3）。
3. The VSCode Extension shall アタッチ・シーン再生の「デバッグ開始」・SHIORI リロードの後の再アタッチで、常に `127.0.0.1` の、構成の `port`（既定 9276）へつなぐ。
4. The VSCode Extension shall `port` と `sourcePresentation` の扱いを、今と同じに保つ。

### Requirement 11: 他の spec との約束を保つ

**Objective:** As a pasta の開発者, I want 同じウェーブで並ぶ spec と、互いに壊さない約束が要件に書いてあってほしい, so that 後から入る側が、何を保てばよいかを迷わない。

#### Acceptance Criteria

1. The VSCode Extension shall 入力のたびの診断を、`parse_str` の誤りの形（`SyntaxError` の行・桁・文。行と桁は 1 から数え、桁は 1 文字を 1 と数える。文の末尾に pest の `  = 説明` の行がある）から作る。`ponytail-dsl-parser` がこの形を保つことを前提にする。
2. If `parse_str` が位置の無い誤りを返した, then the VSCode Extension shall 【前提】1 行目の頭に、`pasta_check validate` と同じ文で 1 件だけ出す（今の `parse_str` は文字列に対して位置の無い誤りを返さない）。
3. The change shall `partial.rs` のほかの `pasta_dsl/src/parser/**` を変えない。`pasta_dsl/tests/cue_cmd_parse_test.rs` は `infer_rule_from_line` の 4 つのテストだけを消す（`ponytail-dsl-parser` が同じファイルを触るなら、後から入る側が rebase で合わせる）。
4. The change shall `book/src/debug/*.md` のうち、`host` と「エディタの診断」の節だけを変える。

### Requirement 12: 死んだコードと同語反復を無くす

**Objective:** As a pasta の開発者, I want どこからも使われないファイル・規則・関数・テストが無くなってほしい, so that 読み手が使われない道を追わずに済む。

#### Acceptance Criteria

1. The VSCode Extension shall どこからも参照されない `scripts/build-wasm.bat` を持たない。
2. The VSCode Extension shall 自分で作った値を確かめるだけのテスト（`src/test/e2e.test.ts` の 200〜288 行。起動の状態の写しとファイルの有無）を持たない。
3. The VSCode Extension shall 使われない再輸出（`reloadShiori.ts` の 18〜22 行）・使われない `DiagnosticsManager.clearAll`・テストだけが読む起動の状態の `fallbackMode`（N10）を持たない。
4. The pasta_dsl crate shall 部分解析の中で、各行をファイル全体として解く段（第 3 段）・行の切り出し（`chunk_lines`）・行頭からの規則の推測（`infer_rule_from_line`）と、常に空の誤りの範囲（`PartialParseError.span`）を持たない（N3）。
5. The pasta_lsp test suite shall 何も確かめない、または緩すぎる診断のテスト（`diagnostics_test.rs` の「100 行より前」「トークンか診断のどちらかがある」、`transport.rs` の `test_wasm_analysis_result_from_error`）を、本当の位置と件数を確かめるテストに置き換える（N8）。
6. The VSCode Extension test suite shall 誤った今の色を固めるテスト（`tmGrammar.test.ts` の「既存: エスケープ ＠＠単語・＄＄x の色分けは不変」など）を、要件 4〜6 の色を確かめるテストに置き換える。

### Requirement 13: マニュアル・README・steering を、直した挙動と構成に合わせる

**Objective:** As a ゴーストの作者・pasta の開発者, I want 書いてあることと、拡張とクレートの動きが食い違わないでほしい, so that 例を写しても、注意や誤りが出ない。

#### Acceptance Criteria

1. When 要件 10 が入る, the pasta manual shall `debug/vscode-setup.md`・`debug/source-level.md`・`debug/troubleshooting.md` の構成の例と表から `host` を消し、接続先が `127.0.0.1` に固定であることを書く。
2. When 要件 1〜2 が入る, the pasta manual shall `debug/validate.md` の「エディタの診断」の「構文エラーの重複」に、入力のたびの診断が保存時と同じ位置・同じ文であることを書く。
3. When 要件 5.5 が入る, the pasta manual shall `internals/talk-output.md` の「単位の読み方の写し」の記述（`inline-escape` を `sakura_escape` の写しとする行）を、直した後の規則に合わせる。
4. When 要件 9 が入る, the change shall `crates/pasta_lsp/README.md`・`TEST_COVERAGE.md`（`pasta_lsp` の行）・`.kiro/steering/tech.md`（15・66〜69・111・124 行）・`.kiro/steering/structure.md`（186〜205 行）から、言語サーバと tower-lsp の記述を消し、今の構成を書く。

### Requirement 14: 直していない挙動を保ち、品質の関門を通す

**Objective:** As a pasta の開発者, I want 直しのほかの挙動が変わらず、決まった関門を通ってほしい, so that 整理のついでに別の不具合が入らない。

#### Acceptance Criteria

1. When 挙動を変える直し（要件 1〜6・8・10）を入れる, the change shall 直す前の誤りを再現するテストを先に置き、直したあとにそのテストが通る。
2. The change shall `cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_lsp -p pasta_dsl -p pasta_check`、拡張の `npm test` を通る。
3. The change shall VSIX を作れる（`build-wasm.ps1` で WASM を作り、`vsce package` が通る）。
4. Where `pasta.tmLanguage.json` を変える, the change shall マニュアルのハイライトの自己テスト（`book/tools/highlight/tokenizer-test.mjs`・`highlight-html-test.mjs`）を通す。
5. The VSCode Extension shall ゴースト全体の検査（保存時の診断）の起動の条件・集め方・表示を、要件 2 のほかは今と同じに保つ。
