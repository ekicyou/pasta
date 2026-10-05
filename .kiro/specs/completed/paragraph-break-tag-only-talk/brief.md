# Brief: paragraph-break-tag-only-talk

> **ステータス**: 未着手（2026-10-05、areka セッション「emo2初回起動」からの申し送りで起票）。Phase 11 Wave 3（バグ修正。`call-execution-correctness` と並走可）。着手するときは `/kiro-start paragraph-break-tag-only-talk` で開始する。

> **範囲の拡大（2026-10-05、設計ディスカッションで決定）**: タグの読み取りを SSP（UKADOC）にそろえる修正を本仕様に取り込んだ。対象は Rust のトークナイザ・Lua の外見の観測・DSL の文法の 3 か所と、VS Code の文法定義、マニュアルである。エスケープ `\%` と囲み `\_?…\_?` の読み方も決める。下の Scope・Out of Boundary・Constraints は拡大後の内容に改めた。Approach の節は起票時のまま残す（決定は `research.md` と `requirements.md` が正）。

## Problem

段落区切りの改行（`\n[spot_newlines×100]`）は、同じスポットに字があるときだけ出す決まりである。しかし `sakura_builder` は、さくらスクリプトのタグしか含まない `talk` も「字あり」と数える。そのため、字の無い出力のあとに余分な改行が出る。

表情を単語で返すゴースト（`＠通常` などの単語の値が `\s[…]` だけ）でこれが起きる。バルーンに大きな空きができる（areka の実機で確認）。

- (a) 字を 1 つも出していないスポットの最初の台詞の前に `\n[150]` が出る。
- (b) 字の無い行（`むらさき：＠通常`）が `\n[150]` を出し、続く台詞の前でもう 1 つ出る。改行が 2 つ重なる。

## Current State

申し送り元は emo2（pasta.dll 0.3.7・埋め込み pasta_scripts）の `＊OnFirstBoot`。2026-10-05 に現行 main（b7379789）でも同じ分岐が残っていることをコード上で確認した。

- **ベースウェアによらない**: areka に加え、SSP でも同じ空きが出ることを追報で確認した（開発版のゴースト・pasta.dll の版は未確認）。
  - SSP が返した台本の原文には `\p[0]\n[150]\s[1000]\![bind,…]`（字なし）→ `\p[1]\n[150]\s[静観]僕は…` → `\p[0]\n[150]\s[1000]…つまり、、` とあり、むらさきの欄で `\n[150]` が 2 回効いている。
  - エモの最初の台詞の前にも `\p[1]\n[150]\s[笑顔]僕はエモ。` が付く。冒頭のエモの出力は `\p[1]\s[静観]\1\![move,…]` で、字は無い。
- **証拠**: areka のワークツリー `C:\home\maz\git\areka\.claude\worktrees\areka-p0-translate-pipeline-2cf68b\.kiro\specs\areka-P0-budoux-reveal-reflow\evidence\` に置かれている。
  - 台本の原文 `ssp-onfirstboot-script.txt`
  - SSP の画面 `12-ssp-pasta-blank-lines.jpg`
  - areka の画面 `04-pasta-blank-lines.jpg`
  - 要件フェーズでは、この台本の原文をテストの期待値の材料にできる。

- `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` の `BUILDER.build` には、内側のトークンを振り分ける分岐がある。
  - **S3**（184 行付近）: `inner.type == "talk" and inner.text ~= nil and inner.text ~= ""` のとき、保留中の改行を出力し、`spot_has_text[last_spot] = true` にする。本文がタグだけかどうかは見ない。
  - **S4**（195 行付近）: 状態を変えずに出力だけする。
- 経路:
  - 単語参照 `＠通常` は `act.エモ:talk(act.エモ:word("通常"))` になる。
  - `ACT_IMPL.talk`（`act.lua` 205 行付近）は値を `tostring` して `talk` トークンにする。単語の値がタグ文字列でも、種別は `talk` のまま。
- マニュアルとの食い違い:
  - 利用者向けの `book/src/reference/pasta-toml.md` の `spot_newlines`（250 行付近）は「すでに台詞を出したスポットへ戻るとき」に改行すると書く。タグだけの出力は台詞ではないため、現行の挙動はこの記述と食い違う。
  - 内部設計の `book/src/internals/talk-output.md` 156 行付近は「空でない `talk`」を基準と書いており、現行の挙動のとおり。
- 再利用できる部品:
  - `appearance.lua` の `tag_at`・`NAME_PATTERN`・`ARG_PATTERN` は Lua 側のタグ読み取り。`\\` を読み飛ばす扱いもある。
  - Rust の `Tokenizer::SAKURA_TAG_PATTERN`（`sakura_script/tokenizer.rs` 127 行付近）は Rust 側のタグの定義。
- テスト: `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua`。
- 段落区切りの規則は `sakura-script-newline`（完了。完全遅延方式）が決めた。最初の発言より前の表示制御の扱いは `act-token-grouping-fix`（完了）が変えた。

## Desired Outcome

- タグを除くと字が残らない `talk` は、段落区切りの判定では字なしとして扱われる。保留中の改行を出さず、`spot_has_text` を真にしない。
- 申し送りの (a)・(b) の並び（タグだけの単語の行、字の無い行、続く台詞）で、余分な `\n[150]` が出ない。字のあるスポットへ戻ったときの改行は、従来どおり 1 つ出る。
- マニュアルの内部設計 `internals/talk-output.md` が新しい判定を書き、利用者向けの `reference/pasta-toml.md` と一致している。スキル `references/` を再生成している。
- 修正を固定するテストがある。申し送りの最小例をもとにした Lua のテスト。

## Approach

要件フェーズで次を決める。推奨を併記する。

- **「字」の定義**: タグを除いた残りが空なら字なしとする。境界になるタグの扱いを決める。
  - `\\`（バックスラッシュそのものを表示する）は字として数える（推奨）。
  - `\_u[…]`・`\&[…]`（文字を表示するタグ）は字として数えるか。推奨は数える。
  - `\n` などの改行タグだけの `talk` は字なしとする（推奨）。
  - 空白だけの残りの扱い。
- **判定の位置**: 次の 2 案から選ぶ。
  - (1) `sakura_builder.lua` の中で、`appearance.lua` と同じタグの文字集合を使って判定する。推奨は (1)。Lua 側だけで閉じ、`tag_at` を共有できる。
  - (2) Rust の `@pasta_sakura_script` に判定関数を足し、`SAKURA_TAG_PATTERN` を使う。
- **保留中の改行の位置**: タグと字が混ざる `talk`（`\s[10]やあ`）では、改行は従来どおり `talk` の前に出す（変えない）。
- **ゴースト側の書き方の扱い**: `＠単語` は DSL の正規の書き方なので、pasta 側で直す。表情を `talk` 以外のトークンにする変更（単語の値の種別判定など）は範囲外。

## Scope

- **In**:
  - `sakura_builder.lua` の段落区切りの判定（S3・S4 の振り分け）
  - 字の判定に使うタグの読み取り（`appearance.lua` との共有を含む）
  - タグの読み取りを SSP にそろえること（名前の規則、引数のエスケープと引用、エスケープ `\\`・`\%`、囲み `\_?…\_?`）。対象は `sakura_script/tokenizer.rs`・`appearance.lua`・`grammar.pest`・`pasta.tmLanguage.json`
  - DSL で `\%` を受理すること（`element_gen.rs` の Escape の腕の 1 か所）
  - 3 つの読み取りの一致を保つ適合テストと、修正を固定するテスト
  - マニュアルの該当章（文法・Lua モジュール・内部設計・`pasta.toml`）の更新と、スキル `references/` の再生成
  - 破壊的変更の告知
- **Out**:
  - 段落区切りの規則そのもの（完全遅延方式・切替時の再評価）の変更
  - BudouX の改行と表示済みの字の再配置（areka 側の `areka-P0-budoux-reveal-reflow` が持つ）
  - 外見の観測・復旧の規則（何を記録し、いつ復旧するか）。変えるのはタグの切り出しと名前の判定だけ
  - タグの意味の解釈、既知のタグ名の一覧
  - Lua から囲みの開きと閉じを別々のトークンで積む書き方

## Boundary Candidates

- 段落区切りの判定（`sakura_builder.lua` の `BUILDER.build`）
- タグの読み取りの共有（`appearance.lua` の `tag_at` を判定から使えるようにする）
- マニュアル・生成スキル

## Out of Boundary

- `pasta_scripts/pasta/act.lua` のトークン化とグループ化（`call-execution-correctness` が Wave 3 で持つ）
- `element_gen.rs` の Call の腕（同上）
- ウェイトの挿入と BudouX の改行の処理そのもの（`wait_inserter.rs`・`line_breaker.rs` のコード。正規表現を受け取って使うだけ）
- areka（バルーンの描画）

## Upstream / Downstream

- **Upstream**:
  - `sakura-script-newline`（完了。段落区切りの規則）。
  - `act-token-grouping-fix`（完了。発言より前の表示制御の出力）。
  - `actor-surface-restore`（完了。`appearance.lua`）。
- **Downstream**: 表情を単語で返すゴースト（emo2 ほか）

## Existing Spec Touchpoints

- **Extends**: `sakura-script-newline`（完了）の has-text の意味論を、利用者向けマニュアルの「台詞」の意味に合わせる
- **Adjacent**: areka の `areka-P0-budoux-reveal-reflow`（同じ実機の空きの別原因）

## Constraints

- マニュアルが権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する。
- 読みの変わらないテキスト（`requirements.md` の用語）だけのトークの出力は、バイト単位で変えない。それ以外で出力やパース結果が変わる書き方は、破壊的変更として告知する。
- タグの定義は Rust を正とし、Lua と pest は写しを持つ。同じ事例の表を 3 か所に通す適合テストで一致を保つ。`appearance.lua` は `@pasta_*` を `require` しない。
- 並走条件（Wave 3）: 編集するソースは `sakura_builder.lua`・`appearance.lua`・`sakura_script/tokenizer.rs`・`grammar.pest`・`element_gen.rs` の Escape の腕・`pasta.tmLanguage.json` と、そのテストに限る。`pasta_scripts/pasta/act.lua` と `element_gen.rs` の Call の腕は `call-execution-correctness` が持つため触らない。
