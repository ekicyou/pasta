# ギャップ分析: actor-proxy-act-delegation

- 実施日: 2026-10-04
- 対象: `requirements.md`（仮定 A1・A2 付きの草案）と現行 main（34afab28）
- パスは特記なければ `crates/pasta_lua/pasta_scripts/pasta/` 基準

## 1. 分析サマリー

- 500 の原因はすべて 1 か所に集まる。`PROXY_IMPL` の `call_expr`（`actor.lua` 176–184 行付近の `handler(self, ...)`）と `word`（235–236 行付近の `handler(self)`）が、どの段で見つかった関数にもプロキシを渡している。`GLOBAL.yield`・`close_ghost` だけでなく、act のメソッド（3 段目）とシーン関数（1・2・5 段目）もプロキシを受けて壊れる。
- 関数呼び出しの形（`＠名前（…）`・`＠＄x（…）`、expr モード）では、アクターの段（A1・A2）は探されない（`find_actor_handler` は word モード限定）。そのため案 1 を採ると、アクション行の関数呼び出しは常に ACT を受け取る。プロキシを受け取るのは、単語参照の形（`＠名前`・`＠＄x`）でアクターの表のフィールドに置いた関数だけになる。
- 案 1 は `actor.lua` だけで実装できる。案 2（組み込み関数の側で正規化する）では、静的な `＠yield` が 3 段目の `ACT_IMPL.yield`（`act.lua`）で見つかるため直らない。直すには `act.lua` に手を入れる必要があり、並走条件に反する。
- 案 1 を採ると新しい問題が出る。act のメソッドはメソッドチェーン用に `self`（ACT）を返すため、生成コードの `talk((…))` が `table: 0x…` を台詞にしてしまう。要件 1.5・2.5（仮定 A2）の戻り値の扱いを、プロキシ側で一緒に決める必要がある。
- 作者に見える変更が 2 つある。1 つは `GLOBAL` 関数などがプロキシでなく ACT を受け取るようになること。もう 1 つは、既存のテスト 3 件がプロキシを受け取ることを固定していることである。規模は S、リスクは Low〜Medium。

## 2. 現状の調査

### 2.1 名前の解決の流れ（アクション行）

- 生成コード（`crates/pasta_lua/src/code_gen/element_gen.rs` 360–465 行付近）は次の形を出す。
  - `＠名前` → `act:actor_proxy("A"):talk(act:actor_proxy("A"):word("名前"))`
  - `＠名前（…）` → `act:actor_proxy("A"):talk((act:actor_proxy("A"):expr_fn("名前", …)))`
  - `＠＄x` → `…:word(var.x, "var.x")`
  - `＠＄x（…）` → `…:expr_fn_var(var.x, "var.x", …)`
  - `＠＊名前（…）` → `act:global_fn(…)`（ACT を渡す。プロキシを経由しない）
- `PROXY_IMPL.find_handler`（`actor.lua` 160–168 行付近）は 2 段階で探す。
  1. `find_actor_handler`。word モードだけで、A1 はアクターの表 `self.actor[key]`、A2 はアクター単語辞書（`search_word(key, "__actor_名前__")`。文字列を返す）。
  2. 見つからなければ `self.act:find_act_handler`（`act.lua` 320–361 行付近）。L1 はシーンテーブル、L2 はローカル辞書またはローカルシーン、L3 は act のメソッド（`skip_methods` のときは飛ばす）、L4 は `GLOBAL`、L5 はグローバル辞書またはグローバルシーン。
- 見つかった関数は、`call_expr` では `handler(self, ...)`、`word` では `handler(self)` で呼ばれる。`self` はプロキシである。
- プロキシが持つのは `actor`・`act` フィールドと、`talk`・`sakura_script`・`find_actor_handler`・`find_handler`・`word`・`expr_fn`・`expr_fn_var` だけである。`__index` は `PROXY_IMPL` だけを引き、act へは委ねない。

### 2.2 壊れる経路（コードを読んで確認したもの）

| アクション行の記述 | 見つかる段 | 呼ばれ方 | 壊れる箇所 |
| --- | --- | --- | --- |
| `＠yield`・`＠yield（）` | L3 `ACT_IMPL.yield`（`act.lua` 574 行付近） | `yield(proxy)` | `self:build()` が nil の呼び出しになる |
| `＠チェイントーク`・`＠チェイントーク（）` | L4 `GLOBAL["チェイントーク"]` | `GLOBAL.yield(proxy)` | `act:yield()` が nil の呼び出しになる |
| `＠ゴースト終了`・`＠ゴースト終了（ms）` | L4 `GLOBAL["ゴースト終了"]` | `close_ghost(proxy, ms)` | `act:wait`・`act:raw_script` が nil の呼び出しになる |
| `＠＄x`・`＠＄x（）`（x が `yield`） | L3 を飛ばし、L4 の `GLOBAL.yield` | `GLOBAL.yield(proxy)` | 同上 |
| `＠wait（500）` などの act メソッド | L3 | `ACT_IMPL.wait(proxy, …)` | `self.token` が nil |
| `＠get_property（…）` などの SHIORI_ACT メソッド | L3（`shiori/act.lua`） | `get_property(proxy, …)` | `self.token`・`self:build()` |
| `＠ローカルシーン名（）` やシーンテーブル上の関数 | L1・L2・L5 | `SCENE.xxx(proxy, …)` | 生成されたシーン関数の先頭の `act:init_scene(SCENE)` |

- 行の外での同じ呼び出し（`act:word`・`act:expr_fn`・`＞名前`・`＠＊名前（…）`）は、すべて ACT を渡しており動く。
- 既存のテストは、アクション行から組み込み関数を呼ぶ形を 1 つも含まない。チェイントークのテストは `＞チェイントーク`（Call）だけである（`global_chaintalk_*_test.lua`・`runtime/syntax_test.rs`・`scene_kick_*_e2e_test.rs`）。

### 2.3 戻り値の問題（案 1 で表に出る）

- `ACT_IMPL.yield`・`wait`・`raw_script`・`talk` など、act のメソッドの多くは `self`（ACT）を返す。
- 生成コードは戻り値をそのまま `talk` に渡し、`ACT_IMPL.talk` は nil 以外を `tostring` で台詞にする。案 1 で ACT を渡すと、`さくら：＠yield` は再開後に `table: 0x…` を台詞にしてしまう。
- `GLOBAL.yield`・`close_ghost` は nil を返すので問題ない。ただし静的な `＠yield` は L3 の `ACT_IMPL.yield` で見つかる（L4 より先）。
- 現在は、この経路はその前にエラーで止まるため表に出ていない。
- `act.lua` を変えずに防ぐには、プロキシ側の後処理（`call_expr`・`word`）で「戻り値が `self.act` またはプロキシそのものなら nil にする」必要がある（仮定 A2）。

### 2.4 規約・関連資産

- テストの置き場所:
  - Lua の単体テストは `crates/pasta_lua/tests/lua_specs/*_test.lua`（`lua_test` フレームワーク、`describe`/`test`/`expect`）に置く。
  - SHIORI 経由の E2E は `crates/pasta_shiori/tests/*_e2e_test.rs` に置き、フィクスチャのゴーストと `ShioriTestEnv` を使う。`codegen_runtime_safety_e2e_test.rs` と `fixtures/codegen_runtime_safety/` が「500 にならない」ことを固定する手本である。
  - 複数回の応答に分かれるチェイントークは `scene_kick_multibeat_e2e_test.rs` が手本になる。
- 既存のテストのうち、プロキシを受け取ることを固定しているもの（規則を変えると期待が変わる）:
  - `lua_specs/actor_module_test.lua` 67–119 行付近。proxy `expr_fn` のハンドラーがプロキシを受け取る。A1 の `word` の部分はそのまま残る。
  - `lua_specs/act_dynamic_ref_test.lua` 528 行付近。シーンテーブルの関数（L1）がプロキシを受け取る。
  - `lua_specs/act_runtime_safety_test.lua` 210 行付近。未登録アクターの行の `＠関数（）` が `GLOBAL` の関数に `proxy` を渡し、`proxy.actor.name` を使う。
  - `act_dynamic_ref_test.lua` 472 行付近（アクターのフィールドの関数がプロキシを受け取る）は、案 1 でも変わらない。
- マニュアルの該当箇所:
  - `book/src/grammar/variables.md` 177・179 行。
  - `book/src/lua/script-api.md`：37 行（init_scene の注記）、192–208 行（アクタープロキシの表）、222 行、255 行、351 行、538 行。
  - `book/src/internals/internal-modules.md`：175–221 行（PROXY パターン、221 行に「第 1 引数は ACT ではなくプロキシ」）。
- 生成スキル:
  - `node book/tools/gen-skill-refs.mjs` で再生成し、`--check` で食い違いを検査する（CI は `.github/workflows/manual.yml`）。
  - 対象は、`variables.md` から作る `pasta-ghost-authoring/references/variables.md` と、`script-api.md`・`internal-modules.md` から作る `pasta-lua-coding/references/` である。
  - `book/AUTHORING.md` は同じコミットでの再生成と `node book/tools/link-check.mjs` の成功を求める。
- 上流 spec の状況:
  - `dsl-codegen-runtime-safety`（完了、#60）は、プロキシ取得口 `ACT_IMPL.actor_proxy` を `act.lua` に足した（brief の想定と違い、`actor.lua` ではない）。その design は 1.4 で「定義済みのグローバル関数は act を第 1 引数に受け取る」とし、このプロキシの問題を本 spec に送っている。
  - `scene-search-key-normalization`（完了、#57）は `actor.lua` を変えていない（Rust 側の `search_word` を正規化した）。brief の「`actor.lua` を先に触る」は結果として当たらず、衝突はない。
  - `act-token-grouping-fix`（未着手）は `act.lua` のグループ化だけを持つ。`ACT_IMPL.yield` を変える予定はない。

## 3. 要件と資産の対応

| 要件 | 関係する資産 | ギャップ |
| --- | --- | --- |
| 1.1–1.4、1.6、1.7 | `actor.lua` の `call_expr`・`word`、`global.lua`、`shiori/entry.lua` | **Missing**: act の段で見つかった関数に ACT を渡す経路がない |
| 1.5、2.5 | `actor.lua` の後処理、`act.lua` のメソッドの戻り値 `self` | **Missing**: 戻り値が ACT やプロキシのときの扱いがない。**Constraint**: `act.lua` の戻り値は変えられない |
| 2.1–2.3 | `PROXY_IMPL.find_handler`（どの段で見つかったかを返さない） | **Missing**: 見つかった段が呼び出し側に伝わらない。**Constraint**: `find_handler` はマニュアル（script-api 255 行）に載る公開の形 |
| 2.4 | 生成されたシーン関数の `act:init_scene` | ACT を渡せば解消する（追加のギャップなし） |
| 2.6、3.1–3.5 | 既存の検索と後処理 | 変えない。回帰テストで固定する |
| 4.1–4.5 | `lua_specs`、`pasta_shiori` の E2E フィクスチャ | **Missing**: アクション行から組み込み関数を呼ぶテストがない。既存の 3 件は期待を変える |
| 5.1–5.4 | マニュアル 3 章、`gen-skill-refs.mjs` | **Missing**: 規則の書き換え。**Constraint**: 同じコミットで再生成と link-check |
| 並走条件 | `crates/pasta_shiori/tests/support/scripts/pasta/{actor,act,global}.lua`（スクリプトの写し） | **Unknown**: 写しを使うテストが `actor.lua` の変更に追従する必要があるか（Research Needed） |

## 4. 実装の選択肢

### 案 A: プロキシの後処理で、見つかった段に応じて渡すものを変える（brief の案 1）

- `actor.lua` の `call_expr`・`word` で、まず `find_actor_handler` を、見つからなければ `self.act:find_act_handler` を呼ぶ。前者で見つかった関数には `self` を、後者で見つかった関数には `self.act` を渡す。
- 公開の `find_handler` の戻り値（ハンドラー 1 つ）は変えない。中で 2 つの検索を分けて呼ぶか、内部用のローカル関数に分ける。
- 戻り値が `self.act` またはプロキシそのものなら nil にする（仮定 A2）。
- `global.lua`・`shiori/entry.lua` は変更なし（防御のための正規化を足すかは任意）。
- ✅ `actor.lua` だけで、2.2 の表の全経路（act メソッド・SHIORI_ACT メソッド・シーン関数を含む）が直る。
- ✅ `＠＊名前（…）`・行の外の呼び出しと受け取るものがそろう（`GLOBAL` 関数の規則が 1 つになる）。
- ❌ 作者に見える変更になる。マニュアルの「アクション行の関数はプロキシを受け取る」を書き換える。
- ❌ 作者の `GLOBAL` 関数は、アクション行から呼ばれても話者（プロキシの `actor`）を知る手段を失う。話者を使いたい関数は、アクターの表に置く（単語参照の形でだけ届く）しかない。

### 案 B: 組み込み関数の側でプロキシを ACT に正規化する（brief の案 2）

- `GLOBAL.yield`・`close_ghost` の先頭で、引数がプロキシなら `.act` に替える。
- ✅ 作者の関数が受け取るもの（プロキシ）と、マニュアルの記述は変わらない。
- ❌ 静的な `＠yield`・`＠yield（）` は L3 の `ACT_IMPL.yield` で見つかるため直らない。直すには `act.lua` の変更が要り、並走条件に反する（`act-token-grouping-fix` と順序を調整する必要がある）。
- ❌ act メソッド・SHIORI_ACT メソッド・シーン関数の経路は 500 のまま残る。規則は「組み込み関数だけ例外」になり、1 つの規則にならない。

### 案 C: 混合（案 A を主として、組み込み関数に防御を足す）

- 案 A に加えて、`GLOBAL.yield`・`close_ghost` が手書き Lua からプロキシを渡されても動くよう正規化を足す。
- ✅ 手書きの Lua（`GLOBAL.yield(act.さくら)` など）にも耐える。
- ❌ 正規化の要否は要件で求めていない（YAGNI）。足すかどうかは設計で決める。
- 変種として、作者の関数はプロキシのまま、組み込み関数と act メソッドだけ ACT にする案もある。ただし「どれが組み込みか」の印が要り、`＠＊名前（…）` との食い違いも残るため、1 つの規則という目的に合いにくい。

## 5. 規模とリスク

- **規模: S（1〜3 日）**。変更は `actor.lua` の 2 関数と小さな補助、Lua テストの追加と 3 件の期待の修正、E2E フィクスチャ 1 つ、マニュアル 3 章と再生成に収まる。
- **リスク: Low〜Medium**。手法はよく知った形で、検索順序は変えない。一方で、作者の関数が受け取るものが変わる（後方互換に影響する）。プロキシを前提に `GLOBAL` 関数を書いた既存のゴースト（`p.actor.name`・`p:talk(…)` を使うもの）は動かなくなる。

## 6. 設計フェーズへの申し送り

- **推奨（情報として）**: 案 A。並走条件（`actor.lua`・`global.lua`・`shiori/entry.lua` に限る）の中で全経路を直せる唯一の案で、`＠＊名前（…）` との整合も取れる。最終判断は要件ディスカッションで行う。
- **決めること**:
  - 見つかった段を後処理に伝える方法。`find_handler` の公開の形を保つ（戻り値を増やすか、内部で 2 つの検索を分けるか）。
  - 戻り値の正規化の範囲（`self.act` とプロキシだけか、テーブル全般か）。
  - `GLOBAL.yield`・`close_ghost` に防御の正規化を足すか。
- **Research Needed**:
  - `crates/pasta_shiori/tests/support/scripts/pasta/` の写し（`actor.lua` を含む）を使うテストが、本番の `actor.lua` の変更に追従する必要があるか。
  - アクション行の `＠yield` で中断・再開したとき、同じ行の残り（`talk` の外側の呼び出し）が再開後に正しく出るか。中断は `word` の中で起き、戻り値は再開後に `talk` に渡る。E2E で確かめる。
  - 既存のサンプルゴースト・ドキュメントの例に、アクション行から `GLOBAL` 関数がプロキシを使う例が無いか（あれば同時に直す）。

## 7. 再検証の記録（2026-10-04、要件ディスカッション中）

現行 main のランタイムで、アクション行の生成コードと同じ呼び方（`act:actor_proxy("さくら")` のプロキシの `word`・`expr_fn`・`expr_fn_var`）をコルーチンの中で実行して確かめた。試作は確認後に破棄した。

- **不具合の再現**: `＠yield`・`＠yield（）`・`＠チェイントーク`・`＠ゴースト終了`・`＠ゴースト終了（500）`・値が `yield` の `＠＄x`・値が `ゴースト終了` の `＠＄x（300）`・`＠wait（500）` の 8 経路が、すべて 2.2 の表のとおりのエラーになった。
- **案 A の試作**: `actor.lua` の `call_expr` と `word` だけを約 10 行変えた（act の段で見つかった関数に `self.act` を渡し、戻り値が `self.act` そのものなら nil にする）。8 経路すべてが直った。
  - `前＠yield後` は、1 回目の応答が「前」、再開後の応答が「後」になった（行の途中の中断と再開は正しく動く。`table: …` は出ない）。
  - `＠ゴースト終了（500）` は「前・待ち 500・`\-`・後」の順に積まれた。
  - `GLOBAL` の関数は、単語参照の形でも関数呼び出しの形でも ACT を受け取った。
- **見つかった段の伝え方**: 関数呼び出しの形（expr モード）ではアクターの段を探さないため、`call_expr` は常に `self.act` を渡せばよい。`word` だけが `find_actor_handler` と `find_act_handler` を分けて呼ぶ。公開の `find_handler` は変えずに済む。
- **既存のテスト**: 試作で落ちたのは 2.4 に挙げた「プロキシを受け取ることを固定しているテスト」だけである（最初に落ちたのは `actor_module_test.lua` の expr_fn の後処理のテスト）。
- **テスト用スクリプトの写し**: `crates/pasta_shiori/tests/support/scripts/pasta/` は古いランタイムの写しで、本番の `actor.lua` とは既に違う。`fixtures/codegen_runtime_safety/pasta.toml` が本番のランタイムを使う方法を示しているので、E2E はその形にならう（写しは追従させない）。
- **マニュアルの該当箇所の追加**: 受け取るものに触れる記述は `grammar/words.md` 303 行（動的関数呼び出し）と `grammar/actor-dictionary.md` 255 行（アクターの関数。案 A でも記述は変わらない）にもある。要件 5.1 を「触れるすべての箇所」に直した。

## 8. 設計フェーズの調査と判断（2026-10-04）

- **Discovery の種類**: Extension（既存のプロキシの後処理の変更）。軽量の調査（統合点・既存のパターン・テストとマニュアルの影響範囲）を行った。外部の依存・新しいライブラリは無い。

### 8.1 調査の記録

- **第 1 引数を固定している既存テストの追加の発見**
  - 2.4 に挙げた 3 件のほかに、`crates/pasta_lua/tests/runtime/syntax_test.rs` の「アクター付きの行の `＠＄f（１）` は、関数の第 1 引数にアクターのプロキシを渡す」（シーン `＊プロキシ`、`SCENE.whoami(p, n)` が `p.actor.name` を使う）が、アクターの外の関数がプロキシを受け取ることを固定している。期待を変えるテストは 4 件になる。
  - 2.4 の `act_dynamic_ref_test.lua` 528 行付近は、`expr_fn_var` がシーンテーブルの関数にプロキシを渡すことを固定するテストである。
  - `proxy_find_handler_test.lua`・`literal_fixes_test.rs`・`scene_test.rs`・`fallback_search_integration_test.rs` は、検索と文字列の単語だけを確かめており、影響を受けない（読んで確認した。実行はしていない）。
- **サンプル・マニュアルの例**
  - `book/src` と `crates/pasta_sample_ghost` に、アクション行から `GLOBAL` 関数・シーンの関数がプロキシを使う例は無い。プロキシを使う例は `grammar/actor-dictionary.md` の `function ACTOR.自己紹介(proxy)` だけで、これはアクターの段の関数であり、規則を変えても正しい。
- **マニュアルの該当箇所（設計時の全文検索）**
  - 受け取るものを述べる記述: `grammar/variables.md` 177・179 行、`grammar/words.md` 194・303 行、`grammar/actor-dictionary.md` 255 行、`lua/script-api.md` 37・200–202・538 行、`internals/internal-modules.md` 213・217・221 行。
  - `lua/script-api.md` 239・267・283 行は「アクション行ではプロキシの `word`・`expr_fn`・`expr_fn_var` を使う」という経路の説明で、規則を変えても正しい。
  - 組み込みの呼び出しの説明は `grammar/call-jump.md` 168–200 行にある（`＞チェイントーク`・`＞yield`・`＞ゴースト終了`）。要件 5.2 の記述の置き場所の候補である。
  - 上の章はすべて `gen-skill-refs.mjs` の対象である（`pasta-ghost-authoring`・`pasta-lua-coding`）。
- **E2E の手本**
  - 500 にならないことと応答の一致: `codegen_runtime_safety_e2e_test.rs`（`ShioriTestEnv`・シーン名をイベント ID にしてシーン関数フォールバックで起動）。
  - 継続トークの残り: `scene_kick_multibeat_e2e_test.rs`（OnSecondChange と `X-Pasta-Time`・固定のトーク間隔）。フィクスチャでトーク間隔を固定する書き方は `fixtures/async_callback/pasta.toml` にある。
- **設計時の再試作**
  - 設計で足した 2 点（下の 8.2）を確かめるため `actor.lua` を一時的に書き換えて試そうとしたが、実行環境の権限で止められたため行っていない。7 節の試作の結果はそのまま有効で、足した 2 点は実装の最初のタスクで確かめる。

### 8.2 設計判断

#### Decision: 見つかった段の伝え方

- **Context**: 後処理が「アクターの段で見つかったか、act の段で見つかったか」を知る必要がある（2.1・2.2）。`find_handler` はマニュアルに載る公開の形である。
- **Alternatives Considered**:
  1. `find_handler` の戻り値を増やす（ハンドラーと段）。
  2. `word` が `find_actor_handler` と `self.act:find_act_handler` を自分で順に呼ぶ。`call_expr` は常に `self.act` を渡す。
- **Selected Approach**: 2。expr モードはアクターの段を探さないため、`call_expr` は検索を変えずに渡すものだけを変える。
- **Rationale**: 公開の形を変えずに済み、変更が最も小さい（7 節の試作と同じ）。
- **Trade-offs**: `word` の中に `find_handler` と同じ「アクターの段 → act の段」の順序がもう 1 か所現れる。2 行であり、共通化はしない。

#### Decision: 戻り値の正規化の範囲

- **Context**: act のメソッドはメソッドチェーン用に ACT を返す。要件 2.5 は「ACT またはアクタープロキシそのもの」を値なしにする。
- **Alternatives Considered**:
  1. `self.act` と同一のときだけ（7 節の試作）。
  2. `self.act` または `self`（呼び出しに使ったプロキシ）と同一のとき。
  3. テーブル全般、またはメタテーブルで判定したプロキシ全般。
- **Selected Approach**: 2。先頭の戻り値だけを見て、該当すれば `nil` だけを返す。該当しなければ戻り値をすべてそのまま返す。
- **Rationale**: 要件 2.5 の文言（ACT・プロキシそのもの）を満たす最小の判定である。3 は要件が求めておらず、作者が意図して返したテーブルの扱い（現行は `tostring`）を変えてしまう。
- **Trade-offs**: 関数が別に作ったプロキシを返した場合は対象にならない（設計ディスカッションの確認事項）。

#### Decision: 組み込み関数の側の防御は足さない

- **Context**: 並走条件は `global.lua`・`shiori/entry.lua` の変更を許している（4 節の案 C）。
- **Selected Approach**: 足さない。`global.lua`・`shiori/entry.lua` は変更しない。
- **Rationale**: 生成コードの経路はすべてプロキシの後処理を通り、そこで ACT になる。手書き Lua が組み込み関数にプロキシを直接渡す使い方は、要件に無い。

### 8.3 統合（Synthesis）の結果

- **Generalization**: 要件 1（組み込み関数）・2.4（シーン関数）・act のメソッドの呼び出しは、すべて「アクターの外で見つかった関数に ACT を渡す」という 1 つの規則の特殊な場合である。組み込み関数ごとの対応は作らない。
- **Build vs. Adopt**: 新しく作るものは局所関数 1 つだけである。検索は既存の `find_actor_handler`・`find_act_handler` を使い、E2E は既存の `ShioriTestEnv` とフィクスチャの形を使う。
- **Simplification**: 段を表す戻り値・印・新しい公開メソッド・設定・互換の切り替えは足さない。`global.lua`・`shiori/entry.lua` の変更は無しにした。

### 8.4 リスクと対処

- プロキシを前提にした作者の `GLOBAL` 関数・シーンの関数が動かなくなる — 要件ディスカッションで受け入れ済み。マニュアルの規則の記述で知らせる。
- マニュアルの記述の取りこぼし — `book/src` を「プロキシ」で検索して残りが無いことを完了の条件にする。
- 継続トークの E2E が時刻に依存する — `X-Pasta-Time` と固定のトーク間隔で決定論にする。
