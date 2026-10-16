# ギャップ分析: failure-output-unification

- 実施日: 2026-10-16（main `3071114a`）
- 対象: `requirements.md`（要件 1〜10、未決事項 Q1〜Q13）
- 方針: 情報と選択肢を示す。最終判断は要件ディスカッション・設計で行う。

## 1. 現状の調査

### 1.1 失敗の出口（2026-10-16 時点の行番号）

| 失敗 | 場所 | ログ | バルーン |
| --- | --- | --- | --- |
| アクション行の値の無い変数 | `act.lua` 205〜213 `ACT_IMPL.talk`（`var_name` があり `text == nil` のとき） | `act:talk - undefined variable: '<var_path>'` | 何も積まない |
| 単語が見つからない（act） | `act.lua` 404〜407 `ACT_IMPL.word` | `act:word - handler not found: key=…, mode='word', via=act` | nil を返す（呼び出し元の `talk` が黙る） |
| 単語が見つからない（proxy） | `actor.lua` 248〜251 `PROXY_IMPL.word` | `proxy:word - handler not found: … via=proxy(<名前>)` | 同上 |
| 関数が見つからない（act） | `act.lua` 421〜431 `call_expr` | `act:expr_fn - handler not found: …` | 同上 |
| 関数が見つからない（proxy） | `actor.lua` 188〜198 `call_expr` | `proxy:expr_fn - handler not found: …` | 同上 |
| グローバル関数が見つからない | `act.lua` 493〜501 `ACT_IMPL.global_fn` | `act:global_fn - function not found: key=…` | 同上 |
| 動的参照のキーにならない | `word.lua` 168〜183 `WORD.dynamic_key(value, var_path, via)` | `<via> - undefined variable / empty variable / unsupported value type` | nil を返す |
| 未登録アクター | `act.lua` 464〜484 `ACT_IMPL.actor_proxy` | `act:actor_proxy - unregistered actor: name=…` | `talk` トークン（アクター付き）`【未登録アクター：名前】` |
| 未登録アクターの立ち位置 | `shiori/sakura_builder.lua` 68〜80 `emit_actor_switch` | `actor_spots fallback: '<名前>' -> spot=0` | なし（spot 0 で出す） |
| 数にできない被演算子 | `act.lua` 524〜535 `ACT.num(op, v, desc)`（act のメソッドではない） | `act:arith - operand is not a number: …` | 0 を返す |
| 文字列にできない被演算子 | `act.lua` 543〜553 `ACT.str(v, desc)`（同上） | `act:concat - operand is not a string or number: …` | 空文字列を返す |
| Call のキーが nil（Lua 直呼びのみ） | `act.lua` 612〜616 `ACT_IMPL.call` | `act:call - nil key (undefined variable?) …` | なし |
| Call が見つからない | `act.lua` 627〜631 | `act:call - handler not found: …` | `act:failure` → `raw_script` `【Call失敗：「キー」が見つからない】` |
| Call のキーにならない値 | `act.lua` 645〜660 `ACT_IMPL.call_key` | `WORD.dynamic_key(…, "act:call")` または `act:call - key is not a string or number: …` | `act:failure` → `【Call失敗：var.x が nil】` など |
| 別名の Call が見つからない | `crates/pasta_lua/src/search/context.rs`（Rust） | `Scene not found (alias applied)`（両方の名前） | Lua 側の Call 失敗表記（書いた名前） |

`act:failure`（`act.lua` 696〜702）は「警告があれば `log.warn`、`【text】` を `raw_script` で積む」だけの関数で、呼び出し元は `act:call` と `act:call_key` の 2 か所。brief の 2026-10-10 の再測定と一致する。

### 1.2 重ねない慣行（現行）

- `ACT_IMPL.talk` は `text == nil` で `var_name` が無ければ黙る。単語・関数が見つからないとき `word`・`expr_fn` が nil を返し、生成コード `act:actor_proxy("X"):talk(act:actor_proxy("X"):word("名前"))` の外側の `talk` は `var_name` を持たないので黙る。これが「内側が警告済みなら重ねない」の実体である。
- `word`・`expr_fn_var` は `WORD.dynamic_key` が nil を返したらそこで止まり、検索の「見つからない」を出さない。
- `call_key` は「呼ばない」印 `SKIP_CALL` を返し、`act:call` は印を見たら黙る。
- 未登録アクターは、直前の話者（`self.token` の末尾から最初の `talk`/`sakura_script`）が同じ名前なら目印を繰り返さない。
- 重ねない規則は各関数に散っていて、1 か所に無い。未登録アクターでは `actor_proxy` と `sakura_builder` の 2 行が重なっている。

### 1.3 トークンと出力の性質

- `raw_script` は `group_by_actor`（`act.lua` 21〜79）で直前のアクターのグループに入り、無ければトップレベルに出る。`sakura_builder` は `raw_script` をそのまま出し、字のある発言とみなさない（段落区切り・has-text の判定に影響しない。`sakura_builder.lua` 356〜358）。
- 未登録アクターの目印は `talk` トークンなので、句読点のウェイト・budoux の改行が入り、字のある発言として段落区切りの判定にも効く。Call の失敗表記と性質が違う（要件 3.6・Q4）。
- `act:build()` はトークン 0 件なら nil（→ 204）。失敗表記が 1 つでも積まれると出力は非 nil（→ 200）になる（要件 4.5・Q10）。
- チェイントークで中断したシーンは `STORE.co_scene` に残り、再開時は**最初に作った act**（コルーチンが閉じ込めた引数）にトークンを積む。`EVENT.fire` が作る新しい act は使われない（`shiori/event/init.lua` 191〜233、`scene.lua` 193〜214）。

### 1.4 設定の読み方

- pasta.toml の `[loader]`・`[scene.alias]` 以外は `custom_fields` として `@pasta_config` から Lua に見える。Lua では `pasta.config.get(section, key, default)` で読む（`pasta/config.lua`）。新しいキーは Rust の構造体を増やさずに読める。
- `[ghost]` は `PastaConfig::apply_shiori_defaults`（`loader/config/mod.rs` 157 付近）が既定値を補う。既定値の正本は `GhostConfig::default()`（`sections.rs` 201〜）。`[ghost]` に置くならここに 1 キー足すのが既存の型。
- 型が合わない値の扱い: `[talk]`・`[persistence]`・`[logging]`・`[debug]` は既定値に戻る（マニュアル `reference/pasta-toml.md` 37 行）。`custom_fields` を Lua で直接読む場合は、Lua 側で型を確かめないと既定値に戻らない（要件 7.5）。
- `pasta_check` は pasta.toml を読んでいない（`crates/pasta_check/src/` に参照なし）。要件 7.6 の仮定は現状と合う。

### 1.5 名前の表記の出どころ

- 警告に出る名前は生成コードが渡す文字列である。変数は `var.x`・`save.x`・`args[1]`（`element_gen.rs` 48 `dynamic_ref_args`、`ACT_IMPL.talk` の第 3 引数）、関数は `@名前()`・`@*名前()`・`@$パス()`（`expr_gen.rs` 255 `operand_desc`）。
- 生成する Lua を変えない約束（要件 10.1）の下では、DSL の書き方（`＄x`・`＄＊x`・`＄０`・`＠名前（）`）への変換はランタイムで行うことになる。対応は機械的（`var.` → `＄`、`save.` → `＄＊`、`args[n]` → `＄(n-1)` を全角数字に、`@` → `＠`、`()` → `（）`）。シーン引数は 0 始まりの DSL 番号と 1 始まりの Lua 添字のずれに注意が要る。

### 1.6 マニュアルとスキル

- 失敗時の挙動は `grammar/variables.md`（92・112・158〜161・214〜243・348）、`grammar/words.md`（268〜281・306〜307・322）、`grammar/call-jump.md`（99〜119 の表・201・236・275〜292）、`grammar/action-line.md`（63〜65）、`grammar/actor-dictionary.md`（116）、`lua/script-api.md`（70・247〜296・323〜387・393〜420・558〜567・595・643〜699）、`lua/patterns.md` 188、`internals/internal-modules.md`（176〜211・256〜260）、`internals/talk-output.md`（87〜89・113・153・183）、`internals/registry-search.md` 228、`internals/transpiler.md`（230〜257）に散っている。1 か所にまとめたページは無い。
- 起動しない・喋らないときの診断は `reference/startup.md` §4（118〜182）にあるが、読み込み時の失敗だけを扱う。`debug/troubleshooting.md` は DAP の接続だけを扱う。
- スキル生成は `book/tools/gen-skill-refs.mjs` の GENERATION_MAP（23 件）。`grammar/*`・`reference/pasta-toml.md` は `pasta-ghost-authoring`、`lua/*`・`reference/startup.md`・`internals/internal-modules.md` は `pasta-lua-coding` へ出る。`internals/talk-output.md`・`lua/patterns.md` は生成対象外。新しいページを作るなら GENERATION_MAP への追加が要る。

### 1.7 失敗の文字列を確かめている既存のテスト

- 失敗表記: `pasta_lua/tests/lua_specs/act_call_restore_test.lua`・`act_runtime_safety_test.lua`、`pasta_lua/tests/scene_alias_search_test.rs`・`symbol_name_search_test.rs`・`transpiler/dynamic_call_test.rs`・`transpiler/runtime_safety_test.rs`、`pasta_shiori/tests/call_execution_correctness_e2e_test.rs`・`codegen_runtime_safety_e2e_test.rs`。
- 警告の文言: 上記に加えて `lua_specs/actor_proxy_act_delegation_test.lua`・`act_dynamic_ref_test.lua`、`transpiler/scene_test.rs`、`pasta_shiori/src/shiori.rs`（単体テスト）。
- 「未定義なら空」をバルーンの出力で確かめているテストは、既定で失敗表記を出す（Q1 の仮定）と期待値が変わる。洗い出しは設計で行う（Research Needed R5）。

## 2. 要件と資産の対応

| 要件 | 既存の資産 | ギャップ |
| --- | --- | --- |
| 1.1〜1.4 出口を 1 つ | `act:failure`（Call のみ） | **Missing**: 全種類が通る出口。`ACT.num`・`ACT.str`・`WORD.dynamic_key` は act を持たない（**Constraint**: 生成 Lua を変えない） |
| 1.5 重大度 | 全箇所 `log.warn` | なし |
| 2.1 値の無い変数 | `ACT_IMPL.talk` の警告 | **Missing**: 表記の出力 |
| 2.2〜2.4 単語・関数 | `word`・`call_expr`（act/proxy）・`global_fn` の警告 | **Missing**: 表記。proxy 側は `self.act` を持つので出口に届く |
| 2.5 動的参照 | `WORD.dynamic_key` | **Missing**: 表記。`dynamic_key` は act を受けないが、呼び出し元（`word`・`expr_fn_var`・`call_key`）は act を持つ。理由を返して呼び出し元が報告する形にできる |
| 2.6 未登録アクター | `actor_proxy` の目印 | **Constraint**: 現行は `talk` トークン。出口へ寄せると性質（ウェイト・グループの開始・段落の判定）が変わる（Q4） |
| 2.7・2.8 数・文字列にできない | `ACT.num`・`ACT.str` の警告 | **Missing**: 表記。**Unknown**: 実行中の act をどう知るか（R1） |
| 2.9 Call | `act:failure` | 載せ替えのみ。文言は Q2・Q3 で変わりうる |
| 2.10 `act:call(nil)` | 警告のみ | **Missing**: 表記（Q8） |
| 2.12 発言でない行 | — | トークン列に積めば自然に満たす。ただし `ACT.num`・`ACT.str` は R1 次第 |
| 3.1〜3.2 形を 1 か所で | 文言は各所に直書き | **Missing**: 形の定義の置き場所 |
| 3.3〜3.4 DSL の表記 | 生成コードが Lua の表記を渡す | **Missing**: ランタイムでの変換（1.5） |
| 3.5 ログの文言を保つ | 現行の文言 | なし（変えないことの確認テスト） |
| 3.6 ウェイト・改行なし | `raw_script` の性質 | 未登録アクターの目印だけ **Constraint**（Q4） |
| 3.7 エスケープ | 選択肢の `escape_choice`（`sakura_builder.lua` 17）、`escape_tag_arg`（`shiori/act.lua` 205） | **Missing**: 本文用のエスケープ。**Unknown**: `%` 変数など SSP が展開する文字の扱い（R3） |
| 3.8 行番号なし | ソースマップはデバッグ有効時のみ | なし（仮定どおりなら作業なし） |
| 4.1〜4.4 位置 | `raw_script` のグループ化、チェイントークの act の扱い | 大半は既存の性質で満たす。R1 の方式によっては 4.4 に注意 |
| 4.5 表記だけの出力 | `act:build` の nil 判定 | 既存の性質で満たす（挙動の変化として明示が要る） |
| 4.6 前後への影響なし | `raw_script` は has-text に効かない | 未登録アクターの目印の扱い次第（Q4） |
| 5.1〜5.3 重ねない | 1.2 の慣行 | **Missing**: 規則の 1 か所化。新たに表記を出す `talk` の nil 経路と、内側が報告済みかの区別（今は「`var_name` が無い」で代用） |
| 5.4 立ち位置の警告 | `sakura_builder.lua` 75 | **Constraint**: `pasta/shiori/` 配下（brief の範囲外の列挙には無い。Q9） |
| 5.6 別名の両名ログ | `search/context.rs` | 触らなければ満たす |
| 6.1〜6.3 続け方 | 現行 | なし（回帰テストで固定） |
| 6.4・7.x 切り替え | `pasta.config.get`、`[ghost]` の既定値補完 | **Missing**: キー・既定値・型検査・マニュアル |
| 8.1 Lua から報告 | `act:failure`（公開 API としてマニュアルに記載） | 拡張または置き換え（Q13） |
| 8.2 act 無しの報告 | — | **Missing**（Q12）。`runtime-error-balloon` が `SHIORI.request` の外側から使う |
| 9.x マニュアル | 1.6 の各ページ | **Missing**: 1 か所のページ・節。GENERATION_MAP の判断 |
| 10.1 生成 Lua 不変 | — | **Constraint**（全設計を縛る） |
| 10.2〜10.5 テスト | 1.7 | 期待値の更新と新規テスト |

## 3. 実装の選択肢

### 選択肢 A: `act.lua` の中に出口を広げる（既存の拡張）

- `act:failure` を一本化した出口にし、種類・対象・説明を受けて形を組み立てる。各箇所の `log.warn` を `self:failure(…)`（proxy は `self.act:failure(…)`）に置き換える。
- `WORD.dynamic_key` は警告を出さず理由を返す形にし、呼び出し元（act を持つ）が報告する。
- `ACT.num`・`ACT.str` は「実行中の act」を見つける手段が要る（R1）。
- 利点: 新しいファイルが無い。既存の `act:failure` の呼び出し元をそのまま使える。
- 欠点: `act.lua`（726 行）が大きくなる。形の定義・表記の変換・エスケープ・重ねない規則が `act.lua` に同居する。`act.lua` は共有接点で、後続の `call-attribute-filter`・`scene-anchor-link` も触る。

### 選択肢 B: 新しいモジュールに出口と形を置く

- 例: `pasta/failure.lua`（名前は設計で決める）。形の定義（種類の名前・並び・枠）、DSL 表記への変換、エスケープ、切り替えの読み取り、ログへの出力を持つ。act を受けて積む関数と、act を受けずに文字列とログだけを返す関数（要件 8.2）の 2 つを出す。
- `act:failure` はこのモジュールへの薄い入口として残す（Q13 の互換）。`word.lua`・`actor.lua` からも require できる（`act` を require しないので循環しない。`word.lua` が `dynamic_key` を持つ理由と同じ）。
- `ACT.num`・`ACT.str` の「実行中の act」もこのモジュールが持てる（R1）。
- 利点: 形と規則が 1 ファイルにまとまり、`runtime-error-balloon` がそのまま使える。`act.lua` は呼び出しの置き換えだけで済み、共有接点の差分が小さい。単体テストを分けやすい。
- 欠点: ファイルが 1 つ増える。`pasta_scripts` の自己展開 zip・luacheck の対象に入れる必要がある（既存の手順どおり）。

### 選択肢 C: 段階的（抽出 → 新モジュール → 載せ替え）

- 1 段目: `act:failure` の中身を新モジュールへ挙動を変えずに抽出（Call の表記は同じ文字列のまま。テスト不変）。
- 2 段目: 形の定義・表記の変換・エスケープ・切り替えを新モジュールに入れ、Call の表記を新しい形へ（期待値を更新）。
- 3 段目以降: 種類ごとに載せ替え（`talk`・`word`/`expr_fn`/`global_fn`（act と proxy）・`dynamic_key`・`actor_proxy`・`num`/`str`・`call(nil)`）。1 種類 = 1 検証 = 1 コミット。
- 最後: 重ねない規則の確認、未登録アクターの立ち位置の警告、マニュアルとスキルの再生成。
- 利点: プロジェクトの規則（抽出を先に・1 抽出 = 1 検証 = 1 コミット）に合う。途中で止めても main を壊さない。
- 欠点: タスク数が多くなる（下の規模）。

## 4. 調べが要る点（Research Needed）

- **R1 実行中の act の見つけ方（`ACT.num`・`ACT.str`）**: 生成 Lua を変えずに表記を積む方法。候補は (a) コルーチンから act を引く弱参照の表（`coroutine.running()` をキーにする。`SCENE.co_exec` の `wrapped_fn` で登録。ただし `shiori/event/choice_select.lua` の `create_scene_coroutine` と `callback.lua` も独自にコルーチンを作るので、範囲外のファイルに触れずに済むか確かめる）、(b) 「最後に作った act」（チェイントークの再開では古い act に積まれるため誤る。1.3）、(c) 失敗を保留して、次に呼ばれた act のメソッドが自分のトークン列へ流す（位置がずれる可能性。式だけの代入行の後に発言が無い場合の扱い）。Lua ブロックからコルーチンの外で `PASTA.num` を直接呼ぶ場合の落とし方も決める。
- **R2 未登録アクターの目印のトークン**: `raw_script` にするとアクターのグループを開かず、目印が前のアクターのバルーンに入る。目印をそのアクターの発言の先頭に保ちつつウェイト・改行を入れないには、アクター付きで字のある発言とみなさない積み方が要る。`group_by_actor`（`act.lua`）だけで済むか、`sakura_builder.lua` の分類も要るかを確かめる（後者は Q9 と同じく `pasta/shiori/` 配下）。
- **R3 失敗表記のエスケープ**: さくらスクリプトの `\` のエスケープ（`\\`）に加えて、SSP が本文で展開する `%` 系の環境変数の扱い。UKADOC で確かめる。現行の Call の表記はエスケープしていないので、Call の表記も変わる。
- **R4 設定の置き場所**: `[ghost]` に 1 キー（`GhostConfig` と `apply_shiori_defaults` に足し、マニュアルは「SHIORI デフォルト有」）か、新しいセクションか。型が合わない値を既定値に戻す処理を Rust（`GhostConfig` の型検査）と Lua（`pasta.config.get` の後の型検査）のどちらに置くか。
- **R5 既定で表記を出す場合のテストへの影響**: 「未定義なら空」をバルーンの出力で確かめている既存テスト（Lua・Rust・`pasta_shiori` の結合テスト・hello-pasta の検証）の洗い出し。`pasta_shiori/tests/` は同じ時期の `shiori-test-support-runtime` も整理するので、期待値の更新が重ならないか確かめる。
- **R6 DSL 表記への変換の網羅**: `var_path` の形（`var.x`・`save.x`・`args[n]`、プロパティ参照 `＄％prop.path` の経路があるか）と、`operand_desc` の形（`@名前()`・`@*名前()`・`@$パス()`、括弧で包んだ式）をすべて洗い出す。
- **R7 毎秒・通知系イベントでの表記だけの出力**: 失敗表記だけで出力が非 nil になると、`OnSecondChange` などで書き間違いがあると毎回バルーンが出る（Q10）。`virtual_dispatcher.lua` の経路での影響を確かめる。

## 5. 規模と危険度

- **規模: L（1〜2 週間）**。Lua 側の一本化と載せ替えで約 15〜17 タスク、切り替えの設定を足して約 17〜19 タスク（brief の見積りと一致）。内訳の目安: 出口の抽出 1、形・表記の変換・エスケープ 2〜3、載せ替え 6〜7（`talk`、`word`/`expr_fn`/`global_fn` の act と proxy、`dynamic_key`、`actor_proxy`、`num`/`str`、`call(nil)`）、重ねない規則と立ち位置の警告 1〜2、切り替えの設定 2、act 無しの報告 1、マニュアルとスキル 2、テストの期待値更新 1〜2。
- **危険度: Medium**。既存の型（`act:failure`・`raw_script`・`pasta.config.get`）の延長でほぼ書けるが、(1) `ACT.num`・`ACT.str` に act を届ける方法（R1）が生成 Lua を変えない制約の下で未確定、(2) 既定で表記を出すと空で展開していた全箇所の出力が変わる破壊的変更で、テストの期待値の更新が広い、(3) 未登録アクターの目印の性質の変更（R2）が段落区切りの判定に触れる。

## 6. 設計フェーズへの推奨

- **推奨の形**: 選択肢 C（段階的）で、到達点は選択肢 B（新しいモジュールに出口・形・切り替え・act 無しの報告を置き、`act.lua`・`actor.lua`・`word.lua` は呼び出しの置き換えだけ）。理由: プロジェクトの「抽出を先に、1 抽出 = 1 検証 = 1 コミット」に合い、共有接点の `act.lua` の差分が小さく、`runtime-error-balloon` がそのまま使える。
- **設計で決めること**:
  - 出口の引数（種類・対象・説明）と、形の定義の持ち方（種類ごとの名前の表）。
  - R1 の方式。まず生成 Lua を変えない側（コルーチンから act を引く表など）で考える（brief の約束）。変えるなら `scene-attribute-store` を先に main へ入れ、rebase で合わせる。
  - R2 の積み方と、Q9 の `sakura_builder.lua` を触るかどうか。
  - `WORD.dynamic_key` を「理由を返す」形に変えるときの、Lua から直接呼んでいる箇所（マニュアル `lua/script-api.md` 558〜567 に公開の関数として載っている）との互換。
  - マニュアルの 1 か所の置き場所（新しいページなら GENERATION_MAP に足すか）。候補は `reference/` の下の新しいページ、または `reference/startup.md` §4 の隣の節。
- **持ち越す調べ**: R1〜R7。
