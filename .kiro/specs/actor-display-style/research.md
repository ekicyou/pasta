# ギャップ分析: actor-display-style

- **対象**: `requirements.md`（2026-10-18 生成・未承認）と、main `06c2805a` 時点のコード
- **分析の範囲**: 出力の組み立て（Lua）・アクター設定の読み込み（Rust／Lua）・`pasta_check validate`・テスト・マニュアル・スキル references
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
