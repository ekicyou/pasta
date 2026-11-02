# Requirements Document

## Project Description (Input)
ゴーストの作者が辞書に Lua ブロックを書くと、トランスパイラがその中身を書き換えてしまう（2026-10-19 の `/ponytail:ponytail-audit` の指摘より）。ブロックの各行に空白 4 つの字下げが足されるため、長い文字列（`[[ ]]`）の中身に空白が入る。次の空白でない行が `end` のとき、長い文字列の中の空行も消える。加えて、効かない変換の設定（`LineEnding`・`comment_mode`）、`TranspileContext` の死んだ状態（`current_module`・`register_global_scene`・使い手の無くなった `merge_from`）、素通しの入口（`with_defaults`・`transpile_with_sink`）が、変換器を読む人の手間を増やしている。`crates/pasta_lua/src` の `code_gen/`・`normalize.rs`・`config.rs`・`context.rs`・`transpiler.rs` の監査の指摘を残らず直し、Lua ブロックの中身が書いたとおりに出て（長い文字列も空行もそのまま）、効かない設定・死んだ状態・素通しが無くなるようにする。ファイルの属性の欄と関数（`accumulate_file_attr`・`file_attrs`・`merge_attrs`）は、0.5.0 の `scene-attribute-store` が使うので残す。

## Introduction

この spec は、2026-10-19 の監査が `pasta_lua` のトランスパイラ（`code_gen/`・`normalize.rs`・`config.rs`・`context.rs`・`transpiler.rs` と、`lib.rs` のそれらの再輸出）で見つけた指摘 1〜4 と、ギャップ分析（2026-11-02 の監査のやり直し）で同じ領域に増えた指摘を、残らず直す。直す中身は 2 種類ある。

- **挙動を変える直し**：Lua ブロックの中身の書き換え（字下げ・空行の削除）をやめる（要件 1）。アクター辞書の単語定義の行を、デバッグのソースマップに載せる（要件 5。ギャップ分析で増えた）。
- **挙動を変えない撤去と整理**：効かない設定（要件 3）、死んだ状態と素通し（要件 4）、重なった規則と古い説明（要件 6）。

要件は「その領域の監査の指摘が、残らず直っている」ことと、挙動を変える直しの決めごとである（2026-10-19 開発者の決定）。どう直すか（関数・ファイルの形）は設計で決める。

> brief.md だけでは決めきれなかった論点は、仮定を置いて書き、research.md の「要件ディスカッションへの持ち越し」に並べた（要件 2.1・2.2・5・6 と、`LineShift` の扱い）。持ち越し 2〜5 は、brief の決めごと（監査の指摘を残らず直す・ギャップ分析で増えた同じ領域の指摘は In・pub API を消してよい・ほかの持ち場に触らない）で閉じた。持ち越し 1（要件 2.1・2.2 のバイト単位の一致）は、開発者が推奨案のとおり承認した。保てないと設計で分かったときは、research.md の案 B へ退く（2026-11-02 の要件ディスカッション）。

## Boundary Context

- **In scope**
  - 監査の 1〜4 番（Lua ブロックの中身の書き換え・死んだ変換の設定・`TranspileContext` の死んだ状態・素通し）。
  - 申し送り（2026-11-01、`ponytail-lua-loader` の設計より）の `TranspileContext::merge_from`。使い手が 0 になったので、死んだ状態に足して消す。`context.rs` 117 行の古い説明もいっしょに消える。
  - ギャップ分析で増えた、同じ領域の指摘（要件 4.5・4.6、要件 5、要件 6）と、設計で `ponytail-dsl-parser` の申し送りから増えた指摘（要件 9）。
  - 直しに合わせたスナップショット（`tests/transpiler`・`src/code_gen/*_tests.rs`）と見本（`tests/fixtures/sample.generated.lua` ほか）の作り直し、マニュアル `book/src/internals/transpiler.md` の直し。
- **Out of scope**
  - `TranspileContext` のファイルの属性の欄と関数（`accumulate_file_attr`・`file_attrs`・`merge_attrs`）と、`generate_global_scene` の `_file_attrs` 引数（2026-10-19 開発者の決定。0.5.0 の `scene-attribute-store` が、ファイルレベル属性の継承に使う）。
  - `loader/`・`runtime/`（`ponytail-lua-loader` で完了）。本番のローダとランタイムが `TranspileContext` をファイルごとに捨てていること自体は変えない。
  - `debug/`。`ponytail-lua-debug` は 2026-11-02 に閉じ、残る部品（`source_map/` など）への指摘は 0.5.1 の DAP 撤去 spec が受け継ぐ。デバッグのソースマップが読む行の対応の形（`LineShift` を含む）は変えない（research.md の持ち越し 3）。例外は要件 8.5 のテスト 1 本の入力だけ。
  - `pasta_core` の `SceneRegistry::merge_from`・`WordDefRegistry::merge_from`（`ponytail-core-registry` の持ち場）。消す順は要件 8.4 のとおり。
  - `string_literalizer.rs`（文字列のリテラル化の規則）。要件 5.3 で、エラーの位置を保つ呼び方だけを揃える。
  - Lua ブロックの内容の検査（今と同じく、トランスパイラは中身を検査しない。誤りはロード時の Lua エラーになる）。
  - README と `.kiro/steering/tech.md` の pub API の断り書き（`ponytail-lua-loader` が書いた。この spec は書かない）。
- **Adjacent expectations**
  - `pasta_dsl` が渡す AST と span の値は変わらない（`ponytail-dsl-parser` は span の値を変えない）。
  - ローダのキャッシュは、キャッシュに記録した `pasta_lua` の版が変わると丸ごと作り直される（今の仕組み）。直った版を出すと版が上がるので、字下げの入った古い変換結果は使われない。この spec はキャッシュの仕組みを変えない。
  - デバッグのソースマップ（`debug/source_map`）は、`.pasta` の行と生成した `.lua` の行の対応を今と同じ形で受け取る。この spec は対応の中身を、要件 2.3〜2.4・要件 5 のとおりに保つか増やすだけで、形を変えない。
  - 同じウェーブの約束：`code_gen/element_gen.rs` は、このウェーブではこの spec だけが触る。`lib.rs` の再輸出の行・マニュアルの同じ頁の別の節は、後から入る側が合わせる。

## Requirements

### Requirement 1: Lua ブロックの中身を書いたとおりに出す

**Objective:** As a ゴーストの作者, I want 辞書に書いた Lua ブロックが、1 文字も変わらずに生成コードに出てほしい, so that 長い文字列で書いたトークや設定の値が、実行時に黙って別の文字列にならない。

#### Acceptance Criteria

1. When 作者がグローバルシーン・ローカルシーン・アクター辞書のいずれかに Lua ブロックを書いた, the Pasta Transpiler shall ブロックの内容の各行を、行頭と行末の空白を含めて書いたとおりの文字列で生成コードに出す（字下げを足さない）。
2. When Lua ブロックが長い文字列（`[[ … ]]`・`[==[ … ]==]` など）を含む, the Pasta Transpiler shall 生成コードを実行したときのその文字列の値を、作者が書いた中身と同じにする。例：```` ```lua ```` の中の 5 行 `local MSG = [[`・`こんにちは`・（空行）・`end`・`]]` は、実行すると `MSG` が `"こんにちは\n\nend\n"` になる（今は `"    こんにちは\n    end\n    "`）。
3. When Lua ブロックの中に空白だけの行がある, the Pasta Transpiler shall その行を、次の空白でない行が `end` であるかどうかによらず、消さずに出す。
4. The Pasta Transpiler shall 1 つの Lua ブロックの内容の行を、行の数と順番を変えずに、続いた行として生成コードに出す（内容の n 行目は、出したブロックの n 行目になる）。
5. Where アクター辞書の Lua ブロックの言語識別子が `lua` でない, the Pasta Transpiler shall 今と同じくそのブロックを出さない（グローバルシーンの Lua ブロックは、今と同じく言語識別子によらず出す）。
6. The Pasta Transpiler shall Lua ブロックを出す位置（アクター辞書の単語定義の後・グローバルシーンのすべての関数定義の後）を、今と同じにする。

### Requirement 2: ブロックの外の生成コードと、行の対応を保つ

**Objective:** As a ゴーストの作者とデバッガを使う開発者, I want Lua ブロックの直しで、ほかの生成コードとブレークポイントの位置が変わらないでほしい, so that 直した版に上げても、辞書の動きとデバッグのしかたが今のまま使える。

#### Acceptance Criteria

1. The Pasta Transpiler shall Lua ブロックを含まない辞書の変換結果を、今とバイト単位で同じにする。項目が 1 つも無いシーン・入れ子のブロックの閉じ（`end` が続く所）・アクター辞書だけのファイルを含む（例：見本 `tests/fixtures/sample.pasta` の変換結果 `sample.generated.lua` は変わらない）。
2. When Lua ブロックを含む辞書を変換した, the Pasta Transpiler shall ブロックの内容の行を除いた生成コードの行の並びを、今と同じにする（変わるのは、ブロックの内容の行の字下げと、要件 1.3 で消さなくなった空行だけである）。
3. The Pasta Transpiler shall Lua ブロックの内容の各行を、今と同じく、その行を書いた `.pasta` の行に対応づける。シーン関数の頭の `init_scene` に渡す行の対応と、デバッグのソースマップの両方で対応づける。
4. When 要件 1.3 で Lua ブロックの中の空行を出すようになった, the Pasta Transpiler shall その行より後ろのすべての行の対応を、最終的に出した `.lua` の行番号で記録する（行の対応と出力の行がずれない）。
5. The Pasta Transpiler shall 生成コードの改行を、今と同じく LF だけにし、末尾を改行 1 つで終える。
6. If Lua ブロックの内容が Lua として誤っている, then the Pasta Transpiler shall 今と同じく変換を通す（中身を検査しない。誤りは、今と同じくロード時の Lua エラーになる）。

### Requirement 3: 効かない変換の設定を無くす

**Objective:** As a pasta の開発者, I want 出力に効かない設定が、変換の設定と生成器から無くなってほしい, so that 「改行の種類やコメントの有無を変えられる」と読み違えて、効かない設定を調べる手間がかからない。

#### Acceptance Criteria

1. The Pasta Transpiler shall 変換の設定として、出力に効く項目（グローバルシーン名の別名表）だけを持つ。改行の種類の設定（`LineEnding` と `TranspilerConfig` の `line_ending`・`with_line_ending`）と、コメントの有無の設定（`comment_mode`・`without_comments`）を持たない。
2. The Lua Code Generator shall 改行の種類の設定（`line_ending` の欄と `with_line_ending`）を持たず、作り方を 1 つにする。
3. The pasta_lua crate shall `LineEnding` を公開しない（`lib.rs` の再輸出から外す）。
4. When 効かない設定を消した, the Pasta Transpiler shall 出力のバイト列を変えない（Windows を含むどの環境でも、生成コードは今と同じく LF だけである）。

### Requirement 4: 死んだ状態と素通しを無くす

**Objective:** As a pasta の開発者, I want 本番で使われない状態・関数・入口が無くなってほしい, so that 変換の道が 1 本に見え、どれが本物の入口かを探さずに済む。

#### Acceptance Criteria

1. The Transpile Context shall 本番で読まれない状態と関数を持たない：`current_module` の欄とその読み書き、`register_global_scene`（宣言名を受け取る `register_global_scene_named` に委譲するだけの方）、`merge_from`（申し送りのとおり、使い手が 0）。
2. The Pasta Transpiler shall グローバルシーンの生成で使われない文脈の引数（`generate_global_scene` の `_context`）を持たない。
3. The Lua Transpiler shall 同じ働きの入口を重ねて持たない：`with_defaults`（`Default` と同じ）、`transpile_with_sink`（`transpile_with_source_map` の素通し）、テストだけが使う `config()`。
4. The normalize module shall テストだけが使う包み（`normalize_output`。`normalize_output_with_shift` の文字列だけを返す）を持たない。
5. The Pasta Transpiler shall 変換の設定の作り方を 1 つにする（`TranspilerConfig::new` は `Default` と同じなので持たない）。
6. The Transpile Context shall ファイルの属性の欄と関数（`accumulate_file_attr`・`file_attrs`・`merge_attrs`）と、`generate_global_scene` の `_file_attrs` 引数を、今のまま持つ（Out of scope のとおり）。
7. When 死んだ状態と素通しを消した, the Pasta Transpiler shall 出力のバイト列と行の対応を変えない。消した入口を使っていたテストは、残る入口（`transpile`・`transpile_with_source_map`）で同じことを確かめる。

### Requirement 5: アクター辞書の単語定義の行も対応づける

**Objective:** As a デバッガを使う開発者, I want アクター辞書の単語定義の行にも、グローバルやシーンの単語定義と同じく生成した行の対応があってほしい, so that 単語定義の種類によってブレークポイントが置けたり置けなかったりしない。

#### Acceptance Criteria

1. When アクター辞書の単語定義（例：`％さくら` の配下の `　＠通常：\s[0]`）を変換した, the Pasta Transpiler shall 生成した単語の登録の行（`ACTOR:create_word(…)` の行）を、その単語定義の `.pasta` の行に、デバッグのソースマップで対応づける（グローバルとシーンの単語定義と同じ規則）。
2. The Pasta Transpiler shall アクター辞書の単語定義の生成コードのバイト列を、今と同じにする。
3. If アクター辞書・グローバル・シーンの単語の値を Lua の文字列にできない（ロングブラケットの `=` を 10 個まで試しても閉じ括弧と衝突する）, then the Pasta Transpiler shall 3 種類のどの単語定義でも同じく、その単語定義の位置つきの変換エラーにする。

### Requirement 6: 重なった規則と古い説明を無くす

**Objective:** As a pasta の開発者, I want 同じ規則が 1 か所にだけあり、説明が今の道と合っていてほしい, so that 規則を変えるときに直し漏れが出ず、説明に従って調べて迷わない。

#### Acceptance Criteria

1. The Pasta Transpiler shall 変数の経路（`var.名前`・`save.名前`）を決める規則を 1 か所に持ち、変数代入と変数の参照の両方で同じ規則を使う（出力は今と同じ）。
2. The Pasta Transpiler shall アクター辞書の単語定義と、グローバル・シーンの単語定義を、同じ 1 つの規則で生成する（要件 5 の対応づけとエラーの位置も、この規則で揃う）。
3. The Pasta Transpiler shall 位置を持つ構文要素の行の対応づけを、どの要素でも同じ規則で行う（出した行を、その要素の始まりの `.pasta` の位置に対応づける。何も出さない要素は対応づけない）。
4. The pasta_lua source comments shall トランスパイラの道を今のとおりに説明する。とくに「本番ではシンクを付けないので記録は働かない」という説明を残さない（今は、行の対応を記録する仕組みが常に付き、呼び手のシンクだけが任意である）。

### Requirement 7: 振る舞いを固めるテストと品質の関門

**Objective:** As a pasta の開発者, I want 直す前に今の振る舞いを固め、直した後もテストが直した振る舞いを守ってほしい, so that 直しが生成コードとブレークポイントの位置を壊していないことを、いつでも確かめられる。

#### Acceptance Criteria

1. The pasta_lua tests shall シーンとアクター辞書の Lua ブロックに、長い文字列と `end` の前の空行を置いた辞書を変換し、ブロックの内容が書いたとおりに出ることを確かめる。
2. The pasta_lua tests shall 要件 1.2 の例の生成コードを Lua として実行し、長い文字列の値が作者の書いた中身と同じになることを確かめる。
3. The pasta_lua tests shall 要件 1.1 の辞書で、ブロックの各行の行の対応（`init_scene` に渡す対応と、デバッグのソースマップの両方）が、書いた `.pasta` の行を指すことを確かめる。
4. The pasta_lua tests shall 要件 5.1 のアクター辞書の単語定義の行の対応を確かめる。
5. The pasta_lua tests shall 直す前に、Lua ブロックを含まない辞書の変換結果を固め（今の出力を期待値にする）、直した後も同じであることを確かめる（要件 2.1）。
6. When Lua ブロックを含むスナップショット（`tests/transpiler`・`src/code_gen/*_tests.rs`）を作り直した, the pasta_lua tests shall 作り直した差分が、要件 2.2 の 2 種類（ブロックの内容の行の字下げと、消さなくなった空行）だけであることを確かめてから、新しいスナップショットを正にする。
7. The 変更 shall 1 つ直すごとにテストで確かめ、1 コミットにする。品質の関門として、`cargo fmt --check`・clippy（警告 0）・`cargo test -p pasta_lua` が通る。`pasta_lua` のテストが書き換える `sample.generated.lua` の改行だけの差分は戻し、コミットに混ぜない。

### Requirement 8: マニュアルとほかの spec との境目

**Objective:** As a マニュアルの読み手と、同じウェーブの spec の開発者, I want マニュアルが直した後の振る舞いを書き、ほかの spec の持ち場を壊さないでほしい, so that マニュアルのとおりに読んで迷わず、同じウェーブの spec が互いに合わせ直さずに入れる。

#### Acceptance Criteria

1. The マニュアル shall `book/src/internals/transpiler.md` を、直した後の振る舞いに合わせる：設定の表と説明（99〜102 行）から消した設定を除き、出力の正規化の節（「ブロックの閉じ直前の空行を除く」手順）・入口の表（`transpile_with_sink` の行）・記録する構文要素の一覧（アクター辞書の単語定義を足す）・不変条件（`normalize_output` の行）を直す。
2. The マニュアル shall `book/src/grammar/block-structure.md` の「Lua ブロックの内容は検証されず、そのまま生成される Lua コードに出力される」を、直した後の事実として保つ（書き換えない）。
3. The 変更 shall README と `.kiro/steering/tech.md` に pub API の断り書きを書き足さない（`ponytail-lua-loader` が書いた）。
4. If この spec が `ponytail-core-registry` より後に main に入る, then the 変更 shall 使い手の無くなった `pasta_core` の `SceneRegistry::merge_from`・`WordDefRegistry::merge_from` も消す。先に入る場合は、`pasta_core` のファイルに触らない（あちらが消す）。
5. The 変更 shall `debug/`・`loader/`・`runtime/`・`pasta_core`（要件 8.4 の場合を除く）のソースに触らない。ただし `debug/source_map_builder_tests.rs` の `map_builder_sink_finish_rebases_to_final_lua_lines` の入力だけは、この spec が直す（要件 1.3 で消す「`end` の前の空行の削除」を固めているため。`debug/` の持ち場の spec は 2026-11-02 に閉じ、このウェーブに居ない）。

### Requirement 9: 同じ領域の数のあふれと飽和を直す

**Objective:** As a ゴーストの作者, I want シーンの引数の番号と `!select` の秒数が、書いたとおりの値で生成コードに出てほしい, so that 大きな番号や大きな秒数を書いたときに、黙って別の値になったり変換が落ちたりしない。

> 2026-11-02 の設計で `ponytail-dsl-parser` の申し送りから足した（同じ領域の指摘。`ponytail-dsl-parser` は `＄２５６` 以上を構文の誤りにし、i64 に入らない整数を浮動小数にすると決めた）。

#### Acceptance Criteria

1. When 作者がシーンの引数 `＄０`〜`＄２５５` を書いた, the Pasta Transpiler shall `args[1]`〜`args[256]` を出す（今は `＄２５５` で、debug ビルドは変換が panic し、release ビルドは `args[0]` を出す）。
2. When `!select` の秒数が小数部の無い浮動小数である, the Pasta Transpiler shall その値を i64 の範囲に詰めずに出す（例：`1e19` は `10000000000000000000`。今は `9223372036854775807`）。
3. The Pasta Transpiler shall 9.2 の直しで、今の `!select` の出力（`10.0` は `10`、`2.5` は `2.5`、`-0.0` は `0`）を変えない。
