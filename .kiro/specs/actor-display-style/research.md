# ギャップ分析: actor-display-style

- **対象**: `requirements.md`（2026-10-18 生成・未承認）と、main `06c2805a` 時点のコード
- **分析の範囲**: 出力の組み立て（Lua）・アクター設定の読み込み（Rust／Lua）・`pasta_check validate`・テスト・マニュアル・スキル references
- **要件ディスカッション（2026-10-18）の後**: 本書の OQ-1〜8 は requirements.md の「確定事項」で決着した。OQ-2〜5・7・8 は本書の仮定どおり。OQ-1 は対象外のまま。OQ-6 は要件を見え方で書き直し、タグを出す時点は下の Research Needed の実機確認で設計が決める（要件 2.10）。本書の要件番号は確定前のもの（旧 2.2 は 2.2・2.3 に分け、旧 2.5 は 2.2 に吸収）。
- **参照した外部資料**: ukadoc（`\f[…]` の各属性、`\f[default]`、`\x`、`\x[noclear]`、`\c`、`\n[パーセント]`）

## 要約

- **足りないのは装いだけ**。同じスポットで話者が替わったときの `\p[spot]` の出し直し、`spot_newlines` の改行、立ち絵の復旧は、すでにある。テストも 1 本ある（`sakura_builder_test.lua` 910〜939 行「同一スポット共有の2アクター交代」）。
- **差し込む場所は 1 か所に決まる**。`emit_actor_switch`（`sakura_builder.lua` 70〜83 行）が `\p[spot]` と復旧タグを出す。その直後に装いのタグを足せば、改行（字の直前に出る）より前に並ぶ。装いのタグは内側のトークンの経路を通らないので、字の判定と改行の保留に影響しない。要件 2.4 の出力の順序が、そのまま得られる。
- **アクターの値は TOML の表そのまま**。`[actor."X"].font` は、Lua の `actor.font` として builder に届く。読み込みの経路は変えなくてよい。`act.lua` も触らずに済む見込み（brief の制約を満たす）。
- **書き間違いを検査する仕組みは無い**。`[actor]` には Rust の型が無く（`custom_fields` として素通り）、`pasta_check validate` も `[actor]` を読まない。要件 7（読み込みエラー＋validate）を満たすには、検査の規則を新しく作る必要がある。先例は `[scene.alias]` で、規則を `pasta_core`（`SceneAliasTable::from_entries`）に置き、pasta.dll と pasta_check の両方から使っている。
- **最大の不確実性は SSP の振る舞い**（OQ-6）。文字装飾をスコープごとに保つのか、スコープをまたいで保つのか。`\c`・選択肢・新しいスクリプトの始めで既定へ戻るのか。ukadoc の `\x` の例（`\1…\f[bold,1]…\x` で「スコープは本体側に戻り、フォント変更は解除」）からは、どちらとも読める。設計の前に、SSP 2.9.18 で確かめる必要がある。

## 1. 今の状態

### 1.1 出力の組み立て（`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`・383 行）

- `BUILDER.build(grouped_tokens, config, actor_spots, appearance)` が唯一の入り口である。呼び出し元は `pasta/shiori/act.lua` の 72 行。`config` には `spot_newlines`・`choice_newlines` を渡している（`[ghost]` から読む。49 行）。
- 1 回のビルドの中だけの状態は `last_actor`・`last_spot`・`spot_has_text`・`pending_break`・`spot_tail` の 5 つ（277〜288 行）。`last_actor` は毎回 `nil` から始まる。そのため、各ビルドで最初に話すグループでは必ず `emit_actor_switch` が呼ばれる（要件 2.8「組み立ての始めは装い無し」と相性がよい）。
- 話者の交代は、表が同じかどうかで決める（`last_actor ~= actor`、310 行）。同じスポットでも、別のアクターなら `\p[spot]` を出し直す。要件 2.4 の出力の例の 2 つ目の `\p[0]` はここから来る。
- `emit_actor_switch` が出すのは `\p[spot]` と `APPEARANCE.restore(...)` の結果だけ。改行の判定・出力・保留は、呼び出し側のループが持つ（60〜63 行のコメント）。
- 段落区切りの改行（`\n[spot_newlines×100]`）は、字を出すトークンの直前に出る（345〜347 行）。切り替えで出したタグは、その前に並ぶ。
- `clear`（`\c`）と台詞に書いた `\c` は `pending_break`・`spot_has_text`・`spot_tail` を戻す。ただし `last_actor`・`last_spot` は戻さない（324〜334・361〜370 行）。だから `\c` の後に同じアクターが続いても、切り替えは起きない。要件 3.2（`\c` の後に出し直さない）は、今の流れのままで満たせる。
- `choice` は区切りを出して `spot_tail` を変えるだけ（335〜340 行）。要件 3.3 も今のままで満たせる。
- `\x` を数える仕組みは無い。`text_parts` / `clear_span` が数えるのは、引数の無い `\c` だけである（`\x`・`\C`・`\c[…]` は数えない。talk-output.md 185 行）。要件 3.1（`\x` の後の出し直し）は新しい判定になる。
- 複雑度: `BUILDER.build` は luacheck の複雑度 28 を `ponytail:` の注記つきで許している（265 行）。ループの中に分岐を足すと、さらに上がる。

### 1.2 外見の観測と復旧（`pasta/shiori/appearance.lua`・567 行）

- `APPEARANCE.restore` は、同じアクターが同じスポットで続くときは何も返さない（`owners[spot]`・`last_spots[name]`、554〜558 行）。この状態は `STORE.appearance` に入り、ビルドをまたいで残る。
- 装いの状態は、ビルドをまたいで残さない（要件 2.8）。そのため、`appearance` に相乗りさせず、ビルドの中だけの状態として持つほうが筋がよい。
- シェルの無いアクター（`surface`・`dressup` を書かず、`\s` も出さない）の切り替えでは、`restore_surface` が空を返す（377〜391 行）。ぱすたへ戻るときも、スポットの表示中サーフェスがぱすたの記録と一致すれば空を返す。要件 6.2・6.3 は、今のコードで満たせる見込み。足りないのはテストとマニュアルだけ。
- `APPEARANCE.observe` が分類するのはサーフェス・bind・スコープ切替だけで、`\f` は無視する。装いを足しても、外見の記録は変わらない。

### 1.3 アクター設定の読み込み

- Rust の `inject_actor_names`（`crates/pasta_lua/src/runtime/module_registry.rs` 76〜106 行）が、`[actor."X"]` の各表に `name` を入れる。
- `store.lua` 111〜121 行が `STORE.actors = CONFIG.actor` とし、`actor.lua` 281〜289 行がメタテーブルを付ける。builder に届く `actor` は、TOML の表そのものである。
- 今読まれているキー: `name`・`spot`（store.lua、mouse.lua）・`surface`・`dressup`（appearance.lua）・`budoux`・`script_wait_*`（Rust `sakura_script/mod.rs`）。`font`（または OQ-2 で決まる名前）は、どこからも読まれていない。
- 登録していないアクターは、`act:actor_proxy` がその場で `{ name = name }` を作る（`pasta/shiori/act.lua` 532〜533 行）。この表には装いのキーが無いので、要件 2.9 は自然に満たせる。
- 副作用: `PROXY_IMPL.find_actor_handler` は `self.actor[key]` を最優先で返す（`actor.lua` 132〜137 行）。表の値のキー（`font`）は、そのアクターの会話行の `＠font` で `tostring(table)`（`table: 0x…`）になる。`dressup` にも同じ問題がある。要件 8.5 でマニュアルに載せる。キー名を決めるとき（OQ-2）に、この点も考える。

### 1.4 設定の検査

- `PastaConfig::parse`（`crates/pasta_lua/src/loader/config/mod.rs` 69〜103 行）が型を付けて読むのは、`[loader]` と `[scene.alias]` だけである。そのほかは `custom_fields` として型無しで素通りする。
- 型が合わないときの扱い（pasta-toml.md 30〜43 行の表）:
  - `[loader]`・`[scene]` は読み込みエラー。
  - `[talk]` など一部の表は、表ごと黙って既定へ戻る。
  - `[actor]`・`[ghost]` は、この表に載っていない（検査されない）。
- `pasta_check validate` の `crates/pasta_check/src/validate/config.rs`（288 行）が読むのは、TOML の構文・`[loader]` の型・`[scene.alias]` だけである。`[actor]` は読まない。報告文は `validate/report.rs` 331〜365 行にある。
- `[scene.alias]` の先例: 規則を `pasta_core::SceneAliasTable::from_entries` に 1 つ置き、pasta.dll（読み込みエラー）と pasta_check（エラーの報告）の両方が使う。行番号は、それぞれが `toml::Spanned` から出す。
- `pasta-check-validate-edges` は main に入っている（`5f260790`）。roadmap の「validate に報告させるなら、その後に実装する」の条件は、もう満たしている。

### 1.5 SSP の `\f`（ukadoc）

- 字の見た目を変える属性: `color`・`name`・`height`・`bold`・`italic`・`underline`・`strike`・`sub`・`sup`・`outline`・`shadowcolor`・`shadowstyle`。どれも値 `default` で、その属性だけをバルーンの既定へ戻せる。
- 行の寄せ: `align`（次の意図的な改行まで効く）・`valign`。選択肢とアンカーの装飾: `cursor*`・`anchor*`。全体の操作: `\f[default]`（全属性を既定へ）・`\f[disable]`。
- `\x` はクリック後に「スコープを `\0` に戻し、`\f` 系の効果を解除する」。`\x[noclear]` は効果を残す。
- `\c` の説明は「字を消す」だけで、`\f` については書いていない。
- スコープの切り替え（`\p[N]`・`\0`・`\1`）で `\f` の状態がスコープごとに保たれるかは、ukadoc に書いていない（**Research Needed**）。
- `\n[パーセント]` は「通常の」行の高さに対する割合である。装いで `height` を変えたとき、`spot_newlines` の改行の幅が変わるかは未確認（**Research Needed**。改行は装いのタグの後に出る）。

### 1.6 テスト

- builder の単体テスト: `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua`（3,071 行）。アクターは `{ name = "さくら", spot = 0 }` のような素の表で、グループ化トークンを手で組む。`result:find(...)`・`count(...)`・`toBe` で確かめる。装いのテストは、アクターの表に `font = {...}` を足すだけで書ける。
- 外見の復旧: `lua_specs/appearance_test.lua`（937 行）・`store_appearance_test.lua`。
- 既定の出力のバイト列を固定したテスト: `crates/pasta_shiori/tests/byte_invariant_test.rs`（`assert_golden_bytes`・`GOLDEN_ONBOOT`）・`kick_unused_byte_invariant_test.rs`。要件 5.3（出力を 1 文字も変えない）の関門になる。
- hello-pasta: `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml` のアクターは `spot` だけ（女の子=0、男の子=1）。テストは `crates/pasta_sample_ghost/tests/`。
- Rust の結合テストの土台: `crates/pasta_shiori/tests/fixtures/*/pasta.toml`。
- pasta_check: `validate/config.rs` 内の単体テスト（236 行〜）と、`crates/pasta_check/tests/`。

### 1.7 マニュアルとスキル references

- `reference/pasta-toml.md`: `[actor."名前"]` のキーの表（393〜399 行）・surface/dressup の節（442〜460 行）・型が合わないときの表（30〜43 行）。
- `grammar/actor-dictionary.md`: 「バルーン連携」（102〜145 行）・「同一スポット共有時の外見の復旧」（147〜231 行）。共有の形は、外見の面ではもう載っている。足りないのは、シェルの無い相手役の例と装い。
- `grammar/sakura-script.md`: よく使うタグの表（118〜133 行）に `\f` は無い。「ここに無いタグも透過する」とだけ書いてある。
- `reference/shell.md`: 66〜76 行の「吹き出しとアクターの spot」は、descript.txt のバルーン位置の話だけ。
- `internals/talk-output.md`: `emit_actor_switch` の手順（153〜160 行）と、区切りの表（164〜185 行）がある。装いの段を足す場所は明確である。
- book/src のどこにも `\f` の説明は無い。
- スキル references: `book/tools/gen-skill-refs.mjs`（40〜63 行に章とスキルの対応）。
  - 再生成の対象になるのは `pasta-toml.md`・`actor-dictionary.md`・`sakura-script.md`。
  - `shell.md`・`talk-output.md` は対象外。
  - CI の `.github/workflows/manual.yml` 80〜81 行が `--check` で食い違いを検査する。
  - 手順は `book/AUTHORING.md` 163〜170 行と、steering の `workflow.md`（Manual Sync Gate）にある。

## 2. 要件と資産の対応

| 要件 | 使える資産 | ギャップ |
| ---- | ---------- | -------- |
| 1.1〜1.4 装いを書く | アクターの表は TOML のまま届く | **Missing**: 装いの表を読んで `\f` のタグ列に直す処理。キーの名前と形は **Unknown**（OQ-2〜4） |
| 1.5 ほかのキーを変えない | 読み込みの経路は変えずに済む | Constraint: `＠font` の副作用（1.3） |
| 2.1 切り替えで出す | `emit_actor_switch` の直後が差し込み口 | **Missing**: 装いのタグを出す段 |
| 2.2・2.5・2.6 既定へ戻す | — | **Missing**: 「どのバルーンに、誰の装いが付いているか」の、ビルドの中だけの状態。**Unknown**: SSP がスコープごとに保つか（OQ-6）。戻し方（`\f[default]` か属性ごとか。OQ-5） |
| 2.3 続けて話すときは出さない | `last_actor` の判定がそのまま使える | なし |
| 2.4 出力の順序 | 改行は字の直前に出る | なし（差し込み口で決まる） |
| 2.7 字として数えない | 切り替えのタグは内側のトークンを通らない | なし |
| 2.8 組み立ての始めは装い無し | `last_actor` は毎回 `nil` から始まる | 状態をビルドの中に置けば満たせる。**Unknown**: SSP がスクリプトの始めで戻すか |
| 2.9 未登録アクター | その場の表に装いのキーが無い | なし |
| 3.1 `\x` の後の出し直し | `APPEARANCE.tag_at`・`clear_span` の読み方 | **Missing**: 引数の無い `\x` を見つける判定と、出し直しの段（ループの中の分岐が増える）。OQ-7 |
| 3.2・3.3 `\c`・選択肢の後 | 今の流れは切り替えを起こさない | **Unknown**: SSP の実際の振る舞い（OQ-6） |
| 4.1〜4.3 台詞の `\f` | 台詞のタグは素通り（Rust の tokenizer も透過） | なし（装いは切り替えのときだけ出すので、台詞の中には差し込まない） |
| 5.1〜5.3 出力を変えない | バイト列の固定テスト・hello-pasta | Constraint: 装いの無いアクターしかいなければ、新しい段は空を返すこと |
| 6.1〜6.3 共有の正式な支え | 改行・復旧は実装済み・テスト 1 本 | **Missing**: シェルのあるアクター 1 人＋シェルの無いアクター 2 人以上のテスト（改行・復旧・装いを 1 本で） |
| 7.1〜7.4 読み込みエラー | `[scene.alias]` の先例（pasta_core に規則・`Spanned` で行番号） | **Missing**: `[actor]` の装いの検査（Rust）。`custom_fields` から `[actor.*.font]` を取り出して検査する段 |
| 7.5 validate | `validate/config.rs`・`report.rs` | **Missing**: 新しい finding の種類と報告文。`release` の検査の段は validate を通すので、自動で付いてくる |
| 7.6 値の中身は検査しない | — | なし |
| 8.1〜8.8 マニュアル | 共有の外見の節がある | **Missing**: 装いの節・シェルの無い相手役の例・`\f` と装いの関係・talk-output の段の追記。references の再生成 |

## 3. 実装の選択肢

### 選択肢 A: `sakura_builder.lua` を広げる

- `emit_actor_switch` の中（または直後）で、`actor.font` からタグ列を作って出す。
- 「バルーンごとに付いている装い」をローカルの表（`spot_style` のようなもの）で持つ。
- `\x` の判定は `text_parts` の隣に足す。
- 検査は Rust の `PastaConfig::parse` と pasta_check に足す。
- ✅ ファイルが増えない。差し込み口が 1 か所で済む。
- ❌ `BUILDER.build` の複雑度（今 28）がさらに上がる。`\x` の出し直しは、ループの中に分岐を足すことになる。
- ❌ 同じファイルを、`vertical-balloon-lines`（`choice_break`）も触る。

### 選択肢 B: 装いの専用モジュールを作る（`pasta/shiori/actor_style.lua` のようなもの）

- `appearance.lua` と同じ形にする。モジュールは状態を持たず、状態の表を引数で受け取る。
- 提供するのは次の 3 つ:
  - `STYLE.new()`
  - `STYLE.on_switch(state, actor, spot)`: 戻すタグ＋装いのタグを返す
  - `STYLE.observe(state, text)`: `\x` を見て、出し直しの印を立てる
- builder に足すのは呼び出し数行だけ。
- 装いの表をタグ列に直す処理（値の文字列化）も、このモジュールに置く。
- 検査の規則は Rust（`pasta_core`）に 1 つ置き、読み込みと validate の両方で使う（`[scene.alias]` と同じ形）。
- ✅ builder の複雑度をほとんど上げない。単体テストを独立に書ける。外見の復旧（appearance）と同じ型で読みやすい。
- ✅ `vertical-balloon-lines` と触る行が重なりにくい。
- ❌ ファイルが 1 つ（＋テスト 1 つ）増える。

### 選択肢 C: 混ぜる

- 出力（Lua）は B のとおり専用モジュールにする。
- 検査は、Lua の読み込みで行う。例: `store.lua` で `CONFIG.actor` を走査して `error()` で止める。pasta_check は Rust で同じ規則を別に持つ。
- ✅ Rust の読み込みに手を入れずに済む。
- ❌ 規則が Lua と Rust に二重になり、ずれやすい。Lua の `error()` では、TOML の行番号を出せない。読み込みエラーの出方が `[scene.alias]` とそろわない。

### 補足: 検査を読み込みエラーにしない場合（OQ-8 で「警告して無視」になったとき）

- Rust 側の検査は pasta_check だけで済む。読み込みでは、Lua 側が知らない属性を無視して log.warn を出す。
- 規則は依然として 2 か所（Lua の無視の判定と、Rust の報告）に必要になる。

## 4. 手間とリスク

- **手間: M（3〜7 日）**
  - 出力の段は小さい（S 相当）。
  - Rust の検査（`pasta_core` の規則・読み込みエラー・validate の finding と報告文・行番号）が中くらいの量になる。
  - マニュアル 5 章と references の再生成、共有のテストがそれに乗る。
- **リスク: 中**
  - 出力の差し込み口は明確で、既定の出力を変えない条件もテストで守られている。
  - 一方で、SSP の文字装飾の持ち方（スコープごとか・`\c`・選択肢・`\x`・スクリプトの始め）が未確認で、その結果しだいで要件 2.2・2.5・3.x の出力が変わる。
  - `vertical-balloon-lines` と `sakura_builder.lua` を共有する。

## 5. 設計フェーズへの申し送り

### 推す方向（決めるのは設計）

- 出力は選択肢 B（`appearance.lua` と同じ形の、状態を持たない専用モジュール＋ビルドの中だけの状態）。
- 検査は `[scene.alias]` と同じ形。規則を `pasta_core` に 1 つ置き、pasta.dll の読み込みと `pasta_check validate` の両方で使う。

### Research Needed（設計の前に SSP 2.9.18 の実機で確かめる。OQ-6・OQ-7）

> 設計フェーズ（2026-10-18）で 1〜3・5・6 を実機で確かめた。4（`\x`）は未確認。結果は本書末尾の「SSP 2.9.18 の実機確認」。2 は下の仮定と違い、`\c` で既定に戻る。

1. `\p[0]\f[color,red]あ\p[1]い\p[0]う\e`
   - 「い」が既定の色か。スコープごとに保つか、スコープをまたいで保つか。
   - 「う」が赤のままか。
2. `\p[0]\f[color,red]あ\cい\e` で、「い」が赤のままか（`\c` は装飾を保つか）。
3. 選択肢の後の字: `\p[0]\f[color,red]あ\n\![*]\q[選ぶ,X]\nい\e` で、「い」が赤のままか。あわせて、選択肢の表示の文字に `\f[color]` が効くか。
4. `\x` の後: `\p[0]\f[color,red]あ\xい\e`
   - 「い」が既定の色に戻るか。
   - スコープが `\0` に戻ることが、`\p[1]` で話しているアクターにどう効くか。
5. 前のスクリプトの装いが、次のスクリプト（次のトーク）に残らないか。
6. `\f[height,…]` の後の `\n[180]` の幅が、装いの文字の高さに依存するか。依存するなら、装いのタグと改行の順序を変える必要がある。

### 設計で決めること

- 装いの状態を「バルーンごと」に持つか「直前の話者」で持つか。1 の結果で決まる。
- 戻すタグの位置。いまの仮定は、切り替え先の `\p[N]` と復旧タグの後。
- `\x` の判定を `APPEARANCE.tag_at` で読むこと。囲みとエスケープの中は数えない。
- 値の文字列化の規則（OQ-4）と、`]`・`\` を含む値の拒否（要件 7.3）。

### 並走の条件

- `vertical-balloon-lines` と同じ `sakura_builder.lua` を触る。
- 選択肢 B なら、builder への変更は `emit_actor_switch` の周りの数行と、`\x` の観測の呼び出しに限られる。
- `pasta-toml.md` は両方が触る。後から入る側が rebase で合わせる。

---

# 設計フェーズの調査（2026-10-18・`/kiro-spec-design -y`）

## Summary

- **Feature**: `actor-display-style`
- **Discovery Scope**: Extension（既存の出力の組み立てと設定の検査への追加。light discovery＋SSP の実機確認）
- **Key Findings**:
  - SSP 2.9.18 の文字装飾は**スコープごと**に保たれ、スコープをまたがない。`\c` で既定に戻る。新しいスクリプトでも既定に戻る。選択肢の後は保たれる（下の実機確認 #1〜#5）。ギャップ分析の「`\c` の後は今の流れのままで満たせる」は誤りで、`\c` の後は出し直しが要る。
  - `toml` 1.1 は `preserve_order` を有効にしておらず、`toml::Table` は名前順になる。Lua の表も順序を持たない。要件 2.1 の「属性の順は書いた順」は、Rust で span から順を取り出さないと満たせない。`toml::de::DeTable::parse`（span つきの木）が公開 API にある。
  - 装いのタグ列を Rust（`pasta_core`）で作れば、規則（12 属性・値の検査・変換）を 1 か所に置ける。Lua は「いつ出すか」だけを持てばよい。

## 使ったスキルと指針

- `kiro-spec-design`（rules: design-principles・design-discovery-light・design-synthesis・design-review-gate）。テンプレート `.kiro/settings/templates/specs/design.md`・`research.md`。
- steering: `product.md`・`tech.md`・`structure.md`（`[scene.alias]` の規則を `pasta_core` に置く形、Lua の状態を持たないモジュールの形）。
- プロジェクトの記録（memory）: 「リファクタリングは安全かつ可逆に」（`BUILDER.build` の分岐を増やさない）、「`.claude/skills` の編集は分類器が拒否することがある」（references の再生成のリスク）、「マニュアルに回避レシピを載せない」、「cargo の前に NoDefaultCurrentDirectoryInExePath を外す」（`cargo tree` で実行）。

## Research Log

### SSP 2.9.18 の実機確認（Research Needed の 6 項目）

- **Context**: 要件 2.10。タグを出す時点を SSP の振る舞いに合わせる。
- **方法**: SSP MCP（`mcp__ssp__sakurascript` でスクリプトを送り、`mcp__ssp__dump_balloon` でスコープごとのバルーンの画像を取り、`mcp__ssp__get_status` で開いているバルーンを見る）。対象は起動中のゴースト「悪役令嬢クローディア」（横書きの既定バルーン）。`baseware.version` は `SSP/2.9.18 (20261017-5; …)`。`\_q` で一括表示し、`\x[noclear]` で表示を止めて画像を取った。最後に `\e` を送って閉じた。
- **結果**:

| # | 送ったスクリプト（要点） | 見えたもの | 結論 |
| - | ------------------------ | ---------- | ---- |
| 1 | `\p[0]\f[color,0,180,0]\f[height,24]あああ\p[1]いいいいい\p[0]ううう` | スコープ 0: 「あああううう」が緑・大。スコープ 1: 「いいいいい」は既定の色・大きさ | 装いはスコープごとに保たれ、別のスコープへ移らない。戻ると元の装いのまま |
| 1b | `\p[0]\f[color,0,180,0]あああ\p[0]ううう\p[0]\f[default]\n[180]えええ` | 「あああううう」が緑、「えええ」は既定で、1.8 行ぶん下 | `\p[0]` の出し直しでは保たれる。`\f[default]` で戻る。要件 2.5 の例の形がそのまま効く |
| 2 | `\p[0]\f[color,0,180,0]\f[height,24]あああ\cかきくけこ` | 「かきくけこ」は既定の色・大きさ | **`\c` で既定に戻る** |
| 2b | `\p[1]\f[…]かかか\p[0]あああ\cさしす\p[1]きくけ` | スコープ 1 の「かかかきくけ」は緑・大のまま | `\c` は自分のスコープだけを戻す |
| 3 | `\p[0]\f[color,0,180,0]\f[height,24]あああ\n\![*]\q[えらぶ,OnTestNothing]\nかきくけこ` | 「かきくけこ」は緑・大。選択肢「えらぶ」は大きさを受け継ぎ、色は選択肢の色（茶） | 選択肢の後も保たれる。選択肢の字は `height` を受け継ぐ |
| 4 | `\p[0]\f[…]あああ\xかきくけこ` | 設計フェーズではクリック待ちのまま。クリックを送る手段が無く、確かめられなかった。実装の後（タスク 5.1）に確かめた。結果は表の下の「実装の後の確認（タスク 5.1）」 | **確認済み**（下を参照） |
| 5 | 1 本目 `\p[1]\f[…]かかか\p[0]\f[…]あああ\e`、2 本目 `\p[0]さしすせそ\p[1]たちつてと` | 2 本目は両スコープとも既定 | 新しいスクリプトの始めで既定に戻る |
| 6 | `\f[height,30]あああ\f[default]\n[180]かかか` と `\f[height,30]あああ\n[180]\f[default]かかか` と `あああ\n[180]かかか`、および `あああ\f[height,30]\n[180]かかか` と `あああ\n[180]\f[height,30]かかか` | 行の送りの差は数ピクセル以内で、順の入れ替えで目に見える違いは無い（大きい字の行はその分だけ高い） | 横書きでは装いのタグと `\n[180]` の順は表示に効かない。縦書きもタスク 5.1 で確かめ、同じ（下を参照） |
| 7 | `\p[0]あ\p[1]\f[color,…]\f[default]\p[0]い` と、対照の `\p[0]あ\p[1]う\p[0]い` | 前者は `balloon(0=0)`、後者は `balloon(0=0/1=0)` | 字の無いスコープに `\f` だけを出しても、バルーンは開かない |

- **Implications**: 状態はスポットごと。別のスポットへ移るときに戻す必要は無い。同じスポットで装いのある話者が関わる交代では `\f[default]` で戻す。`\c` の後は出し直す。選択肢の後は何もしない。ビルドの最初の装いの前に `\f[default]` は要らない。`\x` は未確認のため、どちらの振る舞いでも表示が同じになるよう出し直す。

#### 実装の後の確認（タスク 5.1・2026-10-18）

- **方法**: 設計フェーズと同じ。SSP MCP の `sakurascript` で送り、`dump_balloon` で画像を取った。対象は起動中のゴースト「悪役令嬢クローディア」。`\x` のクリック待ちは、computer-use で吹き出しを 1 回クリックして進めた。縦書きは、バルーンを一時的に「窓際のぱすたさん縦書き」（`vertical,1`）に替えて確かめ、終わったら「クローディア」に戻した。

| # | 送ったスクリプト（要点） | 見えたもの | 結論 |
| - | ------------------------ | ---------- | ---- |
| 4a | `\_q\p[0]\f[color,200,0,0]\f[height,24]あああ\_q\xかきくけこ\x[noclear]` | クリックの後、「かきくけこ」は既定の色（黒）・大きさ | **`\x` で既定に戻る**（ukadoc のとおり） |
| 4b | `\_q\p[0]\f[color,0,150,0]ううう\p[1]\f[color,200,0,0]\f[height,24]いいい\_q\xかきくけこ\x[noclear]` | クリックの後、両スコープの吹き出しが消え、「かきくけこ」はスコープ 0 に既定の色・大きさで出た。スコープ 0 の緑も残らない | `\x` は全スコープを消し、スコープを `\0` に戻し、スコープ 0 の装いも既定に戻す |
| 6v | 縦書きのバルーンで `\_q\p[0]\f[height,30]あああ\p[0]\f[default]\n[180]かかか` と `\_q\p[0]\f[height,30]あああ\n[180]\f[default]かかか`、対照の `\_q\p[0]あああ\n[180]かかか` | 前の 2 つの画像は同じ（大きい字の列の分だけ、次の列が左へ寄る）。対照は普通の幅の 2 列 | 縦書きでも、`\f[default]` と `\n[180]` の順は表示に効かない |

- **Implications**: 出し方は変えない。`\x` の後に話し手の装いを出し直し、スポット 0 も装い付きとみなす今の実装（`STYLE.through`）は、4a・4b の振る舞いに合う（`\x` の後の字はスコープ 0 に既定の装いで出るので、出し直しが要る）。`\x` の後のスポット 1 以降の状態は Styled のまま残すため、次の切り替えで `\f[default]` が余分に出ることがある。表示は変わらない（要件 2.10 で許される）。縦書きでも 2.5 の順（戻すタグ → 改行）のままでよい。
- **注**: タスク 5.1 の文面は「このブランチの pasta.dll で動く確認用のゴーストを別の SSP（readonly・別ポート）で動かす」としていた。ただ、SSP MCP（`dump_balloon`）は別に立てた SSP には届かない。そこで、確かめる対象を SSP の振る舞い（#4・#6）に絞り、起動中の SSP にスクリプトを直接送った。pasta.dll が出すバイト列そのものは、出力の組み立てのテスト（`sakura_builder_test.lua`）と、4.1 のレビューで実際の `PastaShiori` を読み込んで出した共有スポットの例の一致で確かめてある。

### toml の表の順序

- **Context**: 要件 2.1「属性の順は書いた順」。
- **Sources**: `cargo tree -p pasta_lua -e features -i toml`（features は default・display・parse・serde・std。`preserve_order` 無し）、`crates/pasta_lua/src/runtime/module_registry.rs` の `toml_to_lua`（表を Lua の表に、整数を `f64` に直す）、`toml-1.1.7` のソース（`src/de/parser/detable.rs` に `pub type DeTable<'i> = Map<Spanned<DeString<'i>>, Spanned<DeValue<'i>>>` と `DeTable::parse`）。
- **Findings**: Rust でも Lua でも、型つきの表からは書いた順が取れない。span つきの木（`DeTable`）なら値の位置で並べ直せる。Lua の整数は `f64` になるので、大きな整数の 10 進の文字列化は Rust のほうが正確。
- **Implications**: タグ列は Rust で作る。`preserve_order` は有効にしない（ほかの表の並びに影響が出るため）。

### 検査の先例と報告の種類

- **Sources**: `pasta_core/src/registry/scene_alias.rs`、`pasta_lua/src/loader/config/mod.rs` の `parse_scene_aliases`、`pasta_check/src/validate/config.rs` の `read_aliases`、`report.rs` の `FindingKind`・`skip_reasons`、`book/src/debug/validate.md` の表、`tests/validate_parity_test.rs`（pasta.dll と pasta_check の読み込みの成否の一致）。
- **Findings**: 規則は `pasta_core`、取り出し（span と行番号）は各クレートが持つ。報告の種類を足すと `FindingKind::ALL`・`validate_report_test.rs`・`debug/validate.md` も変わる。`skip_reasons` に入る設定の誤りは、シーンの集まりに関わるものだけ。
- **Implications**: `config-actor-font` を足し、`skip_reasons` には入れない。一致テストに `font` の例を足す。

### 内部設計の章とモジュールの一覧

- **Sources**: `book/src/internals/talk-output.md`（`emit_actor_switch` の手順・モジュール表・境界の表）、`internals/shiori.md` 103 行（SHIORI 用のモジュール一覧）、`internals/index.md` 90 行、`lua/modules/pasta-config.md` 51 行（`name` の注入）、`book/tools/gen-skill-refs.mjs` の `GENERATION_MAP`（`pasta-config.md` は pasta-lua-coding の references の元）、`crates/pasta_lua/build.rs`（`pasta_scripts/` を丸ごと zip に入れる）。
- **Implications**: 新しい Lua モジュールは登録不要。マニュアルは要件 8 の 5 章に加え、`debug/validate.md`・`pasta-config.md`・内部設計の 2 章も直す。references は pasta-ghost-authoring の 3 つと pasta-lua-coding の `pasta-config.md` を再生成する。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
| ------ | ----------- | --------- | ------------------- | ----- |
| R1: Rust でタグ列を作り `@pasta_config` の `font` を置き換える | 検査と変換を `pasta_core` に置き、読み込みで `font` を文字列にする | 規則が 1 か所。書いた順を守れる。Lua が単純（文字列を出すだけ）。新しいモジュール・キーが要らない | `@pasta_config` の `font` が TOML の表と違う形になる（マニュアルで説明） | **採用**（OQ-1） |
| R2: Lua で表からタグ列を作る（固定の順） | Rust は検査だけ、Lua が 12 属性の一覧の順で並べる | Rust→Lua の受け渡しが要らない。`font` は表のまま | 要件 2.1 の「書いた順」を満たさない（要件の変更が要る）。12 属性の一覧が Rust と Lua の 2 か所 | 代案 |
| R3: Rust でタグ列を作り、別の内部モジュールで渡す | `@pasta_actor_font = { 名前 = タグ列 }` を登録 | `font` は表のまま。書いた順を守れる | 内部モジュールが 1 つ増える。ビルダーのテストに差し替えが要る | 代案 |
| B1: 出し入れを `sakura_builder.lua` に直接書く | ループの中に分岐を足す | ファイルが増えない | `BUILDER.build` の複雑度がさらに上がる。`vertical-balloon-lines` と衝突しやすい | 不採用 |
| B2: 状態を持たない専用モジュール | `appearance.lua` と同じ形 | ビルダーは 1 行ずつ。単体で試せる | ファイルが 1 つ増える | **採用** |

## Design Decisions

### Decision: タグ列は Rust（`pasta_core`）で作り、`@pasta_config` の `font` を置き換える

- **Context**: 要件 2.1（書いた順）・7（読み込みエラーと validate の両方）・確定事項 7（`[scene.alias]` と同じ形）。
- **Alternatives Considered**: R2・R3（上の表）。
- **Selected Approach**: `ActorFont::from_input` が検査とタグ列への変換を行う。pasta.dll は読み込みで `custom_fields` の `font` を検査済みのタグ列（文字列）に置き換え、Lua はそれを出すだけ。
- **Rationale**: 規則（12 属性・型・禁止文字・変換）が 1 か所に集まり、pasta.dll と pasta_check がずれない。書いた順を守れる。Lua の整数が `f64` になる問題も避けられる。
- **Trade-offs**: `@pasta_config` の `actor."名前".font` が TOML の表と違う形（文字列）で読める。`＠font` がそのタグ列に解決される（どちらにしても単語として使えない点は同じ）。
- **Follow-up**: マニュアル（`pasta-config.md`・`pasta-toml.md`・`actor-dictionary.md`）に形を書く。

### Decision: 装いの状態はスポットごと、ビルドの中だけ

- **Context**: 実機確認 #1・#2・#5、要件 2.8・5.2。
- **Selected Approach**: `used`（切り替えに使った）・`styled`（装いを付けた）をスポットごとに持ち、装いの無いアクターの `\c` で戻す。`any`（装いのあるアクターが出たか）が偽の間は何もしない。
- **Rationale**: SSP がスコープごとに保ち、`\c` と新しいスクリプトで戻すので、この 2 つのフラグで「戻す必要があるか」が決まる。装いの無いゴーストの出力は、`any` が偽のまま変わらない。
- **Trade-offs**: `\x` の後と、最上位の `raw_script` の中は保守的（冗長な `\f[default]` を出すことがある）。要件 2.10 で許される。

### Decision: `\c`・`\x` の後の出し直しは、変換後の文字列への挿入で行う

- **Context**: 要件 3.1・3.2。台詞の中の `\c`（「あ\cい」）の後ろの字も装いで出す必要がある。
- **Alternatives Considered**: トークンを `\c` で分けて変換する（`talk_to_script` の待ちの挿入が境目で変わりうる）、トークンの後にまとめて出す（同じトークンの中の `\c` の後ろの字に間に合わない）。
- **Selected Approach**: `STYLE.through` が、変換後の文字列の引数の無い `\c`・`\x` の直後にタグ列を挿入する。読み取りは `APPEARANCE.tag_at`。
- **Rationale**: 字の判定は変換前のテキストで済んでいるので影響しない。作者の書いた文字は消さず・入れ替えない（4.2）。装いの無いアクターでは挿入しない（5.1）。

### Decision: `config-actor-font` は飛び先の検査を省く理由に入れない

- **Context**: `skip_reasons` は「検査した辞書のシーンが pasta.dll と同じと言えない」ときの理由。
- **Selected Approach**: 装いの誤りは報告するが、飛び先の検査は続ける。
- **Rationale**: 装いはシーンの集まりに関わらない。報告はより多く出る方が作者に役立つ。

## Synthesis

- **Generalization**: 「設定の 1 区画を `pasta_core` の規則で検査し、pasta.dll は読み込みエラー、pasta_check は行ごとの報告にする」は `[scene.alias]` と同じ形。共通の仕組みにはしない（2 例しかなく、取り出し方も違う）。
- **Build vs Adopt**: SSP の `\f[default]` を使う（属性ごとに `default` で戻す方法は採らない。1 つのタグで済み、台詞の `\f` も戻せる）。toml の `DeTable` を使い、自前の span 計算は書かない。
- **Simplification**: 装いの状態は 2 つのフラグと 1 つの真偽値だけ。「どの装いが付いているか」は持たない（同じ装いのアクターが戻っても `\f[default]`＋装いを出す。表示は同じ）。12 属性の一覧は Rust だけに置く。

## OPEN QUESTIONS（設計）— 2026-10-18 の設計ディスカッションで全項目を推奨案で確定（design.md「設計の確定事項」）

### OQ-1: 装いのタグ列を Lua へ渡す形

- **仮定（設計の草案）**: pasta.dll の読み込みで、`@pasta_config` の `actor."名前".font` を検査済みのタグ列（文字列）に置き換える（R1）。
- **推奨**: R1。規則が `pasta_core` の 1 か所に集まり、要件 2.1 の「書いた順」を満たし、新しいモジュールもキーも増えない。
- **代案**: R2（Lua が表から固定の順で作る。要件 2.1 を「属性の順は Requirement 1.2 の一覧の順」に変える必要がある）、R3（内部モジュール `@pasta_actor_font` で渡す）。
- **例**:

```toml
[actor."おじさん"]
spot = 0
font = { bold = true, color = "90,110,140" }
```

```lua
-- R1: @pasta_config から読むと
config.actor["おじさん"].font  --> "\\f[bold,1]\\f[color,90,110,140]"
-- R2: 表のまま（出力は一覧の順で \f[color,90,110,140]\f[bold,1]）
config.actor["おじさん"].font.bold  --> true
```

### OQ-2: `\x` の後の SSP の振る舞い（未確認）

- **仮定**: ukadoc のとおり、`\x`（引数なし）はクリック後に `\f` を解除し、スコープを `\0` に戻す。装いのあるアクターの台詞の `\x` の直後に装いのタグを出し直す（スコープ 0 に効く）。状態はスポット 0 も「装いを付けた」にする。
- **推奨**: 草案のまま。解除されてもされなくても表示は同じ（解除されないなら冗長なタグになるだけ。2.10 で許される）。
- **代案**: `\x` の後に `\p[spot]` も出し直して、話者のスポットへ戻す（スポット 1 で話すアクターの `\x` の後の字が `\0` に出る、今の振る舞いを変えてしまう。本 spec の範囲を超える）。
- **実装時の確認手順**: SSP 2.9.18 で `\_q\p[0]\f[color,0,180,0]あああ\xかきくけこ\x[noclear]\e` を送り、クリックした後に `dump_balloon` で「かきくけこ」の色を見る。`\p[1]\f[color,0,180,0]あ\xい` も同じに見る。結果を本書に追記し、解除されないと分かっても出力は変えない。
- **例（出力）**: `おじさん：やあ\xまたね。` → `\p[0]\f[color,90,110,140]やあ\x\f[color,90,110,140]またね。\e`

### OQ-3: 要件 4.3 の「その組み立てで装いのあるアクターが話したとき」の読み方

- **仮定**: スポット単位で読む。切り替え先のスポットで、そのビルドで装いを付けた（`\c` で消していない）か、切り替え先のアクターが装いを持つときだけ戻す。
- **推奨**: 草案のまま。要件 5.2（「同じバルーンに装いのあるアクターの台詞が表示されていない」なら出力を変えない）と矛盾しない読み方はこれだけ。
- **代案**: ビルド単位で読む（装いのあるアクターがどこかで話したら、そのビルドのすべての切り替えで戻す）。5.2 の例で出力が変わる。
- **例**: おじさん（装い・スポット 0）→ ぱすた（スポット 1・台詞に `\f[bold,1]`）→ 男の子（スポット 1・装い無し）。草案: 男の子への切り替えでは何も出さない（ぱすたの太字が男の子に残る。今の動きのまま）。代案: `\p[1]\f[default]男の子の台詞`。

### OQ-4: 装いのあるアクターのグループで出した選択肢の字

- **仮定**: 何もしない。選択肢の字は装いの `height`・`name` などを受け継ぎ、色は選択肢の色になる（実機 #3）。
- **推奨**: 草案のまま。選択肢はそのアクターのグループで出したもので、受け継ぐのは SSP の振る舞い。要件にも選択肢の見え方の定めは無い。
- **代案**: 選択肢の直前に `\f[default]`、直後に装いを出し直す（出力が増え、`vertical-balloon-lines` の選択肢の区切りと同じ場所を触る）。
- **例**: `font = { height = 14 }` のおじさんのグループで選択肢 → `\p[0]\f[height,14]どっち？\n[150]\![*]\q[こっち,…]`（「こっち」も 14 の大きさ）。

### OQ-5: 縦書きのバルーンでの `\f[height]` と `\n[180]` の順

- **仮定**: 横書きでは順が表示に効かなかった（実機 #6）ので、縦書きでも装いのタグ → 改行の順（要件 2.5）のままにする。
- **推奨**: 草案のまま。実装の後に、窓際のぱすたさんの縦書きのバルーン（`vertical,1`）で `\p[0]\f[height,30]あ\p[0]\f[default]\n[180]い` と `\p[0]\f[height,30]あ\n[180]\f[default]い` を比べ、列の送りに差があれば報告する。
- **代案**: 戻すタグを改行の後ろに置く（`\p[0]\n[180]\f[default]い`）。要件 2.5 の例の順と違うので、要件の変更が要る。

### OQ-6: `config-actor-font` と飛び先の検査

- **仮定**: `skip_reasons` に入れない（飛び先の検査を続ける）。
- **推奨**: 草案のまま（装いはシーンの集まりに関わらない）。`debug/validate.md` の「省く理由」の列挙には足さない。
- **代案**: ほかの `config-*` と同じく省く理由に入れる（一覧の見た目はそろうが、報告が減る）。
- **例**: `font = { colour = "red" }` と、存在しない飛び先 `＞おわり` が同じゴーストにあるとき、草案では `config-actor-font`（誤り）と `call-target-missing`（注意）の両方が出る。

## Risks & Mitigations

- `\x` の振る舞いが未確認 — どちらでも表示が同じになる出し直しで備え、実装時に実機で確かめる（OQ-2）。
- `sakura_builder.lua` を `vertical-balloon-lines` などと共有 — 変更を 2 つの関数の引数と 1 行ずつに限る。
- スキル references の再生成が `.claude/skills/` への書き込みで分類器に拒まれる — 生成した差分を示して開発者に許可を求める。
- `@pasta_config` の `font` の形が TOML と違う — マニュアルの 3 か所に書き、`config_test.rs` で固定する。

## References

- ukadoc: `\f[…]`（各属性・`default`）、`\x`・`\x[noclear]`、`\c`、`\n[パーセント]`（ギャップ分析で参照済み）。
- `toml` 1.1.7 のソース `src/de/parser/detable.rs`（`DeTable::parse`）。
- `.kiro/specs/completed/scene-name-alias/design.md`（規則を `pasta_core` に置く先例）。
- `.kiro/specs/completed/actor-surface-restore/`（`appearance.lua` の状態を持たないモジュールの先例）。
