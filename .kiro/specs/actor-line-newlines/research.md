# ギャップ分析: actor-line-newlines

## 要約

- **対象**: `[ghost] continuation_newlines`（既定 `0`）を足し、継続行（`ContinueAction`）の最初の見える字の前に `\n[N]` を 1 つ入れる。対象は継続行だけ（2026-11-05 の見直し後の brief）。
- **発見の規模**: 既存の拡張（Extension）。設定の読み込み・出力側の状態機械・生成する Lua の 3 か所に、既にある型（`spot_newlines` の遅延方式・`put_break`・`apply_shiori_defaults`）をなぞって足す。Rust の `talk_to_script`・budoux の折り方は変えない。
- **主な難所**: 「既定 `0` で 1 バイトも変えない」（要件 6.1）と「設定しても、規則 6・7 で入れなかった境目は今と同じ」（要件 4.5）。今の `merge_consecutive_talks` は同じアクターの連続した `talk` を 1 つに結合してから `talk_to_script`（ウェイト・budoux）に渡すので、継続行の印が結合を切ると、改行を入れない所でもウェイトや budoux の折り位置が変わりうる。印は「結合を切らない」形で運ぶ必要がある。
- **推奨の方向**: 案 B（印を結合後の台詞の中の「継ぎ目」として持ち、改行を入れると決めた継ぎ目でだけ台詞を分けて `put_break` を通す）。規則 6 の判定は Rust の `column_move` と同じ分類（`\n`・`\n[…]`→次の列、`\c`→最初の列、`\_l[…]`→飛ぶ）を Lua 側に写す形になり、`ColumnMove` の写しの決まり（片方だけ直さない）に入る。
- **工数と危険度**: M（3〜7 日）／中。既存の型に沿うが、継ぎ目の扱いとバイト同一の検証に手間がかかる。

## 書き方の案（ぱすた開発セッション提出）

ぱすた開発セッション（ghost_dev）が出した「今 → 新しい版」の例の要約。想定の設定は `[ghost] continuation_newlines = 1.8`。原文（辞書の全文と、出てほしいさくらスクリプト）は ghost_dev 側にある。設計で出力の期待値を固めるときは原文と突き合わせる。

| 例 | 今の辞書 | 新しい版の辞書 | 出てほしい出力 |
| -- | -------- | -------------- | -------------- |
| (a) talk_poem `＊雑談_ポエム_会ってみたい` | 継続行の頭に `＠改行` | 継続行の頭の `＠改行` を消す | 継続行の前に `\n[180]` が入り、今と同じ位置で列が替わる。書き直した行 `ぱすた：＠左　…` には入らない（書き直し行は対象外、かつ頭が `\_l[…]`） |
| (b) talk_misc `・オチ_生存戦略` | 局所シーンの頭の行に `＠改行` | 変えない（局所シーンの頭は手の `＠改行` のまま） | 出力は今と同じ |
| (c) talk_ojisan | おじさん（相手役）の継続行の頭に `＠改行`（`\n[180]\_u[0x3000]`） | 相手役の `＠改行` は残す | 相手役の継続行は改行が二重にならず今と同じ（規則 6）。話し手が替わった後のぱすたの行は `spot_newlines` だけ（規則 3） |
| (d) boot `＊レベルアップ`、ball `＊ボール_窓の外のお天気` | `＠改ページ` で始まる行・`＠左` の行、行の末尾に `＠選択肢左`（`\_l[-253,0]`） | 継続行の頭の `＠改行` を消す | `＠改ページ`・`＠左` で始まる行の前には入らない（規則 6）。行の末尾の `\_l[…]` は次の継続行の改行を止めない（規則 7 は改行だけ）。選択肢の前後は `choice_newlines`・`after_choice_newlines` のまま |

## 現状の調査

### 関係する資産

| 場所 | 今の働き | この機能との関係 |
| ---- | -------- | ---------------- |
| `crates/pasta_lua/src/code_gen/element_gen.rs` `generate_continue_action` | 継続行の各アクションを `act:actor_proxy("名前"):talk(…)` などとして出す。`generate_action_line` と同じ形で、行の境目は出力に残らない | 継続行の頭の印を出す場所。`generate_action_line` は変えない（書き直し行は対象外） |
| `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_local_scene_items` | `ActionLine`・`ContinueAction`・`VarSet`・`CallScene` などを振り分ける。`last_actor` は Call 行・変数代入行をまたいで残る | 規則 9（空行・コメント・変数代入・Call 行は行に数えない）は既にこの形。印を出すのは `ContinueAction` の枝だけでよい |
| `generate_action` と `record_span_since`（`code_gen/mod.rs`） | アクション 1 つにつき Lua 1 行を出し、`.pasta` の行との対応（`line_map`・ソースマップ）を記録する | 印を別の Lua 行で出すと、出力の行番号が全体にずれ、`line_map` の文字列と生成物（`sample.expected.lua` など）が大きく変わる。同じ行に載せる案もある（下の案の比較） |
| `crates/pasta_lua/pasta_scripts/pasta/act.lua` `group_by_actor`・`merge_consecutive_talks`・`is_all_empty_raw_script`・`is_speaker_token` | トークンをアクターごとにまとめ、同じグループの連続した `talk` を 1 つに結合する。トークンが無い・空の `raw_script` だけなら `build` が `nil` を返す | 印を独立したトークンにすると、結合が切れる（ウェイト・budoux の出力が変わる）、空の判定が変わる（`nil` が `\e` に化ける）、話者の遡り・グループの頭のタグ列（外見の復旧）が変わる。どれも要件 6.1 に触れる |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` `BUILDER.build` | `pending_break`（spot_newlines の保留）・`spot_has_text`・`spot_tail`（選択肢）・`columns`（budoux の列）の状態機械。区切りは `put_break` を通して列を進める。字の判定は `has_text`・`emits_text`、`\c` は `text_parts`・`clear_span` | 「継続行の保留」を足す場所。優先順位は `pending_break` ＞ `spot_tail == "choice"` ＞ 継続行の保留、で 1 つだけ出す（要件 3.5）。話し手の切り替え・`clear`・台詞の `\c`・`clear_spot`・選択肢で保留を捨てる |
| 同 `newline_tag`・`plain_newline_tag` | `spot_newlines`・`choice_newlines` は常に `\n[N]`、`after_choice_newlines` は `N` が 100 なら `\n` | 要件 1.4（`N` が 100 なら `\n`）は `plain_newline_tag` と同じ換算。brief の「`spot_newlines` と同じ」は型と読み込みの話で、換算は `plain_newline_tag` を使う |
| 同 `next_column`・`first_column` と Rust `line_breaker.rs` の `ColumnMove`・`column_move` | 列の動かし方の写し（片方だけ直さない） | 規則 6 の「位置を決めるタグ」は `column_move` が列を動かすと読む単位と同じ集合。Lua 側には分類の写しがまだ無い（`APPEARANCE.tag_at` で名前と引数は読める） |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` `SHIORI_ACT.new`・`build` | `CONFIG.get("ghost", "spot_newlines", 1.5)` などを読み、`BUILDER.build` の `config` に渡す | `continuation_newlines` を同じ経路で読み、渡す。既定のリテラル `0` を置く |
| `crates/pasta_lua/src/loader/config/sections.rs` `GhostConfig`・`mod.rs` `apply_shiori_defaults` | `[ghost]` の既定値の正本（SSOT）。書かなかったキーを `@pasta_config` に補う | `continuation_newlines: f64 = 0.0` を足す。補った結果 `@pasta_config` の `ghost` に `continuation_newlines = 0.0` が現れる（出力は変わらない） |
| `crates/pasta_lua/tests/loader/config_defaults_test.rs` | SSOT と、`act.lua` の既定リテラル・マニュアルの表の行・テンプレートの既定値が食い違わないかを検査する | 新しいキーも同じ検査に入れる。マニュアルの `[ghost]` の表の行とテンプレートに `continuation_newlines` と `0.0` 相当の表記が要る（表記の形は検査に合わせる） |
| `crates/pasta_lua/src/loader/cache.rs` | 生成した Lua のキャッシュを、クレートの版（`CARGO_PKG_VERSION`）で無効にする | 0.5.1 への版上げで古いキャッシュは捨てられる。開発中（同じ版）は古いキャッシュが残りうる（テストへの影響を設計で確かめる） |
| `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua` | builder の規則のテスト（spot_newlines・選択肢・budoux の列・`\c` など） | 規則 1〜11 のテストを足す場所 |
| `crates/pasta_lua/tests/transpiler/*`・`src/code_gen/*_tests.rs`・`tests/fixtures/sample.expected.lua` | 生成する Lua の文字列を比べるテスト（`actor_proxy(` を含む比較が約 53 か所） | 印を出すのは継続行だけなので、影響は継続行を含むテストに限られる。`sample.pasta` には継続行が約 27 行あるので、`sample.expected.lua`（と、テストが書き換える `sample.generated.lua`）は変わる。改行コードだけの差分をコミットに混ぜない |

### マニュアルとスキルの参照

- `book/src/reference/pasta-toml.md`: `[ghost]` の表（262 行付近）と節（`spot_newlines`・`choice_newlines`・`after_choice_newlines` の並び）、テンプレートの既定値の行（140 行付近）、`budoux` の節の「pasta が台詞と台詞の間に出す区切り」（532 行付近）に継続行の改行を足す。
- `book/src/grammar/action-line.md`: 「行継続」（継続行は前の発言に続けて出力される）と「改行」（継続行の間の空行は改行にならない）を、既定のときと設定したときに分けて書き直す。
- `book/src/internals/talk-output.md`: 状態の表（`pending_break` などの並び）、区切りの表、`BUILDER.build` の `config` の項目、境界の受け渡しの表に足す。
- そのほか `after_choice_newlines` を挙げている章（`book/src/lua/modules/pasta-config.md`・`book/src/internals/loader.md`・`book/src/internals/internal-modules.md`・`book/src/grammar/block-structure.md`・`book/AUTHORING.md`）は、`[ghost]` のキーの一覧を持つかを見て、持つなら足す。
- スキルの `references/` のうち生成ヘッダのあるものは `node book/tools/gen-skill-refs.mjs` で作り直す（手で直さない）。`pasta-ghost-authoring/references/pasta-toml.md`・`block-structure.md`、`pasta-lua-coding/references/pasta-config.md`・`internal-modules.md` などが当たる。

## 要件と資産の対応

| 要件 | 資産 | ギャップ |
| ---- | ---- | -------- |
| 1.1〜1.6 設定 | `GhostConfig`・`apply_shiori_defaults`・`SHIORI_ACT.new`・`config_defaults_test` | **Missing**: キーと既定値・読み込み・渡す所。**Constraint**: SSOT の検査がマニュアルの表とリテラルを見る |
| 1.3 `N` が 0 以下 | `newline_tag` の換算 | **Unknown**: 負の値の扱いは brief に無い（OPEN QUESTION 1） |
| 1.5 数でない値 | `spot_newlines` は型の検査をせず、使う所（`math.floor(ratio * 100)`）で Lua のエラーになる | **Constraint**: 「同じ扱い」なので、新しい検査は足さない。既定 `0` のときに値を読まずに済ませると、数でない値がエラーにならない場面が出る（設計で `spot_newlines` と合わせる） |
| 2.1・2.2・2.5 継続行だけ・出力の並びで判定 | `generate_continue_action`、builder の `last_actor`・`spot_has_text` | **Missing**: 継続行の頭の印と、それを builder まで運ぶ道 |
| 2.3 行に数えないもの | `generate_local_scene_items` | 既にある（印を継続行の枝だけで出せば満たす） |
| 2.4 字の無い継続行で重ねない | `pending_break` と同じ真偽 1 つの保留 | **Missing**: 保留の状態 1 つ |
| 2.6 作者が書いたものを変えない | `put` / `put_raw`・`head_written_by_author` | **Constraint**: 継ぎ目で台詞を分けると、分けた後ろの片の頭の扱い（守る `put` か守らない `put_raw` か）を決め直す必要がある |
| 3.1〜3.5 他の区切りとの重なり | `pending_break`・`spot_tail`・`clear`・`text_parts`・`clear_spot` | **Missing**: 保留を捨てる所（切り替え・選択肢・`\c`・`clear_spot`）と優先順位 |
| 4.1・4.4 行の頭の位置を決めるタグ | Rust `column_move`、Lua `APPEARANCE.tag_at` | **Missing**: Lua 側の分類の写し。**Constraint**: `ColumnMove` の写しの決まり |
| 4.2・4.3 前の行の末尾 | 出力の追跡は `tracking_buffer` の `lone` だけ | **Missing**: 「最後の見える字の後に改行を出したか」の追跡（`talk_to_script` の結果に入った改行・`newline` トークン・`sakura_script` の `\n` を含む） |
| 4.5・6.1 バイト同一 | `merge_consecutive_talks`・`talk_to_script` | **Constraint**: 結合を切らないこと。**Research Needed**: 継ぎ目で分けたときのウェイト（句読点の後のウェイト）と budoux（前後の文字を見て切れ目を決める）への影響 |
| 5.1・5.2 budoux の列 | `put_break`・`next_column` | 既にある（`put_break` を通せば満たす） |
| 6.3 手書きからの移行 | — | **Research Needed**: 継ぎ目で分けて改行を入れたとき、手書き（結合した 1 つの台詞の中の `\n[180]`）と比べて改行の位置と幅は同じだが、表情のタグと改行の順（`\s[…]\n[180]` か `\n[180]\s[…]` か）と、budoux の折り位置が変わりうる。要件は「字と字の間・幅」だけを求めている |
| 7.1〜7.5 マニュアル | 上の「マニュアルとスキルの参照」 | **Missing**: 記述。**Constraint**: 生成物は生成器で作り直す・`config_defaults_test` の表の形 |

## 統合の難所

1. **結合を切らない**: 今の出力は、同じアクターの連続した `talk` を 1 つに結合してから変換する。継続行の印が独立したトークンだと結合が切れ、既定 `0` でも出力が変わる（要件 6.1 違反）。また印だけが積まれた出力で `ACT_IMPL.build` の空の判定が偽になり、`nil` だった応答が `\e` だけになる。印は、結合・空の判定・話者の遡り（`actor_proxy` の直前の話者）・グループの頭のタグ列（外見の復旧）・`BUILDER.speaks` のどれからも見えない形にする。
2. **印の置き場所とグループ**: 継続行の前に Call 先で別の話し手が字を出していると、`group_by_actor` は話者でないトークンを今のグループ（別の話し手のグループ）に入れる。印を独立したトークンにすると、印が別の話し手のグループに入り、その後の切り替えで捨てる必要がある。印を次の話者のトークン（`talk`・`sakura_script`）の欄として持てば、このずれは起きない。
3. **規則 6・7 は行をまたいで見る**: 継続行の頭のタグは、結合した台詞の中・別の `sakura_script` トークン（DSL の行に書いた `\n`・`\_l[…]`・`\c` は `sakura_script` になる）・`newline` トークンにまたがる。前の行の末尾の改行も同様。判定は「最後の見える字から次の見える字まで」の区間を、トークンの境目と結合した台詞の中の継ぎ目の両方で見る必要がある。
4. **区切りの置き場所**: `spot_newlines` は字を出すトークンの直前（テキストの頭のタグより前）に出る。継続行の改行も同じ置き場所にすると、`　　：＠注目　みんなに…` は `\n[180]\s[注目]みんなに…` になる。手書きの `＠注目　＠改行　…` の出力は `\s[注目]\n[180]…` なので、見た目は同じだがバイトは違う（要件 6.3 は字と字の間と幅だけを求める）。ぱすた開発セッションの「出てほしいさくらスクリプト」と照らして、どちらに置くかを設計で決める。
5. **生成する Lua の形と行番号**: 印を別の Lua 行（例 `act:continuation()`）で出すと、以後の Lua の行番号がずれ、`line_map` の文字列・ソースマップ・デバッガの行の対応・比較用の生成物が変わる（`record_span_since` で継続行の span を記録すれば対応は保てる）。同じ行に載せる（例: 継続行の最初のアクションの `actor_proxy` に印の引数を足す、最初の呼び出しの前に `act:continuation();` を同じ行で置く）と行番号は変わらない。ただし最初のアクションが関数呼び出し（`expr_fn`）だと、関数の中で積まれた台詞に印が付く順序の問題がある。
6. **既定のリテラルと SSOT の検査**: `act.lua` の `CONFIG.get("ghost", "continuation_newlines", 0)` の書き方（`0` か `0.0` か）と、`GhostConfig` の `f64` の表記（`{:?}` で `0.0`）を、`config_defaults_test` の照合に合わせる。
7. **隣の spec**: `cue-command-runtime` は場面に入ったときと Call から戻ったときにアクター指定行を台本へ流し、同じ生成の場所（`scope_gen.rs`・`element_gen.rs`）を触る。`scene-anchor-link` は `act.lua` を触る。同じウェーブに置かない（brief）。

## 実装の案

### 案 A: 印を独立したトークンにする（既存の部品を拡張）

- 生成: 継続行の頭で `act:continuation()`（名前は仮）を出す。`ACT_IMPL.continuation` が `{ type = "continuation" }` を積む。
- act.lua: `merge_consecutive_talks` が印を飛ばして結合を続ける、`is_all_empty_raw_script` が印を無視する、`group_by_actor` が印を次の話者のグループへ回す、などの手当てを足す。
- builder: 印で保留を立て、字を出すトークンの直前で出す。
- ✅ 生成と蓄積が素直。Lua から `act:continuation()` を呼べば同じ挙動になる（ただし要件では Lua の台詞は対象外）。
- ❌ 結合を飛ばすと、結合した台詞の中のどこが継ぎ目かが失われ、継ぎ目で改行を入れられない。結合を切ると要件 6.1・4.5 に触れる。手当ての数が多く、どれか 1 つの漏れで既定の出力が変わる。

### 案 B: 印を結合後の台詞の「継ぎ目」として持つ（推奨の方向）

- 生成: 継続行の最初のアクションに印を付ける（同じ Lua 行に載せる形か、別の行か、は設計で決める）。印は次に積む話者のトークンに `continuation = true` のような欄として付く。
- act.lua: `merge_consecutive_talks` は今までどおり結合し、結合した `talk` に「継ぎ目の位置（バイト位置の並び）」を残す。結合しない境目（`sakura_script` など）では、トークンの欄のまま残す。空の判定・話者の遡り・グループは何も変わらない。
- builder: 継ぎ目を通るとき、保留の条件（同じ話し手・吹き出しに字がある・規則 6・7）を見て、入れると決めた継ぎ目でだけ台詞をそこで分け、前の片 → `put_break` → 後ろの片の順に変換する。入れない継ぎ目では今までどおり 1 つの台詞として変換する。既定 `0` では継ぎ目を一切見ない。
- ✅ 要件 6.1・4.5 のバイト同一を構造で守れる（入れないときは変換の入力が今と同じ）。グループ・空の判定・外見の復旧に手を入れない。
- ❌ 継ぎ目の位置を結合のたびに足し合わせる手間と、分けた後ろの片の頭の扱い（`put` / `put_raw`）・字の判定（`text_parts` を片ごとに）の決め直しがある。builder の複雑度（今 28）がさらに上がるので、継ぎ目の処理を局所関数に出す必要がある。

### 案 C: 混成（生成は案 A の形、運び方は案 B）

- 生成は独立した呼び出し `act:continuation()` にし、`ACT_IMPL.continuation` はトークンを積まずに act の欄（「次の話者のトークンに印を付ける」）を立てるだけにする。以後は案 B と同じ。
- ✅ 生成する Lua が読みやすく（`act:continuation()` の 1 行）、最初のアクションが関数呼び出しのときの順序の問題も、印を立てた後に最初に積まれた話者のトークンに付く、と決めれば扱える。
- ❌ 別の行にすると行番号がずれる（統合の難所 5）。同じ行に `act:continuation(); …` と並べれば避けられるが、生成の 1 行 1 アクションの形が崩れる。

## 工数と危険度

- **工数: M（3〜7 日）** — 設定・生成・builder の 3 か所は既存の型をなぞるが、継ぎ目の扱いと、規則 1〜11 のテスト、既定 `0` と相手役の行のバイト同一の検証、マニュアル 3 章＋生成物の更新がある。
- **危険度: 中** — 未知の技術は無いが、既定の出力を 1 バイトも変えないことを、結合・ウェイト・budoux・外見の復旧・空の判定のすべてで守る必要がある。

## 設計への申し送り

- **推奨**: 案 B または案 C（印を結合から見えない形で運び、入れると決めた継ぎ目でだけ台詞を分ける）。案 A は要件 6.1・4.5 を守る手当てが多く、漏れやすい。
- **決めること**: 印の名前と生成する Lua の形（同じ行か別の行か・行番号とソースマップ）、継ぎ目の表し方、区切りの置き場所（頭のタグの前か後か。ぱすた開発セッションの期待値と照合）、規則 6 の分類の Lua の写し（`column_move` と同じ集合。`\n[half]` も次の列、`\c[…]` は数えない）、規則 7 の追跡の持ち方、保留を捨てる所の一覧、`config_defaults_test` の表記。
- **Research Needed**:
  1. 継ぎ目で台詞を分けて変換したとき、ウェイトの挿入（句読点の後・行の終わりの扱い）と budoux（前後の文字から切れ目を決める）の出力が、結合したまま変換したときとどう違うか。改行を入れる継ぎ目だけで分けるなら、違いは「改行を入れた所の前後」に限られるかを確かめる。
  2. hello-pasta の辞書（継続行のある `.pasta` が 6 ファイル）で、既定 `0` の出力が変わらないことを見本の出力のテストで確かめられるか。
  3. 開発中（同じ版のまま）の生成キャッシュが、印の無い古い Lua を返してテストを惑わせないか。
  4. ぱすた開発セッションの「出てほしいさくらスクリプト」の原文（書き方の案 (a)〜(d)）を受け取り、設計の期待値に写す。

## OPEN QUESTIONS

1. **`continuation_newlines` が負の値、または 100 倍して切り捨てると 0 になる値（`0.009` など）のとき、何を出すか。** brief は「既定 `0` は何も入れない」だけを決めている。`spot_newlines` の換算をそのまま当てると、`-0.5` は `\n[-50]`、`0.009` は `\n[0]` になる。
   - **推奨**: `N`（値を 100 倍して切り捨てた整数）が `0` 以下なら何も入れない（要件 1.3 に仮定として書いた）。`0` が「入れない」の意味なので、`0` より小さい値も同じに読むのが自然で、`\n[-50]`（ベースウェアによって行を戻す・無視するなど挙動が定まらない）を出さずに済む。
   - **例**: `continuation_newlines = -1` → 継続行の前に何も入れない（`0` と同じ）。`continuation_newlines = 0.5` → `\n[50]`。
