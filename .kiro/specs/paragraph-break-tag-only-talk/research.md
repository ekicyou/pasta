# ギャップ分析: paragraph-break-tag-only-talk

## 1. 現状調査（要件 ↔ 既存資産マップ）

| 要件 | 既存資産 | ギャップ |
|---|---|---|
| R1 字の定義 | `appearance.lua` 13–55 行: `NAME_PATTERN`・`ARG_PATTERN`・`tag_at`・`next_tag`（いずれもモジュール内の `local`）。`\\` は `tag_at` が `nil` を返し、`next_tag` が 2 文字読み飛ばす。`\` ＋ タグ名に使えない文字も `nil`（1 文字進める）。Rust の `Tokenizer::SAKURA_TAG_PATTERN`（`sakura_script/tokenizer.rs` 127 行）が同じ定義 | **Missing**: 「タグを除いて字が残るか」を返す関数が無い。タグの読み取り部品はあるが `appearance.lua` の外から使えない（公開されていない）。**Constraint**: 文字を表示するタグ（`\_u`・`\_m`・`\&`）を字として数える例外は、どちらの側にも無い（新規の規則） |
| R2 字の無い talk の扱い | `sakura_builder.lua` 184 行（S3）: `inner.type == "talk" and inner.text ~= nil and inner.text ~= ""` で保留の改行を出し `spot_has_text[last_spot] = true`。195 行（S4）: 出力だけ | **Missing**: S3 の条件に「字がある」を足し、字の無い `talk` を S4 に流す。S4 はすでに「出力＋外見の観測、状態は不変」なので 2.1–2.3・2.5 はこれで満たせる。2.4（字の無い talk → 別トークン → 字のある talk で、改行が字のある talk の直前に出る）も S3/S4 の分岐をそのまま使えば自然にそうなる |
| R3 既存出力の不変 | 判定は `talk_to_script` 前の `inner.text` に対して行える（1.8）。`talk_to_script` はタグにウェイトを付けず、字の無い入力に BudouX の改行も入れない | 字のある `talk` の経路は条件を狭めるだけで不変。**既存のテストへの影響**: `sakura_builder_test.lua`（2428 行）の `talk` はすべて字を含む（`\s[0]A1` など）ため期待値は変わらない見込み。`shiori_act_test.lua`・`pasta_shiori` の e2e（`byte_invariant_test.rs`・`codegen_runtime_safety_e2e_test.rs`・`kick_unused_byte_invariant_test.rs`）の `\n[150]` も字のある台詞の前だけ。フィクスチャの `.pasta` に `＠単語` だけの行は見当たらない（`sample.expected.lua` に表情の単語 `\s[0]` などの定義はある） |
| R4 申し送りの並び | 経路: `＠通常` → `act.エモ:talk(act.エモ:word("通常"))` → `ACT_IMPL.talk`（`act.lua` 205 行）が `tostring` して `talk` トークンへ。同じアクターの連続する `talk` は `merge_consecutive_talks`（`act.lua` 85 行）で 1 つに結合される | ビルダーの修正だけで 4.1・4.2 を満たせる。4.4（DSL からの経路）は `act.lua` を変えずに満たせる。**Constraint**: `＠通常` の行に同じアクターの字のある行が続くと 1 つの `talk` に結合され、字のある `talk` として扱われる（3.1 のとおり、改行は結合後の先頭に出る。申し送りの原文 `\p[0]\n[150]\s[1000]…つまり、、` と同じ形） |
| R5 マニュアル | `book/src/internals/talk-output.md`: 状態表の `spot_has_text`「空でない `talk` を出力したか」・`pending_break`「次の空でない `talk` の前」、本文「内側のトークンのうち、空でない `talk` の前では…」「それ以外のトークン（空の `talk`・`sakura_script` を含む）は判定の状態を変えない」。`book/src/reference/pasta-toml.md` 233 行（表）・248–268 行（`#### spot_newlines`） | **Missing**: 内部設計の基準を「字のある `talk`」へ、字の定義を追記。利用者向けにタグだけの出力は台詞に数えない旨の 1 項目を追記。`reference/pasta-toml.md` は `gen-skill-refs.mjs` の `GENERATION_MAP` で `pasta-ghost-authoring/references/pasta-toml.md` に生成される（再生成が要る）。`internals/` は生成対象外 |
| R6 テスト | `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua`（`lua_unittest_runner.rs` が実行）。`sakura-script-newline` の節（597–1273 行）に同種の保留・破棄のテストがある。`appearance_test.lua` にタグ読み取りの境界ケース（`\\s[0]`・`\_w[100]\s3あ` など） | **Missing**: 申し送り (a)・(b) のバイト一致テスト、字の境界ケースのテスト。既存のヘルパー（`group`・`talk`・`script`）をそのまま使える |

その他の観察:

- `sakura_builder.lua` と `appearance.lua` の複製（`pasta_shiori/tests/support/` などの写し）は無い。`spot_has_text`・`pending_break` を参照するのは `sakura_builder.lua` だけ。
- `BUILDER.build` は循環的複雑度 22（`.luacheckrc` の上限 15）を `luacheck: ignore 561` で許容している。条件を S3 に直接足すと分岐が 1 つ増える。判定を小さな局所関数（「字のある talk か」）に外出しすれば増えない。
- Rust の `@pasta_sakura_script` は「アダプタが注入するレンダラ」（`runtime/renderer_injection.rs`・`factory.rs` 191 行）として登録される。Rust 側に判定関数を足す案は、注入されるレンダラ全部に関数を足す必要があり、境界が広がる。

## 2. 字の境界のケースと既存のタグ読み取りの挙動

| 入力（`talk` のテキスト） | 既存の `tag_at`/`next_tag` の読み | 要件の扱い |
|---|---|---|
| `\s[1000]\![bind,腕,組み,1]` | タグ 2 つ | 字なし（1.2） |
| `\1\![move,-353,,,0,base,base]` | タグ 2 つ（`\1` はスコープ切替） | 字なし（1.2） |
| `\n`・`\n[150]`・`\_w[500]` | タグ | 字なし（1.2） |
| `\\` | タグではない（2 文字読み飛ばし） | 字（1.3） |
| `\_u[0x3042]`・`\_m[0x41]`・`\&[amp]` | タグ（名前 `_u`・`_m`・`&`） | 字（1.4。新規の例外。名前で判定） |
| `\s[0]　`（全角空白） | タグ＋文字 | 字（1.5 の仮定） |
| `\q[はい,OnYes]` | タグ（名前 `q`） | 字なし（1.6 の仮定。現行は字あり → 出力が変わる） |
| `\あ` | タグではない（`\` を 1 文字進める） | 字（1.7） |
| `\s[0]A1` | タグ＋文字 | 字（3.1。従来どおり） |

既知の制約（タグの区切り方を共有することによる。要件の範囲外だが設計で記録する）:

- タグ名は貪欲に読むため、`\nHello` は名前 `nHello` の 1 つのタグになり字なしと判定される（ウェイトの挿入も同じ読み方で `Hello` にウェイトを付けない）。SSP は `\n` ＋ `Hello` と読む。単語の値に ASCII の字が改行タグの直後に続く書き方でだけ起き、段落区切りの改行が出なくなる方向に効く。
- `\_?…\_?`（タグをそのまま表示する囲み）の中のタグもタグとして読む。ウェイトの挿入と同じ制約。

## 3. 実装アプローチの選択肢

### Option A: ビルダー内で判定し、タグの読み取りを `appearance.lua` から公開して共有する（brief の推奨 (1)）
- `appearance.lua` から読み取りの部品（`tag_at`／`next_tag`、または「字が残るか」を返す関数）を公開し、`sakura_builder.lua` の局所関数で「字のある talk か」を判定して S3 の条件に使う。文字を表示するタグの例外（1.4）はビルダー側に置く。
- ✅ 差分が最小（Lua 2 ファイル）。タグの定義が外見の観測と 1 か所で一致する。Wave 3 の並走条件（`act.lua`・`element_gen.rs`・Rust の `sakura_script/` に触れない）を守れる。
- ❌ `appearance`（外見の観測・復旧）が「字の判定」に使われる部品も持つことになり、モジュールの責務が少しにじむ。公開する関数の名前と範囲を決める必要がある。

### Option B: タグの読み取りを新しい小さなモジュールへ移し、`appearance.lua` と `sakura_builder.lua` の両方から使う
- `NAME_PATTERN`・`ARG_PATTERN`・`tag_at`・`next_tag` を新モジュール（例: `pasta.shiori.sakura_tag`）へ移し、字の判定もそこに置く。
- ✅ 責務が明確。将来ほかの Lua コードがタグを読むときにも使える。
- ❌ `appearance.lua` の変更が「読み取りの共有だけ」を超えて移動になる（brief の並走条件の文言より広い）。新しいファイルが増え、スクリプトの埋め込み・ローダの登録を確認する必要がある（Research Needed）。

### Option C: Rust の `@pasta_sakura_script` に判定関数を足す（brief の案 (2)）
- `SAKURA_TAG_PATTERN` で分けた結果から字の有無を返す関数を足し、ビルダーから呼ぶ。
- ✅ タグの定義が Rust の 1 か所（トークナイザ）にそろう。
- ❌ brief の Out of Boundary（Rust の `sakura_script/`）に触れる。アダプタが注入するレンダラすべてに関数が要る。Lua のテストでモックの扱いが増える。差分と境界が最も大きい。

## 4. 工数・リスク
- **工数: S**（1–3 日）。コードの変更は判定関数と S3 の条件だけ。テスト（申し送りの 2 例＋境界のケース）、マニュアル 2 章、`references/` の再生成。
- **リスク: Low**。既存の分岐（S3/S4）と既存の部品（タグの読み取り）を延長するだけ。字のある `talk` の経路は条件を狭めるだけで、既存のテストの期待値は変わらない見込み。判断点は字の境界（未決事項 1–3・5）だけ。

## 5. 設計フェーズへの推奨
- 推奨は **Option A**。判定は `talk_to_script` に渡す前の `inner.text` に対して行い（要件 1.8）、S3 の条件を「`talk` で字がある」に狭める。字の無い `talk` は S4 に流れるので、保留・破棄・外見の観測は既存の規則のまま動く。
- 判定は小さな局所関数に外出しして、`BUILDER.build` の複雑度を増やさない。
- 決めること: `appearance.lua` から何を公開するか（部品 `next_tag` 系か、字の判定そのものか）、文字を表示するタグの一覧（`_u`・`_m`・`&`）を置く場所。
- Research Needed:
  - `cargo test` の全体（`pasta_lua` の lua_specs、`pasta_shiori` の e2e）を走らせ、字の無い `talk` を含む並びの期待値が既存のテストにあるかを確認する（静的な検索では見つからなかった）。
  - Option B を選ぶ場合、新しい Lua モジュールの埋め込み（`pasta_scripts` の登録）手順。

## 6. 要件ディスカッションで確認する事項（未決事項の論点）
1. 文字を表示するタグ（`\_u`・`\&`、brief に無い `\_m`）を字として数えるか。
2. 空白だけの残りを字として数えるか（仮定は数える。空白を特別扱いする規則を足さない）。
3. `\q[…]` など引数を表示するその他のタグを字なしにするか（仮定は字なし。現行は字ありで、出力が変わる）。
4. 字の無い `talk` と字のある `talk` が別トークンのときの改行の位置（仮定は字のある `talk` の直前。既存の S3/S4 の分岐からの自然な帰結）。
5. `sakura_script` トークン（DSL の行内に直接書いたタグ）は、文字を表示するタグを含んでも字なしのままとするか。
6. 判定の位置（Option A/B/C）を要件フェーズで決めるか、設計フェーズに送るか（本分析の推奨は Option A）。

### 要件ディスカッションの決定（2026-10-05）
- 1: 文字を表示するタグ `\_u`・`\_m`・`\&` を字として数える（自明として確定）。
- 2: 空白だけの残りも字として数える（自明として確定）。
- 3: `\q[…]` など、引数の文字を表示するその他のタグは字に数えない（要件 1.6）。
- 4: 改行は字のある `talk` の直前に出す（完全遅延の規則からの帰結として確定）。
- 5: 文字を表示するタグの `sakura_script` トークンも字として数え、書いた経路によらずそろえる（要件 1.9）。ビルダーの S3/S4 の振り分けで `sakura_script` にも同じ判定を当てる。
- 6: 判定の位置は設計フェーズで決める（本分析の推奨は Option A）。
