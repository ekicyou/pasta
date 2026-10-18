# Brief: talk-grapheme-clusters

> **ステータス**: 未着手（2026-10-18 の `/kiro-discovery` で起票）。版は開発者が決める（推奨は 0.4.2。致命的でない不具合）。着手するときは `/kiro-start talk-grapheme-clusters` で開始する。

## Problem

ゴーストの作者が、組み合わせの絵文字（ZWJ でつないだもの・異体字セレクタ付きのもの・肌色の修飾子付きのもの）や結合文字を台詞に書くと、表示が崩れる（2026-10-18、ghost_dev の「ぱすた開発」セッション（窓際のぱすたさん）が pasta 0.4.1 と SSP 2.9.18 の実機で報告）。

- 台詞「りょ🙆‍♀️✨」の出力が `り\_w[40]ょ\_w[40]🙆\_w[40]‍\_w[40]♀\_w[40]️\_w[40]✨` になる。ZWJ（U+200D）と VS16（U+FE0F）の前後にも自動ウェイトが入る。縦書きのバルーンでは、「🙆」と「♀」が別々に描かれた。
- 「✌️」（U+270C U+FE0F）も `✌\_w[40]️` となり、VS16 が切り離される。
- `[talk]` の `chars_period` などに組み合わせの絵文字を書いても、1 文字として扱われない。
- ゴースト側の回避: 辞書では 1 つのコードポイントの絵文字だけを使い、`chars_period` に絵文字を足して語尾の間を出している（並んだ句点扱いの文字の最後にまとめて間が入る動きは、期待どおりだった）。

## Current State

台詞の文字を、どこでもコードポイント（Rust の `char`）単位で扱っている。書記素クラスタを扱う処理は、ワークスペースのどこにも無い（2026-10-18 に確かめた）。

- 流れ（`crates/pasta_lua/src/sakura_script/mod.rs`）: 切り出し（`tokenizer.tokenize`）→ 自動ウェイト（`insert_waits`）→ budoux の改行（`line_breaker::break_lines_impl`）。Lua の `break_lines` も同じ `break_lines_impl` を使う。
- **根の原因は切り出し**（`tokenizer.rs`）。タグでない所では `remaining.chars().next()` で 1 コードポイントずつトークンにし、`classify(c: char)` で種類を決める。
  - `CharSets` は `HashSet<char>` で、`chars_*` の設定を `.chars().collect()` で作る。複数のコードポイントの項目は、ばらばらのコードポイントになる。並び全体は一致せず、部品（VS16 など）だけが単独で一致してしまう。
- 自動ウェイト（`wait_inserter.rs` の `append_with_per_char_wait`）も `text.chars()` で 1 コードポイントごとに `\_w` を入れる。今はトークンが 1 コードポイントなので、根の原因ではない。
- budoux の改行（`line_breaker.rs`）:
  - `PlainChar { ch: char, … }` をコードポイントごとに作る。
  - 幅は `UnicodeWidthChar::width_cjk`。ZWJ と VS16 は 0 だが、♀（East Asian Ambiguous）と肌色の修飾子は 2 と数える。🙆‍♀️ は約 4 桁になる（表示は 2 桁）。
  - budouy 0.2.2 も `char` の位置で区切る。そのため、クラスタの途中に改行が入りうる。
- Lua の側には、台詞を 1 文字ずつ処理する所は無い（`appearance.lua`・`sakura_builder.lua`・`actor_style.lua` はタグの `\` を見るだけ）。
- 依存: `unicode-segmentation` は `Cargo.lock` に無い（間接にも無い）。`unicode-width 0.2.2` はワークスペースの依存。
- マニュアルは「1 文字ずつ」「1 文字ごと」と書くが、「文字」を定義していない。
  - `book/src/lua/modules/pasta-sakura-script.md`（:94・:108-109・:52・:211）
  - `book/src/reference/pasta-toml.md`（:155-160・:667-672・:459・:521）
  - `book/src/internals/talk-output.md`（:221・:227）
  - `book/src/grammar/action-line.md:104` は「`\\` の 2 文字の間には何も入らない」と約束している。同じ不変条件をクラスタへ広げる。
- テスト: `sakura_script` の単体テストが 75 本、結合テストが 89 本ある。絵文字・ZWJ・異体字セレクタ・結合文字を使うものは 1 本も無い。

## Desired Outcome

- 1 つの書記素クラスタ（拡張書記素クラスタ。UAX #29）の途中には、自動ウェイトも budoux の改行も入らない。
- 自動ウェイトの種類の判定（`chars_period`・`chars_comma`・`chars_strong`・`chars_leader`・行頭と行末の禁則）は、クラスタ全体を 1 文字として扱う。`chars_*` に書いた組み合わせの絵文字も、1 文字として一致する。
- budoux の行の幅は、クラスタを 1 つの表示単位として数える（🙆‍♀️ は 2 桁）。
- 1 つのコードポイントだけの文字（今の日本語の台詞の大部分）の出力は、1 文字も変わらない。
- マニュアルが「1 文字 ＝ 書記素クラスタ」と定義し、スキルの references を作り直してある。

## Approach

推奨: **切り出しをクラスタ単位にし、後ろの段は「1 トークン ＝ 1 クラスタ」に乗る**。

- `tokenizer.rs`: タグでない所では、次の書記素クラスタを 1 トークンにする。`CharSets` を `HashSet<String>` にし、`chars_*` の設定もクラスタに分けて作る。
- `wait_inserter.rs`: 1 トークンに 1 つのウェイト（単独の `\` の規則は保つ）。
- `line_breaker.rs`:
  - `PlainChar` にクラスタ（`&str`）を持たせる。
  - 幅は `UnicodeWidthStr::width_cjk`。unicode-width 0.2 の文字列の幅が ZWJ の並びと VS16 を 1 単位に数えるかは、確かめていない。要件定義か設計で確かめる。
  - budoux の区切りの位置（`char` の番号）をクラスタの境界へ寄せ、クラスタの途中の区切りを捨てる。
- クラスタの切り方は、要件定義か設計で決める。
  - 候補 A: `unicode-segmentation` を足す（UAX #29 の全規則。小さく定番）。
  - 候補 B: ZWJ・異体字セレクタ・肌色の修飾子・結合文字だけを見て伸ばす手書きのループ（UAX #29 の一部。`ponytail:` の注記が要る）。
  - 推奨は A。手書きは国旗（地域指示子の対）・ハングルの字母・Prepend などで漏れ、直すたびに規則を足すことになる。

## Scope

- **In**:
  - 切り出し・自動ウェイト・budoux の改行（`break_lines` を含む）のクラスタ単位化。
  - `chars_*` の項目をクラスタとして扱うこと。
  - 退行テスト（報告の 2 例、`chars_period` に組み合わせの絵文字を書いた例、肌色の修飾子、国旗、結合文字、縦書きの budoux）。1 コードポイントの文字の出力が変わらないことの特性化テスト。
  - マニュアル（上の 4 ページ）と、スキル references の再生成。
- **Out**:
  - 表示の幅の正確さを、SSP のバルーンの描画と完全に一致させること（フォントしだい）。
  - タグの直後に結合文字が来る場合（`\n` ＋ U+0301 など）の扱い。タグの読み取りが先に止まり、結合文字は単独のクラスタになる。これを良しとする。
  - Lua の側（台詞を文字単位で処理する所が無い）。

## Boundary Candidates

- クラスタの切り出しと文字の種類の判定（`tokenizer.rs`）。
- ウェイトの挿入（`wait_inserter.rs`）。トークンの粒度が変わるだけ。
- budoux の改行と幅（`line_breaker.rs`）。budoux の `char` の位置とクラスタの対応づけ。

## Out of Boundary

- budoux の禁則（バックログの「budoux の自動改行の禁則」）。別の話として残す。
- 単語・シーン名の正規化（検索のキー）。

## Upstream / Downstream

- **Upstream**: `vertical-balloon-lines`（完了。列を動かすタグでの数え直し・列の持ち越し）。`sakura-script-newline`・`paragraph-break-tag-only-talk`（完了。段落区切りと単独の `\` の守り）。budouy 0.2.2・unicode-width 0.2.2。
- **Downstream**: 絵文字を使うゴースト（窓際のぱすたさん）。`novel-talk-flow`（ノベル向けの出力。台詞ごとのクリック待ち）は同じ出力の流れに乗る。

## Existing Spec Touchpoints

- **Extends**: なし（完了した spec の約束「1 文字ごと」を、クラスタへ定義し直す）。
- **Adjacent**: `vertical-balloon-lines`（完了。`line_breaker.rs` の数え直しと持ち越し）。`actor-display-style`（完了。装いのタグは `put_break` を通らず、字数に数えない）。`sakura_builder.lua` は触らない見込みなので、`novel-talk-flow`・`cue-command-runtime`・`scene-anchor-link` と同じウェーブに置ける。

## Constraints

- x86 と x64（MSVC・静的 CRT）で動くこと。依存を足すなら `cargo-deny`（リリース CI の gate）を通ること。
- 1 コードポイントの文字だけの台詞では、出力を 1 文字も変えないこと（既存のテストの期待値を書き換えない）。
- 単独の `\` の直後の守り（`vertical-balloon-lines` の決定 9）を保つこと。
- 要件定義は Opus でよい（文法・保存の形・公開の Lua の約束に触らない。切り方の選択は決め直せる）。
