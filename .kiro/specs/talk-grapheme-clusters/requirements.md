# Requirements Document

## Project Description (Input)
ゴーストの作者が、組み合わせの絵文字（ZWJ でつないだもの・異体字セレクタ付きのもの・肌色の修飾子付きのもの）や結合文字を台詞に書くと、表示が崩れる（2026-10-18、ghost_dev の「ぱすた開発」セッション（窓際のぱすたさん）が pasta 0.4.1 と SSP 2.9.18 の実機で報告）。台詞「りょ🙆‍♀️✨」が `り\_w[40]ょ\_w[40]🙆\_w[40]‍\_w[40]♀\_w[40]️\_w[40]✨` になり、縦書きのバルーンでは「🙆」と「♀」が別々に描かれる。`[talk]` の `chars_period` などに組み合わせの絵文字を書いても 1 文字として扱われない。

今の pasta は、台詞の文字を、自動ウェイト・文字の種類の判定・budoux の改行のどれでもコードポイント単位で扱っている。これを、拡張書記素クラスタ（UAX #29）を 1 文字として扱うように改め、クラスタの途中に自動ウェイトも改行も入らないようにする。1 つのコードポイントだけでできた文字の出力は変えない（詳しくは `brief.md`）。

## Introduction

この仕様は、`@pasta_sakura_script` の台詞の変換（`talk_to_script`・`break_lines`）が「1 文字」と数える単位を、コードポイントから**拡張書記素クラスタ**（以下「クラスタ」。UAX #29 の extended grapheme cluster）へ改める。対象の読者はゴーストの作者で、絵文字や結合文字を台詞にそのまま書いたときに、見た目どおりの 1 文字として自動ウェイト・文字の種類・改行が働くことを目指す。日本語の台詞の大部分（1 つのコードポイントでできた文字だけの台詞）の出力は 1 文字も変えない。

用語:
- **クラスタ**: 拡張書記素クラスタ（UAX #29）。例: `🙆‍♀️`（U+1F646 U+200D U+2640 U+FE0F）・`✌️`（U+270C U+FE0F）・`👍🏽`（肌色の修飾子付き）・`🇯🇵`（地域指示子の対）・`é`（`e` ＋ U+0301）。
- **1 コードポイントのクラスタ**: 1 つのコードポイントだけでできたクラスタ（ひらがな・漢字・ASCII など、今の日本語の台詞の大部分）。
- **単位**: さくらスクリプトのタグ・エスケープ（`\\`・`\%`）・囲み（`\_?…\_?`・`\_!…\_!`）。今の読み取りの規則（マニュアルの「さくらスクリプトの後処理」）で決まるもの。
- **単独の `\`**: 単位にならない `\`（`\あ` の `\`・テキスト末尾の `\`）。

## Boundary Context

- **In scope**:
  - `talk_to_script` の自動ウェイトと文字の種類の判定、`talk_to_script` と `break_lines` の budoux の改行と行の幅を、クラスタ単位にすること。
  - `pasta.toml` の `[talk]` の `chars_*`（`chars_period`・`chars_comma`・`chars_strong`・`chars_leader`・`chars_line_start_prohibited`・`chars_line_end_prohibited`）に書いた複数のコードポイントの項目を、1 文字として一致させること。
  - 退行テストと、1 コードポイントのクラスタだけの台詞の出力が変わらないことの特性化テスト。
  - マニュアル（「1 文字」の定義）とスキル references の再生成。
- **Out of scope**:
  - 表示の幅を、SSP のバルーンの描画（フォントしだい）と完全に一致させること。
  - 単位の直後に結合文字が来る場合（`\n` ＋ U+0301 など）を前の字とつなぐこと。単位の読み取りが先に止まり、結合文字は単独のクラスタになる。これを正しい動きとする。
  - Lua の側の処理（台詞を 1 文字ずつ処理する所が無い）。`talk_to_script`・`break_lines` の引数と戻り値の形は変えない。
  - budoux の禁則（バックログの「budoux の自動改行の禁則」）。
  - 単語・シーン名の正規化（検索のキー）。
  - 別々の `talk_to_script` の呼び出し（表情の切り替えなどで区切られた台詞）にまたがるクラスタをつなぐこと。1 回の呼び出しに渡したテキストの中だけでクラスタを決める。
- **Adjacent expectations**:
  - 単位の読み取りの規則（`paragraph-break-tag-only-talk`・`sakura-script-newline` で決めたもの）と、Lua の `tag_at`・DSL の文法の写しとの一致は変えない。
  - 列の状態の持ち越し・行を動かす単位での数え直し・単独の `\` の直後の守り（`vertical-balloon-lines` の決定 9）は今の約束のまま保つ。
  - 依存を足す場合は、リリース CI の関門（`cargo-deny`）と、x86・x64（MSVC・静的 CRT）のビルドを通す。

## Requirements

### Requirement 1: クラスタを 1 文字として切り出す
**Objective:** As a ゴーストの作者, I want 組み合わせの絵文字や結合文字を、見た目どおりの 1 文字として扱ってほしい, so that 台詞に書いた絵文字が途中で割れずに表示される

#### Acceptance Criteria
1. When `talk_to_script` または `break_lines` に台詞のテキストが渡されたとき, the pasta_sakura_script module shall 単位でない部分を、クラスタ（拡張書記素クラスタ）ごとに 1 文字として扱う。
2. The pasta_sakura_script module shall 単位（タグ・エスケープ・囲み）の読み取りを今と同じ規則で行い、同じ入力から同じ単位を読み取る。
3. If クラスタの規則のうえでは前の文字につながる位置に `\` があるとき（例: U+0600 などの Prepend の文字の直後の `\h`）, the pasta_sakura_script module shall その `\` から始まる単位を今と同じに読み取り、前の文字のクラスタにその `\` を含めない。
4. When 単位の直後に結合文字・ZWJ・異体字セレクタが来たとき（例: `\n` ＋ U+0301）, the pasta_sakura_script module shall その結合文字などを、単位の後ろの単独のクラスタとして扱う。
5. When 台詞のテキストに ZWJ の並び（`🙆‍♀️`）・異体字セレクタ付きの文字（`✌️`）・肌色の修飾子付きの絵文字（`👍🏽`）・国旗（`🇯🇵`）・結合文字付きの文字（`é`）が含まれるとき, the pasta_sakura_script module shall それぞれを 1 文字として扱う。

### Requirement 2: 自動ウェイトをクラスタの途中に入れない
**Objective:** As a ゴーストの作者, I want 自動ウェイトが絵文字の途中に入らないでほしい, so that `🙆‍♀️` が `🙆` と `♀` に割れて描かれない

#### Acceptance Criteria
1. The pasta_sakura_script module shall `\_w[…]` の自動ウェイトを、1 つのクラスタの途中（クラスタを作るコードポイントの間）に入れない。
2. When 一般の文字またはリーダーに分類されたクラスタを出力するとき, the pasta_sakura_script module shall 1 つのクラスタにつき 1 つのウェイトを、そのクラスタの後ろに入れる。
3. When 台詞「りょ🙆‍♀️✨」を、既定の `chars_*` と `script_wait_normal = 90` で変換したとき, the pasta_sakura_script module shall `り\_w[40]ょ\_w[40]🙆‍♀️\_w[40]✨\_w[40]` を返す。
4. When 台詞「✌️」を一般の文字のウェイトが正の値で変換したとき, the pasta_sakura_script module shall U+270C と U+FE0F の間にウェイトを入れず、`✌️` の後ろに 1 つだけウェイトを入れる。
5. When 句点・読点・強調・行頭禁則に分類されたクラスタが続いたとき, the pasta_sakura_script module shall 今と同じく、並びの最後のクラスタの後ろに、並びの中の最大の値から 50 を引いたウェイトを 1 つだけ入れる。
6. When 単独の `\` の後ろに文字が続くとき, the pasta_sakura_script module shall 今と同じく、その `\` の直後にウェイトを入れず、続くクラスタの後ろに入れる（`\` の直後に `\_w` を置いて `\\` に読まれる出力を作らない）。

### Requirement 3: 文字の種類の判定と `chars_*` をクラスタで行う
**Objective:** As a ゴーストの作者, I want `chars_period` などに組み合わせの絵文字を書いたら、その絵文字が 1 文字として一致してほしい, so that 絵文字の後ろに句点と同じ間を出せる

#### Acceptance Criteria
1. The pasta_sakura_script module shall 文字の種類（句点・読点・強調・リーダー・行頭禁則・行末禁則・一般）の判定を、クラスタ 1 つを単位として行い、判定の優先順（句点・読点・強調・リーダー・行頭禁則・行末禁則・一般）は今のまま保つ。
2. When `[talk]` の `chars_*` に複数のコードポイントでできた文字（例: `chars_period = "。🙆‍♀️"`）が書かれていて、台詞にその文字があるとき, the pasta_sakura_script module shall 台詞のそのクラスタ全体を、その `chars_*` の種類として判定する。
3. If 台詞のクラスタが 2 つ目以降のコードポイントに `chars_*` の文字を含むとき（例: `chars_period` に U+FE0F や `♀` があり、台詞に `🙆‍♀️` があるとき）, the pasta_sakura_script module shall その部品のコードポイントでクラスタの種類を決めない。
4. When `chars_*` を書いていない（既定の値）とき, the pasta_sakura_script module shall 既定の値に含まれる各文字（`ｰ`・`ﾞ`・`ﾟ` を含む）を、今と同じ種類に判定する。
5. When `chars_*` に 1 コードポイントの文字だけを並べた設定が書かれているとき, the pasta_sakura_script module shall 並んだ各文字を、今と同じ種類に判定する（並びの中で隣どうしがクラスタの規則でつながる場合も含む）。 _（前提: 既定の `chars_line_start_prohibited` の末尾 `ｰﾞﾟ` はクラスタの規則で 1 つにつながる。議題 Q1 で確認する）_
6. When 台詞のクラスタが `chars_*` のどの項目とも全体では一致しないとき, the pasta_sakura_script module shall そのクラスタの先頭のコードポイント（基底の文字）で種類を判定する（例: `chars_strong` に `❗` があるとき、`❗️`（U+2757 U+FE0F）も強調として判定する）。 _（前提: 議題 Q2 で確認する）_

### Requirement 4: budoux の改行と行の幅をクラスタで数える
**Objective:** As a ゴーストの作者, I want budoux の自動改行が絵文字の途中で行を割らず、絵文字を見た目どおりの幅で数えてほしい, so that 縦書きのバルーンでも絵文字が 1 文字として描かれ、改行の位置がずれない

#### Acceptance Criteria
1. The pasta_sakura_script module shall `talk_to_script` と `break_lines` のどちらでも、budoux が入れる改行をクラスタの途中に入れない。
2. If budoux の語の区切りがクラスタの途中に来たとき, the pasta_sakura_script module shall その位置では改行を入れない（クラスタの途中の区切りを捨てる）。
3. The pasta_sakura_script module shall 行の字数を、1 つのクラスタを 1 つの表示の単位として数える。
4. When 行の字数を数えるとき, the pasta_sakura_script module shall 東アジアの文字幅で、`🙆‍♀️`・`✌️`・`👍🏽`・`🇯🇵` をそれぞれ 2 と数える。
5. The pasta_sakura_script module shall 1 コードポイントのクラスタの幅を、今と同じ値で数える（制御文字は今と同じく 0、幅が曖昧な文字は今と同じく 2）。
6. The pasta_sakura_script module shall 列の状態（`line`・`width`・`jump`）の読み書き、行を動かす単位（`\n`・`\n[…]`・引数の無い `\c`・`\_l[…]`）での数え直し、呼び出しの間の持ち越しを、今と同じ約束で行う。
7. When 単独の `\` に続く語で行の上限を超えるとき, the pasta_sakura_script module shall 今と同じく、`\` と続く語を 1 つの語として `\` の前で割り、`\` の直後に改行を入れない。
8. When `budoux` を持つアクター（縦書きのバルーンのアクターを含む）が `🙆‍♀️` を含む台詞を話したとき, the pasta_sakura_script module shall 出力の中で `🙆‍♀️` の 4 つのコードポイントを、間に何も挟まずに続けて出す。

### Requirement 5: 1 コードポイントの文字だけの台詞の出力を変えない
**Objective:** As a 既存のゴーストの作者, I want 絵文字を使わない台詞の出力が今と 1 文字も変わらないでほしい, so that 版を上げてもゴーストの見た目とテンポが変わらない

#### Acceptance Criteria
1. While 台詞のテキストの単位でない部分が 1 コードポイントのクラスタだけでできている間, the pasta_sakura_script module shall `talk_to_script` と `break_lines` の出力と列の状態を、今の版と 1 文字も違わないものにする。
2. The pasta_lua test suite shall 既存の `sakura_script` のテスト（単体・結合）の期待値を書き換えずに通る。
3. When 半角カナと半角の濁点・半濁点（`ｶﾞ`・`ﾊﾟ` など、クラスタの規則で 1 つのクラスタになる並び）を含む台詞を変換したとき, the pasta_sakura_script module shall その並びを 1 つのクラスタとして扱う（ウェイトと改行をその間に入れない）。 _（前提: 今の出力から変わる。議題 Q3 で確認する）_
4. The pasta_sakura_script module shall `talk_to_script`・`break_lines` の Lua からの呼び方（引数と戻り値の形）を変えない。

### Requirement 6: 退行テストで約束を確かめる
**Objective:** As a pasta の開発者, I want 報告の例と代表的なクラスタの例をテストで固定したい, so that 後の変更でクラスタが再び割れたら気づける

#### Acceptance Criteria
1. The pasta_lua test suite shall 報告の 2 例（「りょ🙆‍♀️✨」・「✌️」）の自動ウェイトの出力を確かめる。
2. The pasta_lua test suite shall `chars_period` に組み合わせの絵文字を書いた設定で、その絵文字の後ろに句点のウェイトが入ることを確かめる。
3. The pasta_lua test suite shall 肌色の修飾子付きの絵文字・国旗・結合文字付きの文字のそれぞれで、クラスタの途中にウェイトも改行も入らないことを確かめる。
4. The pasta_lua test suite shall budoux の改行（`talk_to_script` と `break_lines`）で、クラスタの途中に改行が入らないことと、クラスタの幅（`🙆‍♀️` が 2）を確かめる。
5. The pasta_lua test suite shall 1 コードポイントのクラスタだけの台詞の出力が変わらないことを、既定の `chars_*` の全文字を含む特性化テストで確かめる。
6. The pasta_lua test suite shall Prepend の文字の直後の単位と、単位の直後の結合文字の扱い（要件 1.3・1.4）を確かめる。

### Requirement 7: マニュアルとスキルで「1 文字」を定義する
**Objective:** As a ゴーストの作者, I want マニュアルが「1 文字」が何を指すかを書いていてほしい, so that 絵文字を台詞や `chars_*` に書いたときの動きを予想できる

#### Acceptance Criteria
1. The pasta user manual shall 自動ウェイト・文字の種類・budoux の幅で使う「1 文字」を、拡張書記素クラスタ（見た目の 1 文字。組み合わせの絵文字・結合文字付きの文字を含む）と定義する。
2. The pasta user manual shall `@pasta_sakura_script` のページ・`pasta.toml` のページ（`[talk]` の `chars_*` と `budoux` の幅）・台詞の出力の内部のページで、「1 文字ずつ」「1 文字ごと」「字数」の記述をこの定義に合わせる。
3. The pasta user manual shall `chars_*` に組み合わせの絵文字を書けることと、それが 1 文字として一致することを書く。
4. The pasta user manual shall アクション行のページの「`\\` の 2 文字の間には何も入らない」という約束と同じく、1 つのクラスタの途中にもウェイトや改行が入らないことを書く。
5. When マニュアルを直したとき, the skill references（`pasta-ghost-authoring`・`pasta-lua-coding`） shall マニュアルから再生成され、鮮度の照合（`gen-skill-refs.mjs --check`）を通る。

### Requirement 8: 配布と依存の条件を保つ
**Objective:** As a pasta の開発者, I want クラスタ対応を入れても配布の条件が変わらないでほしい, so that リリース CI と pasta.dll の配布がそのまま通る

#### Acceptance Criteria
1. The pasta.dll shall x86 と x64（MSVC・静的 CRT）でビルドでき、クラスタ対応の動きが両方で同じになる。
2. If クラスタ対応のために第三者のクレートを足すとき, the release CI shall ライセンスとサプライチェーンの関門（`cargo-deny`）を通り、技術の steering の依存の表にそのクレートが載っている。

## 前提と未決の議題（要件ディスカッションで確かめる）

この草案は `brief.md` で決められない点を、次の前提で書いた。要件ディスカッションで確かめる。

- **Q1（要件 3.4・3.5）**: `chars_*` の文字列をクラスタに分けると、既定の `chars_line_start_prohibited` の末尾 `ｰﾞﾟ`（U+FF70 U+FF9E U+FF9F）が 1 つのクラスタになり、`ｰ`・`ﾞ`・`ﾟ` が単独では一致しなくなる（2026-10-23 に unicode-segmentation 1.13.3 で確かめた）。前提: 既定の値と、1 コードポイントの文字を並べた設定は、今と同じに各文字を判定する。組み合わせの絵文字を書いた設定で、その部品（`♀` など）が台詞に単独で現れたときに一致してよいかは、ここで決める。
- **Q2（要件 3.6）**: 台詞のクラスタが `chars_*` の項目と全体で一致しないとき、先頭のコードポイント（基底の文字）で判定する。`❗` と `❗️`、`。` ＋結合文字、`ｰﾞ` などを今と近い種類に保つための前提。
- **Q3（要件 5.3）**: 半角カナ ＋ 半角の濁点・半濁点（`ｶﾞ`）はクラスタの規則で 1 つのクラスタになる。今は `ｶ\_w[…]ﾞ` のように間にウェイトが入るが、直した後は `ｶﾞ\_w[…]` になる。1 コードポイントの文字だけの台詞ではないので、出力が変わることを許す前提にした。
- **Q4**: `pasta_lua` クレートの公開 Rust API（`CharSets` の `HashSet<char>` のフィールドと `classify(char)`）の型を変えるかは設計で決める。Lua からの呼び方は変えない（要件 5.4）。crates.io の利用者から見た破壊的な変更を 0.4.x で許すかは、ここで確かめる。
- **Q5**: 入れる版。`brief.md` の推奨は 0.4.2（致命的でない不具合）。
