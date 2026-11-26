# ギャップ分析: line-end-wait

- 日付: 2026-11-26
- 対象: `requirements.md`（要件 1〜10・OPEN QUESTIONS 1〜10）と、今のブランチ（3324329d。コードは `main` の 1738c7c8 と同じ）のコード
- 方針: 決定はしない。コードで分かった事実と、取りうる案と、その得失を並べる。決めることは「設計の判断の項目」に番号で挙げる

## 1. まとめ

- 待ちの規則は `wait_inserter.rs` の `insert_waits` 1 か所にあり、値は `resolve_wait_values`（アクター → `[talk]` → 既定）で決まる。行末のキーを足すこと自体は小さい。難しいのは「どこが行末か」を変換に届けることで、今の `talk_to_script` は行の切れ目を知らない。
- **大きな見落としが 1 つある**: 辞書に直接書いた `\_w[500]` は、台詞（talk）のトークンでなく別の `sakura_script` のトークンになり、`merge_consecutive_talks` でつながれず、**別の `talk_to_script` 呼び出し**で変換される。だから「行末のすぐ後ろの作者の `\_w`」（要件 5.1）は、1 回の変換の中では見えない。組み立て（`sakura_builder.lua`）が次のトークンを先読みして変換に知らせる必要がある。`＠単語` が展開した `\_w[…]`（`＠改ページ` など）は台詞のテキストの中にあるので、変換の中で見える。2 つの道を両方扱う。
- 継ぎ目（`seams`）は今 `line_newlines` があるときだけ付く。行末の判定に使うなら、付ける条件を変えるか、行末のための別の印を足す。`actor-line-newlines` の設計は「既定ではトークンの表が今と同じ」を不変条件にしていて、Lua のテスト 4 つがトークンの表を丸ごと比べている。
- 推奨の候補は案 C（Rust の待ちの規則と設定は今のファイルを広げ、行末の位置は Lua の組み立てが見える字の判定で求めて `talk_to_script` の新しい 4 つ目の引数で渡す）。手間は M、危険は中（Lua と Rust の 2 か所に分かれた規則と、バイト単位で同じ出力の保証）。
- 調べが要るもの: mlua の `get::<i64>` が `1200.0` を整数として読むか、budoux の改行が待ちの位置の移動（要件 4.4）で変わらないか、`.claude/skills` の生成ファイルを書くときの許可。

## 2. 今のコード

### 2.1 変換の経路

```
辞書の 1 行 ──transpile──▶ act:line() / actor:talk("…") / actor:sakura_script("\_w[500]") / actor:talk(actor:word("笑顔"))
                          （code_gen/element_gen.rs。アクション行と継続行の頭に必ず act:line()）
act.token ──ACT_IMPL.build──▶ アクターのまとまり（group）── merge_consecutive_talks: 続く talk だけをつなぐ
                          （act.lua。sakura_script・surface・wait・raw_script などはつながず、そこで切れる）
group.tokens の 1 つずつ ──sakura_builder.lua: inner_token_to_string──▶
    talk・sakura_script: SAKURA_SCRIPT.talk_to_script(actor, join_lines(inner, column, line_newlines), column)
    wait: "\_w[ms]"（Lua から積んだ待ち。変換を通らない）
talk_to_script（sakura_script/mod.rs）: resolve_wait_values → tokenize → insert_waits → （budoux があれば）break_lines_impl
```

| 場所 | 今していること | この spec との関わり |
|---|---|---|
| `crates/pasta_lua/src/sakura_script/wait_inserter.rs` | `WaitValues`（5 値）、`insert_waits`。ふつうの字・リーダーはクラスタごと、句読点・行頭禁則は並びの最後に最大値 1 つ。タグ（`SakuraScript`）・行末禁則は並びを打ち切る。`flush_pending` が `値−50 > 0` のときだけ入れる | 行末の並びに行末の値を使う所、ふつうの字・リーダーの行末で行の途中の待ちを入れない所、すぐ後ろの `\_w` で自動の待ちを入れない所 |
| `crates/pasta_lua/src/sakura_script/mod.rs` | `talk_to_script(actor, text, column)`。`resolve_wait_values` はアクターの表を `get::<i64>(key).ok()` で読み、読めなければ `[talk]` の値（無ければ既定） | 行末のキー 5 つの読み出しと決まる順。行末の位置を受け取る口 |
| `crates/pasta_lua/src/sakura_script/tokenizer.rs` | `Token { kind, text }`（位置を持たない）。`\\`・`\%` のエスケープと `\_?…\_?`・`\_!…\_!` の囲みも `SakuraScript` | 位置で行末を渡すなら、トークンの長さを足して位置を求める。見える字の判定が Lua と違う（下の 2.4） |
| `crates/pasta_lua/src/loader/config/sections.rs` | `TalkConfig`（`#[serde(default)]`、5 値は `i64`） | 行末のキー 5 つ（書いたかどうかを区別する型。例: `Option<i64>`） |
| `crates/pasta_lua/src/loader/config/mod.rs` | `talk()` は `get_custom_config` の `try_into().ok()`。型が合わないと `[talk]` 全体が `None` になり、すべて既定に戻る。一方 `parse` は `[scene.alias]` と `actor_font::apply` で読み込みのエラーを返す | 要件 7 の書き間違いの検査を置く所の前例（`actor_font.rs` はキーの名前・アクター名・行番号を付けて `toml::de::Error` を返す） |
| `crates/pasta_lua/src/runtime/module_registry.rs` | `register_sakura_script_module` が `config.talk()` を `sakura_script::register` に渡す | 変更なしで済む見込み |
| `crates/pasta_lua/pasta_scripts/pasta/act.lua` | `ACT_IMPL.line` は `_line_newlines` が nil でも 0 でもないときだけ `_seam_next` を立てる。`talk`・`sakura_script` が `seams = {1}` を受け取る。`merge_consecutive_talks` が位置をずらしてつなぐ | 行末の判定の材料。付ける条件 |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` | `_line_newlines = CONFIG.get("ghost", "line_newlines", 0.0)`（既定は 0.0） | 既定では継ぎ目が付かない |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` | `join_lines`（`line_newlines` が nil・0 なら台詞を読まない）。`read_unit`・`unit_shows_text` が見える字の判定（`actor-line-newlines` の「見える字」）。`BUILDER.build` はまとまりの中のトークンを 1 つずつ変換する | 行末の位置を求め、次のトークンを先読みして変換へ渡す所。`BUILDER.build` は既に複雑度 28 で luacheck を外している |
| `crates/pasta_lua/src/code_gen/element_gen.rs` | `Action::SakuraScript` → `actor:sakura_script("…")`、`Action::WordRef` → `actor:talk(actor:word("…"))` | 辞書に書いた `\_w` と `＠単語` の `\_w` が別の道を通る理由 |
| `crates/pasta_check/src/validate/config.rs` | `read_actor_fonts` が pasta.dll の `actor_font.rs` と同じ取り出し方で `[actor.*.font]` を検査する | 要件 7.1 を読み込みのエラーにするなら、pasta_check にも同じ検査を写す（前例どおり） |

### 2.2 今の出力（特性化テストで固める物の材料）

コードから読める今の出力（ghost_dev の値 `normal=90, period=1200, comma=600, strong=750, leader=300`）:

| 入力（1 行） | 今の出力 | 根拠 |
|---|---|---|
| `雨？`（行末） | `雨\_w[40]？\_w[700]` | 並びの最後に `750−50` |
| `雨？\_w[500]`（辞書に直接） | `雨\_w[40]？\_w[700]\_w[500]` | `\_w[500]` は別の `sakura_script` トークン。別の呼び出しでタグのまま出る |
| `…言った？＠改ページ`（`＠改ページ`＝`\_w[2000]\c`） | `？\_w[700]\_w[2000]\c` | 単語の展開は talk。同じテキストの中で `SakuraScript` が並びを打ち切る |
| `…行ってきます」` | `す\_w[40]」`（「」」の後ろに待ち無し） | 行頭禁則は `pending_max_wait.unwrap_or(0)`。ふつうの字が先に吐き出されるので、並びの値は 0 → 入れない |
| `お母さんに怒られた‥‥` | `…た\_w[40]‥\_w[250]‥\_w[250]` | リーダーはクラスタごと |
| `行ってきます。＠笑顔` | `。\_w[1150]\s[5]` | タグの前で並びを吐き出す（要件 4.5 の位置は今のまま満たされる） |
| 行末が `\\`（エスケープ） | 待ち無し | `\\` は `SakuraScript`。Lua の見える字の判定では見える字 |

ghost_dev（`dic/*.pasta`）で、行末（行の最後）に `\_w[N]` を直接書いた行は 71 行（`\_w[500]` 38・`\_w[1200]` 20・`\_w[1000]` 10・`\_w[800]` 3。2026-11-26 に grep で数えた）。brief の表の 71 と同じ数になる。どれも、句読点などの字のすぐ後ろに辞書で直接書いたもの（＝別の `sakura_script` トークン）である。行末の `\_w` の前にタグを挟んだ行は見当たらない（`＠左　\_w[1000]＠照れ…` のように行の頭に書いた物はある）。

### 2.3 トークンの道が 2 つあること（要件 5 の見落とし）

| 作者の書き方 | トークン | 変換 | 行末の待ちとの関係 |
|---|---|---|---|
| `雨？\_w[500]`（辞書に直接） | talk `雨？` ＋ sakura_script `\_w[500]`（継ぎ目なし） | 2 回の `talk_to_script` | 1 回目の変換は、後ろに `\_w` が来ることを知らない。組み立てが先読みして知らせるしかない |
| `雨？＠間`（`＠間：\_w[500]`） | talk `雨？` ＋ talk `\_w[500]` → つながって 1 つ | 1 回 | 変換の中で、行末の並びの直後の `SakuraScript` トークンとして見える |
| `　　：\_w[500]じゃあ`（次の行の頭） | sakura_script（継ぎ目 `{1}`。`line_newlines` があるとき） | 別 | 要件 5.5 で対象外。継ぎ目の有無で見分けられる |
| `act:wait(500)`（Lua） | `wait` のトークン | 変換を通らない | 要件 5.5 で対象外 |

辞書の `\_w[500]\_w[300]` は、`parse_action.rs` で 1 タグごとに `Action::SakuraScript` になり、つながらない（要件 5.2 の「続けて 2 つ以上」は、トークンが 2 つ続く形で来る）。

talk と sakura_script をつなげば 1 回の変換に入るが、採らない方がよい。`BUILDER.build` は字を出すトークンかどうか（`text_parts`・`emits_text`）をトークンごとに見て、話し手の切り替えの改行（`pending_break`）・選択肢の後の区切り・`\c` の扱い・装い（`STYLE.through`）・外見の観測を決めている。つなぐと、表情だけの `sakura_script` の前と後ろのどちらに区切りが入るかが変わり、要件 6.1（バイト単位で同じ）を壊す。

### 2.4 見える字の判定が 2 つあること

- Lua（`sakura_builder.lua` の `read_unit`・`unit_shows_text`・`has_text`）: `actor-line-newlines` の「見える字」の正。`\\` のエスケープと囲み `\_?…\_?` は字を出すので見える字に数える。`raw_script` は数えない。
- Rust（`tokenizer.rs`）: エスケープと囲みは `SakuraScript`（待ちを付けない）。
- 行末の判定を Rust だけで行うと、「見える字」の定義が 2 か所に分かれる。Lua で求めて位置だけを渡せば定義は 1 か所のまま。ただし、Lua が行末とした字が Rust では `SakuraScript`（`\\`・囲み）になるときの扱いを決める必要がある（要件 4.6 と同じく「待ちを入れない・今と同じ」が素直）。

### 2.5 継ぎ目を付ける条件

- `ACT_IMPL.line` は `_line_newlines` が nil・0 なら何もしない。既定の `line_newlines` は 0.0（`apply_shiori_defaults`）なので、ほとんどのゴーストで継ぎ目が無い。
- `actor-line-newlines` の設計（`completed/actor-line-newlines/design.md` の 296 行目・417 行目）は「既定 0 では `seams` の欄はどのトークンにも現れない（トークンの表が今と同じ）」を不変条件にしている。`tests/lua_specs/line_newlines_test.lua` の 4 つのテストが、トークンの表を `toEqual({ type = "talk", actor = …, text = … })` で丸ごと比べている。
- 行末のキーはアクターの表を Lua で後から書き換えられる（要件 2.3）。`act:line()` の時点で「行末のキーが効くか」を決めて継ぎ目を付けるかを分けることは、まとまりの外の書き換えまで追えず難しい。継ぎ目は「いつも付ける」か「行末のための別の印をいつも付ける」が素直。

### 2.6 アクターのまとまりと行末

- 1 回の `talk_to_script` に渡る `actor` は、まとまりのアクター＝その行を話すアクター。要件 2.4 は今の作りのまま満たされる。
- Lua から `act.ぱすた:talk("a")`・`talk("b")` と積んだ台詞は継ぎ目が無く、1 つのテキストにつながる。まとまりの最後の見える字だけが行末になる（要件 3.7 は「継ぎ目が無ければ途中に行末なし」で自然に満たされる）。
- Call（要件 3.9）: 呼ばれたシーンの行も頭に `act:line()` を出すので、呼ぶ前の行の行末は、呼ばれた行の継ぎ目で分かる。呼ばれた先が Lua の関数で、継ぎ目なしに台詞を積むと、前の行の台詞とつながって行末が分からない（境界の例として要件か設計に 1 行あるとよい）。
- 行末の後ろに、同じまとまりの中で `surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`・`raw_script` のトークンが続くことがある。先読みで「見える字を出さないもの」として読み飛ばすのはどれか（特に `choice` は字を出す）を決める必要がある。

### 2.7 設定と書き間違い

- `[talk]` は `try_into().ok()`。行末のキーを `TalkConfig` に `Option<i64>` で足すだけだと、`script_wait_strong_eol = "1200"` で `[talk]` 全体が黙って既定に戻る（要件 7.2 に反する）。`PastaConfig::parse` の中で、`actor_font::apply` と同じ形で `[talk]` と `[actor.*]` の行末のキーを先に検査して `toml::de::Error` を返せば、要件 7.1・7.2 を満たせる。前例があるので新しい作りは要らない。
- pasta_check（`validate/config.rs`）は `actor_font` の検査を写している。読み込みのエラーにするなら pasta_check にも写す（「検査を関門」）。
- アクターの表（Lua）は `get::<i64>(key).ok()`。mlua の整数の読み方では、`1200.0` のような整数の値の小数や、数字の文字列 `"1200"` が整数として読める見込みがある（**要確認**）。そうなら、`pasta.toml` の `[actor.*]` に `1200.0` と書いた今の `script_wait_*` は効いていて、`[talk]` に書いたものは全体を既定に戻す、という食い違いが今もある。OPEN QUESTION 8（小数を読み込みのエラーにする）を決めるときの材料。
- `TalkConfig` の構造体の書き方は、テストのほとんどが `..TalkConfig::default()` を使っているので、欄を足しても壊れるテストは少ない。

### 2.8 ノベル向けの出力

- 今のコードに、ノベル向けの出力（自動の待ちを入れない出力・`\x`）はまだ無い（`pasta_scripts` に `novel`・`\x` の分岐が無い。`novel-talk-flow`・`novel-areka-contract` は brief と仮の設計だけ）。
- `novel-talk-flow` の仮の設計（`provisional-design.md` 87・92 行目）は、クリック待ちを 1 本の**末尾**に付ける。行末の待ちは組み立ての出力の中に入るので、`\x` より前に来る。要件 8.2 は今の作りのまま自然に満たされる見込み。
- `novel-areka-contract` の取り決めでは「高速再生の 1 本に自動の待ちを入れない」だったが、2026-11-20 の追記で「pasta は高速再生のときも同じさくらスクリプトを返す」（areka が命令で入切）に改まっている（`brief.md` 372 行目）。要件 8.1 は、今は対象になる出力が無く、後から入る出力への約束になる。

### 2.9 `talk_to_script` を直接呼ぶ所

- pasta の `pasta_scripts` で `talk_to_script` を呼ぶのは `sakura_builder.lua` の 1 か所だけ。ghost_dev のゴーストの `scripts/` にも直接の呼び出しは無い（`profile/pasta/pasta_scripts/` の写しだけ）。
- 組み立ては、まとまりの中のトークン 1 つずつを渡すので、テキストの終わりが行末とは限らない（行の途中の `\s[5]` を辞書に直接書くと、台詞のトークンがそこで切れる）。だから組み立てからは「行末はここ（無いこともある）」をいつも明示して渡す必要がある。要件 8.3 の「指定しなければ最後の見える字が行末」は、直接呼ぶときの既定として別に持つことになる。

### 2.10 マニュアルとスキル

- マニュアル: `book/src/reference/pasta-toml.md`（`[talk]` の表 571〜614 行目、アクターの表の `script_wait_*` の行 180 行目、冒頭の例 103〜107 行目）、`book/src/lua/modules/pasta-sakura-script.md`（ウェイトの表・動作仕様）、`book/src/internals/talk-output.md`（「さくらスクリプトの組立」「同じ話し手の行の改行」「さくらスクリプトの後処理」）。
- スキル: `pasta-ghost-authoring/references/pasta-toml.md`・`sakura-script.md`、`pasta-lua-coding/references/pasta-sakura-script.md`・`internal-modules.md` は、1 行目が `GENERATED FROM PASTA MANUAL` の生成物で、`book/tools/gen-skill-refs.mjs` で作り直す。`pasta-ghost-authoring/SKILL.md`（手書き）はウェイトのタグの表と `[talk]` の行を持つ。
- `.claude/skills` への書き込みは、この環境では分類器に止められることがある（開発者の許可が要る）。実装のタスクで、生成ファイルの作り直しを開発者の許可の要る段として扱う。

## 3. 要件と資産の対応

| 要件 | 使える資産 | ギャップ | 種別 |
|---|---|---|---|
| 1.1〜1.4 行末のキー 5 つ・既定＝行の途中の値・`値−50` | `TalkConfig`・`WaitValues`・`flush_pending`・`append_with_wait` | 行末の値（書いたかどうかの区別）を持つ型。行末の並びで行末の値を使う分岐 | Missing |
| 1.5 縦書き・横書きで同じ | 変換はバルーンの向きを知らない | 無し | — |
| 2.1〜2.3 アクターごと・決まる順・毎回読み直す | `resolve_wait_values`（毎回読む） | 行末のキー 5 つの読み出し。決まる順（OQ2） | Missing |
| 2.4 行を話すアクター | 1 回の変換＝1 つのまとまりのアクター | 無し | — |
| 3.1〜3.3 アクション行・継続行の終わり | `act:line()` は両方の頭に出る。`seams` | 継ぎ目が `line_newlines` のときだけ付く。行末の位置が変換に届かない | Missing・Constraint |
| 3.4〜3.6 タグだけが続く行・見える字の無い行・単語の展開 | `read_unit`・`unit_shows_text`・`has_text` | 継ぎ目の前の最後の見える字を求める処理。トークンをまたぐ先読み | Missing |
| 3.7 行の印の無い台詞 | `merge_consecutive_talks`（継ぎ目が無ければつながる） | まとまりの最後の見える字を求める | Missing（小） |
| 3.8 `line_newlines` によらない | — | 継ぎ目を付ける条件の変更（2.5） | Constraint |
| 3.9 Call | 呼ばれた行の `act:line()` | Lua だけで積む先の扱い（2.6） | Unknown |
| 4.1 並びの最大の行末の値 | `pending_max_wait` | 行末の並びで種類ごとに行末の値を引く | Missing |
| 4.2・4.3 ふつうの字・リーダーの行末 | `append_with_wait` | 行末の 1 字だけ行の途中の待ちを入れず行末の値を入れる | Missing |
| 4.4 `す」` | 今は `す\_w[40]」` | 行末のキーが効くときだけ位置を動かす分岐（OQ4） | Missing |
| 4.5 タグの前 | `flush_pending` の位置 | 無し（今のまま） | — |
| 4.6 行末禁則 | `LineEndProhibited` は待ち無し | 無し。`\\`・囲みで終わる行も同じ扱いにするかを決める | Unknown（小） |
| 4.7 効かない行末は今と同じ | — | 効くかどうかの判定（並びの字の種類のどれかに行末のキーがあるか） | Missing |
| 5.1〜5.4 作者の `\_w` で置き換える | 単語の展開は同じテキスト | 辞書に直接書いた `\_w` は別トークン → 組み立ての先読み（2.3） | Missing（主要） |
| 5.5・5.6 置き換えの範囲 | 継ぎ目の有無、`wait` のトークンは変換を通らない | 「すぐ後ろ」の判定（間にタグがあれば足す） | Missing |
| 6.1 バイト単位で同じ | 既存のテスト群（`tests/sakura_script/`・`line_newlines_e2e_test.rs`・`lua_specs`） | 行末のキーが無いときに新しい分岐を一切通らないこと（特に継ぎ目をいつも付ける場合の `join_lines` の早期の戻り） | Constraint |
| 6.2 改行の位置と幅 | `join_lines`・`break_lines_impl` はタグを幅 0 で前の字に付ける | 4.4 で待ちが「」」の後ろへ動いても budoux の改行が変わらないか | Research Needed |
| 7.1〜7.2 書き間違いのエラー | `actor_font::apply` の前例、pasta_check の写し | `[talk]`・`[actor.*]` の行末のキーの検査（pasta.dll と pasta_check の両方） | Missing |
| 7.3 Lua の型違い | `get::<i64>().ok()` | 同じ読み方を使うだけ。小数の扱いは要確認（2.7） | Research Needed |
| 8.1・8.2 ノベル向けの出力 | まだ無い | 今は約束だけ（2.8） | Constraint |
| 8.3・8.4 直接呼ぶとき | `talk_to_script(actor, text, column)` | 4 つ目の引数（または表）で行末を渡す口。省いたときの既定 | Missing |
| 9.1〜9.6 マニュアル・スキル | 2.10 の章と `gen-skill-refs.mjs` | 書き足し。生成ファイルは許可の要る書き込み | Missing・Constraint |
| 10.1〜10.3 検証 | `line_newlines_e2e_test.rs`（DSL→出力の結合テストの作り）、`wait_inserter.rs` の単体テスト、x86・x64 の CI | 特性化テストを先に置く | Missing |

## 4. 実装の案

### 案 A: Rust だけで行末を決める（継ぎ目の位置を渡す）

- 組み立ては、継ぎ目（行の頭の位置）をそのまま `talk_to_script` に渡し、Rust がトークンの位置を数えて「継ぎ目の前の最後の見える字」を求める。作者の `\_w` が別トークンのときは、組み立てが「後ろに作者の `\_w` が来る」印を 1 つ渡す。
- 手を入れる所: `mod.rs`（引数）、`tokenizer.rs`（位置）、`wait_inserter.rs`、`act.lua`（継ぎ目の条件）、`sakura_builder.lua`（継ぎ目と印を渡す）。
- ✅ 待ちの規則が Rust の 1 か所にまとまる。Lua の変更が小さい。
- ❌ 見える字の定義が Lua（`read_unit`）と Rust に分かれる（2.4）。`join_lines` が改行を差し込んだ後のテキストでは継ぎ目の位置がずれるので、差し込んだ後の位置に直して渡す処理が要る。トークンをまたぐ行末（台詞の後ろに辞書で直接書いた `\s[5]` のトークンが続き、その後ろに継ぎ目がある）は、結局 Lua の先読みが要る。

### 案 B: Lua だけで行末を扱う（変換の出力を後から直す）

- 組み立てが、変換の出力の行末の待ちを書き換える（`\_w[…]` を探して値を直す・消す）。
- ✅ Rust を触らない。
- ❌ 出力から「どの `\_w` が自動の待ちか」を見分けられない（作者の `\_w`・単語の展開の `\_w` と区別できない）。ふつうの字・リーダーの行末（要件 4.2・4.3）や `す」`（4.4）の位置の移動は、字の種類の分類（Rust の `chars_*`）を Lua に写さないとできない。規則が 2 か所に分かれる。推奨しない。

### 案 C: 両方（行末の位置は Lua が求め、待ちの規則は Rust が持つ）

- Lua（`sakura_builder.lua`）: まとまりのトークンを先読みし、`read_unit` と同じ見える字の判定で、各テキストの中の「行末の最後の見える字の直後の位置」（＝ `join_lines` の後のテキストでのバイト位置）と、「テキストの終わりが行末で、次のトークンが継ぎ目なしの `\_w[…]` で始まる sakura_script か」を求め、`talk_to_script` の 4 つ目の引数（例: `{ ends = {…}, author_wait = true }`。名前は設計で決める）で渡す。
- Rust: `talk_to_script` が 4 つ目の引数を読み、トークンに位置を付けて、行末の並び（要件の用語の「行末の並び」）に行末の値を使う。テキストの中で行末の並びの直後に `\_w[` の `SakuraScript` トークンがあれば（単語の展開の道）、または `author_wait` の印があれば（辞書に直接書いた道）、行末のキーが効く行末だけ自動の待ちを入れない。4 つ目の引数を省いたら、テキストの最後の見える字を行末とする（要件 8.3）。
- 設定: `TalkConfig` に `Option<i64>` を 5 つ、`WaitValues` に行末の 5 値（書いたかどうかが分かる形）、`resolve_wait_values` に行末のキー 5 つ。`PastaConfig::parse` に行末のキーの型の検査（`actor_font::apply` と同じ形）、pasta_check に同じ検査。
- 継ぎ目: `ACT_IMPL.line` がいつも印を立てる（`seams` をいつも付ける）か、行末のための別の欄にするか（設計の判断 2）。`join_lines` は `line_newlines` が nil・0 なら今のまま読まずに返す。
- ✅ 見える字の定義が Lua の 1 か所のまま。待ちの規則が Rust の 1 か所のまま。トークンをまたぐ行末と、辞書に直接書いた `\_w` の両方を扱える。行末のキーが無いときは Rust の新しい分岐が値を変えない（行末の値＝行の途中の値・位置も動かない）ので、要件 6.1 を保ちやすい。
- ❌ Lua と Rust の間の受け渡し（位置の意味・`join_lines` の後の位置）を正確に決める必要がある。`BUILDER.build` は既に複雑度 28 なので、先読みは別の関数に出す。

### 比べ

| 観点 | 案 A | 案 B | 案 C |
|---|---|---|---|
| 見える字の定義 | 2 か所 | 1 か所（Lua） | 1 か所（Lua） |
| 待ちの規則 | 1 か所（Rust） | 2 か所 | 1 か所（Rust） |
| 辞書に直接書いた `\_w` | 印が要る | 出力の書き換え | 印で扱う |
| `join_lines` 後の位置 | 直す処理が要る | 関係なし | Lua が後の位置で求める |
| 要件 6.1 の保ちやすさ | 中 | 低 | 高 |

## 5. 手間と危険

- 手間: **M（3〜7 日）**。新しい技術は無く、規則と設定は今のファイルを広げるだけだが、Lua と Rust の受け渡し・特性化テスト・pasta_check の写し・マニュアル 3 章とスキルの生成が要る。
- 危険: **中**。規則そのものは単純だが、トークンをまたぐ先読みと位置の受け渡しで、行末のキーが無いゴーストの出力をバイト単位で保つ必要がある。特性化テスト（DSL → `SHIORI.request` の出力。`line_newlines_e2e_test.rs` の作り）を先に置けば下げられる。

## 6. 設計の判断の項目（要件ディスカッションへ）

1. **行末の位置をどこで求めるか**: 案 C（Lua が見える字の判定で求め、位置を渡す）か、案 A（Rust が継ぎ目から求める）か。コードの事実: 見える字の正は `sakura_builder.lua` の `read_unit`・`unit_shows_text`。`join_lines` は改行を差し込むので位置がずれる。
2. **継ぎ目を付ける条件**: いつも付ける（`actor-line-newlines` の「既定ではトークンの表が今と同じ」を改め、`line_newlines_test.lua` の 4 テストを直す）か、行末のための別の欄にするか。行末のキーの有無で分ける案は、Lua の後からの書き換え（要件 2.3）を追えない。トークンの表は出力ではないが、Lua のコードから見える（`act:build()` の戻り値。`novel-talk-flow` の仮の設計は `STORE.play.wait` にトークン列を控える）。
3. **辞書に直接書いた `\_w` の置き換え**（要件 5.1・5.2）: 別トークンなので、組み立ての先読みで「テキストの終わりが行末で、次が継ぎ目なしの `\_w[…]` で始まる sakura_script」を知らせる形にする。`\_w[500]\s[5]`（1 つのタグの後ろに別のタグ）や、`\_w` の後ろの sakura_script が `\_w` 以外のときを要件の例に足すか。
4. **`talk_to_script` の口**（要件 8.3・OPEN QUESTION 10）: 4 つ目の引数を足す（表で渡す）か、3 つ目の列の状態の表に欄を足すか。コードの事実: 直接の呼び出しは pasta にも ghost_dev にも無い（組み立ての 1 か所だけ）。組み立てからは、テキストの終わりが行末でないこともあるので、いつも明示して渡す。直接呼んで省いたときの既定（最後の見える字＝行末）は、行末のキーが無ければ今の出力と同じ。
5. **Lua と Rust で見える字の扱いが違う所**（2.4）: 行末が `\\` のエスケープや囲み `\_?…\_?` のとき、待ちを入れない（要件 4.6 と同じ扱い）でよいか。
6. **行末の後ろに続くトークンの読み飛ばし**（要件 3.4）: 同じまとまりの `surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`・`raw_script` のどれを「見える字を出さないもの」として読み飛ばすか（`choice` は字を出す）。
7. **決まる順**（要件 2.2・OPEN QUESTION 2）: `resolve_wait_values` は 1 キーずつ「アクター → 既定」を引くだけなので、どちらの順でも実装の手間は同じ。コードからは決まらない（作者の分かりやすさで決める）。
8. **`す」` の位置**（要件 4.4・OPEN QUESTION 4）: 今の出力は `す\_w[40]」`（「」」の後ろは待ち無し）。値だけを行末の値にする案だと `す\_w[1200]」` になり、閉じかっこが長い待ちの後に出る。位置を動かす案は、行末のキーが効くときだけ動く。
9. **書き間違いの検査を置く所**（要件 7・OPEN QUESTION 8）: `PastaConfig::parse` で `actor_font::apply` と同じ形にし、pasta_check の `validate/config.rs` にも写す。小数 `1200.0` をエラーにすると、アクターの表を Lua で読む側（mlua の `get::<i64>`）が `1200.0` を受け入れるなら、`pasta.toml` と Lua で扱いが食い違う（要確認）。
10. **Call の先が Lua で継ぎ目なしに台詞を積むとき**（要件 3.9）: 前の行の台詞とつながり、行末が分からない。境界の例として要件か設計に書くか。
11. **ノベル向けの出力**（要件 8.1・8.2・OPEN QUESTION 9）: 今は対象の出力が無い。`novel-talk-flow` の仮の設計は `\x` を 1 本の末尾に付けるので、行末の待ちは自然に前に来る。`novel-areka-contract` は「高速再生でも同じスクリプトを返す」に改まっている。要件 8.1 の「自動の待ちを入れない出力」の文言を今の取り決めに合わせて見直すか。
12. **スキルの生成ファイルの書き込み**（要件 9.5）: `.claude/skills/*/references/` の作り直しは、開発者の許可が要る段として、タスクに分けておく。

## 7. 設計で調べること（Research Needed）

- mlua 0.12（LuaJIT）で、`Table::get::<i64>` が整数の値の小数（`1200.0`）と数字の文字列（`"1200"`）を整数として読むか（判断の項目 9）。
- budoux の改行（`line_breaker.rs` の `break_lines_impl`）が、待ちの位置が「」」の前から後ろへ動いたとき（要件 4.4）と、ふつうの字の行末で行の途中の待ちが消えたとき（4.2）に、改行の位置と幅を変えないか（要件 6.2）。タグは幅 0 で前の字に付いて運ばれるので変わらない見込みだが、テストで確かめる。
- `append_with_wait` の「単独の `\` の後ろには入れない」規則（`\\` に化けるのを防ぐ）を、行末の待ちにも同じく当てること（行末が単独の `\` の行）。
- `STYLE.through`（装い）と `APPEARANCE.observe` が、変換の出力の待ちの数・位置の変化で振る舞いを変えないか（装いはタグを差し込むだけの見込み）。

## 8. 要件ディスカッションでの扱い（2026-11-26）

- 設計フェーズへ回す（6 章の番号）: 1（行末の位置をどこで求めるか）・2（継ぎ目を付ける条件）・3（辞書に直接書いた `\_w` の先読み）・4（`talk_to_script` の口）・6（行末の後ろのトークンの読み飛ばし）・9（検査を置く所と mlua の読み方）・12（スキルの生成ファイルの書き込みを許可の要る段にする）と、7 章の調べもの。
- 要件に入れて閉じた: 5（エスケープ・囲みで終わる行は待ちを入れない → 4.6）・10（Call の先が Lua で積む台詞は前の行の続き → 3.9）・11（ノベル向けの出力の文言を今の取り決めに合わせた → 8.1）・3 の `\_w[500]\s[5]` の例（→ 5.1）・pasta_check の検査（→ 7.4）。
- 議題にした: 7（決まる順）・8（`す」` の位置）と、要件の OPEN QUESTIONS 1・3・6・7。
