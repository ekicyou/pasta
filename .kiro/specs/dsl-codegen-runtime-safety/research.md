# ギャップ分析: dsl-codegen-runtime-safety

## Summary
- **Feature**: `dsl-codegen-runtime-safety`
- **Discovery Scope**: Extension（既存のコード生成・act ランタイム・さくらスクリプト組み立ての拡張）
- **分析日**: 2026-10-04（main `8033b4d7` 相当のワークツリーで確認）
- **Key Findings**:
  - 4 件の 500（U18・U19・U20・U22）は、すべて `crates/pasta_lua/src/code_gen/element_gen.rs` の出力形（`GLOBAL.名前(…)`・`act.アクター:…`・生の算術演算子）が原因。生成側を「act の存在確認付きの口」経由に変えれば、ランタイム側の追加は `act.lua` の小さなメソッド 3 つ程度で済む。既存の `act:expr_fn`（警告＋nil）・`act:set_spot`（名前を文字列で受ける）という前例がある。
  - U08 は生成側だけでは直らない。`\\` を `talk` でも `sakura_script` でも、最終組み立ての Rust トークナイザ（`sakura_script/tokenizer.rs` の `SAKURA_TAG_PATTERN`）が `\\` を 1 単位として認識しないため、`\` が普通の文字として扱われ、ウェイト挿入（`script_wait_normal` > 50 のとき）や BudouX 改行（`line_breaker.rs`）で 2 文字の間に何かが挟まりうる。パターンに `\\\\` の選択肢を足す小変更が要る（brief の Boundary Candidates に無い追加の接点）。
  - 生成コードの形が変わるため、トランスパイラのスナップショット 22 件前後・期待値 Lua フィクスチャ 3 件・生成形を文字列で検査する Rust テスト 7 ファイル程度が更新対象になる。行数は変わらない（1 アクション＝1 行の形を保てる）ため、ソースマップの対応は維持しやすい。

## 1. 現状調査

### 1.1 関係する資産

| 資産 | 場所 | 現状の役割 | 本仕様との関係 |
| ---- | ---- | ---------- | -------------- |
| アクション生成 | `crates/pasta_lua/src/code_gen/element_gen.rs` `generate_action`（309–434 行） | 全アームが `act.{actor}:talk(…)`／`act.{actor}:word(…)`／`act.{actor}:expr_fn(…)`／`act.{actor}:sakura_script(…)` を出力。`＠＊` は `GLOBAL.{name}(act, …)`（372 行） | U18・U19・U20・U08 の主戦場 |
| 式生成 | 同 `generate_expr_to_buffer`（452–528 行） | `FnCall Global` → `GLOBAL.{name}(act, …)`（489 行）、`Binary` → 生の ` + - * / % `（513–524 行）、`Paren` → `( … )` | U18・U22 |
| 変数パス | 同 `resolve_var_path`／`dynamic_ref_args` | `var.x`・`save.x`・`args[n]` を素で出力。`dynamic_ref_args` は値＋パス文字列を渡す前例 | U22 の警告に変数名を渡す手段として流用できる |
| `％` 行 | `crates/pasta_lua/src/code_gen/scope_gen.rs` 278–282 行 | `act:clear_spot()`／`act:set_spot("名前", n)` — **既にアクター名を文字列で渡す形** | U19・U20 の新しい形の前例。U02（未登録は無視）は変えない |
| act 本体 | `crates/pasta_lua/pasta_scripts/pasta/act.lua` | `ACT_IMPL.__index`（146–158 行）: メソッド → `self.actors[key]` → nil。`ACT.new` の実フィールド: `actors`・`save`・`app_ctx`・`var`・`token`・`current_scene` | U19（nil）・U20（メソッド・フィールドが先）の原因 |
| SHIORI act | `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` | `SHIORI_ACT_IMPL.__index` → `rawget(SHIORI_ACT_IMPL)` → `ACT.IMPL.__index`。追加フィールド `_spot_newlines`・`req`、追加メソッド `build`・`get_property`・`set_property`・`transfer_date_to_var`・`transfer_req_to_var` | U20 で衝突する名前が増える。`ACT_IMPL` に足したメソッドは継承で自動的に使える |
| 警告＋nil の前例 | `act.lua` `call_expr`（409–417 行）・`ACT_IMPL.talk`（nil は空・`var_name` があれば警告）・`ACT_IMPL.call`（nil key ガード） | `log.warn("act:expr_fn - handler not found: key='…'…")` の書式 | U18・U22 の警告の書式・レベルの基準 |
| アクター | `crates/pasta_lua/pasta_scripts/pasta/actor.lua` | `ACTOR.get_or_create`（`PASTA.create_actor` の実体）、`ACTOR.create_proxy`、`PROXY_IMPL`（talk・sakura_script・word・expr_fn・expr_fn_var） | **境界外**（Wave 1 では `scene-search-key-normalization` が持つ）。呼ぶのは可、変更は不可 |
| 最終組み立て | `pasta_scripts/pasta/shiori/sakura_builder.lua` | talk・sakura_script の両方を `@pasta_sakura_script.talk_to_script(actor, text)` に通す。`merge_consecutive_talks` は連続 talk の文字列を連結する（sakura_script は連結しない）。has-text は「非空 talk」だけで立つ | U08: talk で `\` を 1 文字出すと隣の talk と連結して `\n` などのタグになる |
| さくらスクリプト変換 | `crates/pasta_lua/src/sakura_script/tokenizer.rs`（`SAKURA_TAG_PATTERN = \\[0-9a-zA-Z_!+*?&-]+(?:\[[^\]]*\])?`）、`wait_inserter.rs`、`line_breaker.rs`（同じ正規表現を `find_iter`） | `\\` を 1 単位と認識しない。`C:\\new` は `\`（普通の文字）＋`\new`（タグ）に分かれる。General 文字の後には `script_wait_normal - 50` ms のウェイトが入る | U08 の根本。生成側で `\\` を出しても、ウェイト・改行の挿入で `\` と `\` の間が割れうる |
| 文法 | `crates/pasta_dsl/src/parser/grammar.pest` 182 行 `sakura_escape = @{ sakura_marker{2} }`、`id = !(reserved_id) ~ identifier` | `\\` は `Action::Escape` として AST に入る（`＠＠`・`＄＄` と同じアーム） | 文法は**変えない**（境界外）。`Escape` アームの出力だけを変える |
| 500 の経路 | `pasta_scripts/pasta/shiori/entry.lua` `SHIORI.request`（`xpcall(EVENT.fire)` → `RES.err`） | シーン内の Lua 実行時エラーはここで 500 になる | 変更不要。テストの観測点 |

### 1.2 規約・パターン
- 生成コードは 1 アクション＝1 行（`writeln` 1 回）。`out_line` の差分で `record_span` を呼ぶ（ソースマップ）。式は 1 行の中に入れ子で展開される。
- 名前は `StringLiteralizer::literalize` で Lua 文字列リテラルにする（`\` や `"` を含むとロングブラケット）。
- act のメソッドは `ACT_IMPL.名前(self, …)`、警告は `log.warn(string.format("act:メソッド - 内容: key='%s'", …))`。
- テスト配置: トランスパイラは `crates/pasta_lua/tests/transpiler/*`（insta スナップショット `snapshots/*.snap`）、Lua 単体は `crates/pasta_lua/tests/lua_specs/*_test.lua`（`lua_unittest_runner.rs` が実行）、SHIORI 経由は `crates/pasta_lua/tests/shiori/*` と `crates/pasta_shiori/tests/*`。

### 1.3 生成形に依存しているテスト・資料（更新対象の見積もり）
- `crates/pasta_lua/tests/transpiler/snapshots/` のスナップショット: 22 ファイル前後（`act.X:talk` か `GLOBAL.X(act` を含むもの）
- `crates/pasta_lua/tests/transpiler/*.rs`: 7 ファイル（`basic_test`・`code_generator_test`・`dynamic_word_ref_test`・`scene_test`・`source_map_seam_test`・`transpile_sink_port_test`・`cue_command_passthrough_test`）
- `crates/pasta_lua/tests/fixtures/`（`sample.expected.lua`・`sample.generated.lua` など 3 件）、`crates/pasta_lua/tests/property_*_test.rs`、`runtime/syntax_test.rs`
- `lua_specs` 6 件・`pasta_shiori/tests` のフィクスチャ 2 件は、手書き Lua で `act.X:talk` を使っている（手書き Lua の挙動は変えないため、多くは更新不要の見込み。要確認）
- マニュアル: `grammar/action-line.md`・`grammar/variables.md`・`grammar/words.md`（`＠＊` 周辺）・`lua/script-api.md`（207 行付近「生成コードは…この形で書く」）・`lua/patterns.md`・`internals/internal-modules.md`（117 行付近）・`internals/transpiler.md`（184–242・308 行付近）・`internals/talk-output.md`（34・58 行）
- 手書きスキル: `.claude/skills/pasta-ghost-authoring/SKILL.md` 176 行（`＠＊func()` → `GLOBAL.func(act)`）
- 生成スキル: `book/tools/gen-skill-refs.mjs` の `GENERATION_MAP` に `grammar/action-line.md`・`grammar/variables.md`（GA）、`lua/script-api.md`・`internals/internal-modules.md`（LC）が載っている。`internals/transpiler.md` は生成対象外

## 2. 要件と資産の対応（Requirement-to-Asset Map）

| 要件 | 必要なもの | 既存資産 | ギャップ |
| ---- | ---------- | -------- | -------- |
| R1.1–1.3 未定義 `＠＊` | 名前を文字列で受け、`GLOBAL[名前]` が関数なら `(act, …)` で呼び、無ければ警告＋nil | `call_expr` の警告＋nil、`GLOBAL` は `act.lua` で require 済み | **Missing**: 存在確認付きの呼び出し口（例 `act:global_fn("名前", …)`）。生成 2 か所（372・489 行）の置換 |
| R1.4 第 1 引数は act | アクション行でも act を渡す | 現行もアクション行で `GLOBAL.f(act, …)`（プロキシではない） | なし（口の中で `self` を渡せばよい。プロキシ経由にしないこと） |
| R2.1–2.2 メンバー名と同名のアクター | アクター名を文字列で受けてプロキシを返す口 | `ACTOR.create_proxy(actor, act)`・`self.actors[name]`・`set_spot` の前例 | **Missing**: プロキシ取得口（例 `act:actor_proxy("名前")`）。生成 8 アームの `act.{actor}` を置換 |
| R2.3–2.4 未登録アクター | 警告し、出力を捨て、行内の評価は続ける | なし | **Missing / Constraint**: 「出力を捨てるプロキシ」をどう作るか。`PROXY_IMPL` は境界外で変更できないため、act 側で代用オブジェクトを返す必要がある（`talk`・`sakura_script`・`word`・`expr_fn`・`expr_fn_var` を持つ）。OQ-1 の決定次第で不要になる（自動作成なら `ACTOR.get_or_create` を呼ぶだけ） |
| R2.5 手書き `act.名前` 不変 | `__index` を変えない | 現行 | なし（新しい口を足すだけ） |
| R3.1–3.4 算術 | 被演算子を数値化し、失敗なら警告＋nil、成功なら Lua と同じ結果 | なし | **Missing**: 算術ヘルパー（例 `act:arith(op, a, b, …)`）。`Binary` アームを入れ子の呼び出しにする。警告に変数名・関数名を出すには、生成側が被演算子の「説明文字列」を渡す必要がある（`dynamic_ref_args` と同じ考え方） |
| R3.5 文字列 `＋` | 連結しない | 現行（Lua の算術） | なし（ヘルパーでも `tonumber` 相当で変換するだけ） |
| R3.6 全位置に適用 | 式の生成が 1 か所 | `generate_expr_to_buffer` に集約済み（代入・式文・引数・Call 引数・動的コール・プロパティ代入はすべて経由） | なし（1 か所直せば全位置に効く） |
| R3.7 数値化の範囲 | 現行の暗黙変換と同じ | Lua の文字列→数値の暗黙変換は `tonumber` と同じ規則（16 進・指数表記・前後の空白を受ける） | **Research Needed（小）**: LuaJIT の暗黙変換と `tonumber` の差が無いことをテストで固定 |
| R4.1–4.4 `\\` | 最終出力に `\\` をそのまま残す | `Escape` アームは 2 文字目だけ talk で出す | **Missing**: `\\` 用の出力（例 `talk("\\\\")` で 2 文字を出す、または `sakura_script("\\\\")`） |
| R4.5 ウェイト・改行で割らない | トークナイザが `\\` を 1 単位として認識する | `SAKURA_TAG_PATTERN` は `\\` を含まない | **Missing**: パターンの先頭に `\\\\` の選択肢を足す（`tokenizer.rs` と、同じ正規表現を使う `line_breaker.rs` の両方に効く）。手書き Lua の talk 文字列中の `\\` にも効く（改善方向の振る舞い変更） |
| R4.6 表示文字として扱う | has-text・スポット改行の判定 | has-text は「非空 talk」で立つ | **Constraint**: talk 経路で出せば has-text は立つが、トークナイザで `\\` を SakuraScript 種別にするとウェイトが入らない（表示文字だがウェイト無し）。sakura_script 経路で出すと has-text が立たない。OQ-6 と設計で整理 |
| R4.7 `＠＠`・`＄＄` 不変 | 同じ `Escape` アームを分岐 | `escape.chars().nth(1)` | なし（`\` のときだけ分岐） |
| R5.1 既存出力不変 | 正常系の最終出力が同じ | — | 生成コードは変わるが最終出力は同じであることを、既存の SHIORI・ランタイムテストで確認 |
| R5.2 500 にならない | SHIORI 経由のテスト | `tests/shiori/*`・`pasta_shiori/tests/*`（`xpcall` → `RES.err` の経路） | **Missing**: 5 件それぞれのフィクスチャとテスト |
| R5.4 ソースマップ | 1 アクション 1 行を保つ | `record_span`・`source_map_seam_test.rs`・`loader_source_map_build_test.rs` | なし（形を 1 行に保てば既存テストで確認できる） |
| R6 マニュアル・スキル | 該当章の更新と再生成 | `gen-skill-refs.mjs`（`--check`）・`link-check.mjs` | 1.3 の資料一覧 |

## 3. 実装方針の選択肢

### Option A: 既存コンポーネントの拡張（act.lua にメソッドを足す）
- `element_gen.rs`: `generate_action` の 8 アームの `act.{actor}` を `act:actor_proxy("名前")` 相当に、`GLOBAL.{name}(act, …)` を `act:global_fn("名前", …)` 相当に、`Binary` を `act:arith("+", 左, 右, 説明…)` 相当の入れ子呼び出しに、`Escape`（`\` のとき）を `\\` 2 文字の出力に変える。
- `act.lua`: `ACT_IMPL` に 3 メソッドを足す。`SHIORI_ACT_IMPL` は継承で自動的に使える。
- `tokenizer.rs`: `SAKURA_TAG_PATTERN` に `\\\\` の選択肢を足す。
- **Trade-offs**:
  - ✅ 変更が最小。既存の前例（`expr_fn`・`set_spot`・`dynamic_ref_args`）に沿う。
  - ✅ アクター名・関数名を文字列で渡すため、Lua の予約語と同じ名前（`end`・`and` など。pasta の `id` 規則は予約語を除外していない）でも生成コードが構文エラーにならなくなる（副次的な改善）。
  - ❌ `ACT_IMPL` にメソッドを足すと、act のメンバー名（手書き Lua の `act.名前` の制限の一覧）と、`＠名前（）`／`＠名前` の 5 段検索の 3 段目（act のメソッド）に新しい名前が増える。`＠arith（）` と書くとヘルパーに当たる。名前の選び方（`_` 始まりなど）を設計で決める必要がある。
  - ❌ 未登録アクターで出力を捨てる場合、`PROXY_IMPL` を触れないので、act 側に代用オブジェクト（no-op の talk 等）を作る必要がある。

### Option B: 生成コード用の新しいランタイムモジュール
- 生成コード専用の小モジュール（例 `pasta.codegen_rt`）を作り、シーンファイル先頭で `local RT = require(…)` して `RT.global_fn(act, "名前", …)` の形で呼ぶ。
- **Trade-offs**:
  - ✅ act のメンバー名・5 段検索の 3 段目を汚さない。
  - ✅ 生成コード専用の責務が 1 か所にまとまる。
  - ❌ 生成ファイルの先頭部（`scope_gen.rs` のファイルヘッダ）にも変更が入り、スナップショットの変更がさらに広がる。
  - ❌ brief の Boundary Candidates（`act.lua` にヘルパー）から外れる。後続仕様（`actor-proxy-act-delegation`）は「新しいプロキシ取得口」を act にある前提で書かれている。

### Option C: ハイブリッド
- U18・U19・U20 は act のメソッド（Option A）、U22 の算術だけは生成コードのローカル関数またはモジュール関数（Option B）にする。算術は act の状態を使わないため act に置く必然性が無い。
- **Trade-offs**:
  - ✅ act に足す名前を 2 つに抑えられる。
  - ❌ 生成コードの呼び出し形が 2 系統になり、マニュアル（`internals/transpiler.md`）の説明が増える。

## 4. 工数とリスク
- **Effort: M（3–7 日）** — 個々のコード変更は小さいが、スナップショット・期待値フィクスチャ・生成形を検査するテストの更新が広く、マニュアル 8 章前後とスキルの再生成が伴う。
- **Risk: Medium** — 使うパターンは既存どおりだが、(1) 生成形の変更がスナップショットを広く揺らす、(2) `\\` のトークナイザ変更が手書き Lua の talk 文字列にも効く、(3) 未登録アクターの扱い（OQ-1・OQ-2）が `PROXY_IMPL` を触れない制約の中で実装形を左右する。

## 5. 設計フェーズへの申し送り

### 推奨
- Option A を基本にする（brief の推奨・後続仕様の前提と一致）。act に足すメソッド名は、手書き Lua・5 段検索と衝突しにくい名前を設計で決める（Option C の採否もここで判断）。
- U08 は「生成側で `\\` を 2 文字出す」＋「トークナイザで `\\` を 1 単位にする」の 2 点セットで直す。生成側だけでは R4.5 を満たせない。
- U22 の警告に名前を出すため、生成側から被演算子の説明（変数パス・関数名）を文字列で渡す。`dynamic_ref_args` と同じ考え方で、値は加工せず渡す。

### 設計で決めること（Research Needed）
1. act に足すメソッドの名前と、5 段検索の 3 段目・手書き Lua の `act.名前` の制限一覧への影響（マニュアル `lua/script-api.md` の更新範囲）。
2. 未登録アクター（OQ-1・OQ-2）の実装形。出力を捨てる代用オブジェクトを act 側に置くか、自動作成なら `ACTOR.get_or_create`（`STORE.actors` に残る＝以後のイベントでも登録済みになる）の副作用を許すか。代用オブジェクトの場合、`word`・`expr_fn` がアクター辞書の段（A1・A2）を飛ばして act の検索に委ねる形でよいか。
3. 警告の回数: 未登録アクターの行は 1 アクション＝1 文のため、1 行に複数のアクションがあると警告が複数回出る。入れ子の算術も失敗が伝わるたびに警告が出うる。許容するか、1 回に抑えるか。
4. `\\` のトークナイザ上の種別（SakuraScript 扱いでウェイト無し／General 扱いで 1 文字として数える）と、BudouX の改行位置推定での幅の数え方（OQ-6 と合わせて決める）。
5. LuaJIT の暗黙の文字列→数値変換と、ヘルパーで使う変換の規則が一致することの確認（R3.1・R3.7）。`％` の負数の扱い（Lua の `a - floor(a/b)*b`）を含めて同じ結果になること。
6. 生成形を文字列で検査している既存テストの更新範囲の確定と、正常系の最終出力が変わらないことを示すテスト（SHIORI 経由）の選定。
7. （要件ディスカッションで追加）`pasta_scripts/pasta/shiori/appearance.lua` も `SAKURA_TAG_PATTERN` と同じ規則で `\` 始まりのタグを読む（`NAME_PATTERN`・`ARG_PATTERN`）。`\\s[0]` のような台詞で、2 文字目の `\` から `\s[0]` を表情タグとして読み誤るおそれがある。トークナイザと同じく `\\` を 1 単位として飛ばす必要があるかを確認する（R4.2・R4.5）。

### 境界の確認
- `actor.lua`（`PROXY_IMPL`）・`pasta_dsl` の文法・`shiori/event/*` は触らない（Wave 1 の並走条件）。`sakura_script/tokenizer.rs` は brief の Boundary Candidates に載っていないが、並走条件の禁止対象ではない。設計の Boundary Commitments に明記する。
- `entry.lua` の `xpcall` → 500 の経路は変えない（テストの観測点として使うだけ）。
