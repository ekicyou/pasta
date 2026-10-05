# Research: call-execution-correctness

## ギャップ分析（2026-10-05）

要件（`requirements.md`、未承認の起草版）と現行コードの差を調べた。決定ではなく、設計フェーズと要件ディスカッションの材料である。

### 1. 現状の調査

#### 1.1 シーン文脈を持つ場所

| 状態 | 場所 | 書く側 | 読む側 |
| ---- | ---- | ------ | ------ |
| `act.current_scene` | `crates/pasta_lua/pasta_scripts/pasta/act.lua` 190–196 行 `ACT_IMPL.init_scene` | `init_scene` だけ（すべてのシーン関数の先頭で生成コードが呼ぶ。`scope_gen.rs` 265 行） | `ACT_IMPL.find_act_handler`（同 330–371 行）の L1・L2。単語参照・Call・式関数の 3 モード共通 |
| `STORE.last_global_scene` | `pasta/store.lua` 67 行 | `init_scene` だけ（`scene.__global_name__` があるとき） | `pasta/shiori/event/choice_select.lua` 61 行（`SCENE.search(選択 ID, STORE.last_global_scene) or SCENE.search(選択 ID, nil)`） |

- どちらも上書きだけで、戻す処理はどこにも無い。
- `act` はイベントごとに新しく作られる。`STORE.last_global_scene` はイベントをまたいで残る（選択肢を選ぶイベントは、選択肢を出した応答の後に来るため）。

#### 1.2 シーン関数を「途中で」呼ぶ経路

`init_scene` を呼ぶ関数（＝シーン関数）を、シーンの途中から呼べる経路は Call だけではない。

| 経路 | ランタイム | 別のグローバルシーンに届くか | 末尾呼び出しか |
| ---- | ---------- | ---------------------------- | -------------- |
| Call 行（`＞名前`・`＞式`） | `ACT_IMPL.call`（`act.lua` 651–668 行）→ `return handler(self, ...)` | 届く（L5 のグローバルシーン前方一致） | 生成コードが最後の項目にだけ `return` を付ける（`scope_gen.rs` 311–330 行、`element_gen.rs` 239–243 行）。ランタイム側は常に末尾位置 |
| 式の関数呼び出し（`＠関数（…）`・`＠＄変数（…）`） | `call_expr`（`act.lua` 419–427 行）、プロキシ経由は `actor.lua` 186–200 行付近 | 届く（`expr` モードも L2・L5 で `SCENE.search` を使う。マニュアル `grammar/variables.md` 238 行が 5 段目をグローバルシーンと明記） | 式の中なので末尾ではない |
| 単語参照（`＠単語`） | `ACT_IMPL.word`（`act.lua` 391–411 行）。ハンドラが関数なら `handler(self)` | L1 の同じシーンテーブルの関数だけ（L2・L5 は単語辞書）。別のグローバルシーンには届かない | — |
| `＠＊関数（…）` | `ACT_IMPL.global_fn`（489–496 行） | `GLOBAL` の関数が自分で `init_scene` しない限り届かない | — |
| Lua からの直接呼び出し | `act:call(nil, "名前", nil)`（マニュアル `lua/script-api.md` 298–321 行が公開 API として記載） | 届く | 呼ぶ側の書き方しだい |

→ 要件 3（仮定 A3）の根拠。Call 行だけを直すと、式の関数呼び出しの経路に同じ取り違えが残る。

#### 1.3 動的コールの生成形と nil

- `element_gen.rs` 229–235 行: `CallTarget::Dynamic(expr)` は `act:call(SCENE.__global_name__, tostring(<式>), {}, <引数>)`。式は `expr_to_string` の出力そのまま（変数参照は生の `var.x`。`expr_gen.rs` 110–112 行）。
- `ACT_IMPL.call` の先頭の nil ガード（`act.lua` 653–656 行）は、`tostring` を通った値が nil にならないため DSL からは届かない。検証しているテストも Lua からの直接呼び出しだけ（`tests/transpiler/dynamic_call_test.rs` 130–161 行 `test_nil_guard_in_act_call`）。
- `ACT_IMPL.call` の第 1 引数 `global_scene_name` と第 3 引数 `attrs` は現在使っていない（互換のために残してある）。`attrs` は後続の `call-attribute-filter` が使う予定。
- 再利用できる部品:
  - `dynamic_ref_args`（`element_gen.rs` 48–52 行）: `値, "変数の経路"` を渡す。
  - `operand_desc`（`expr_gen.rs` 207–231 行）: 変数参照は経路（`var.x`・`save.x`・`args[1]`）、関数呼び出しは `@名前()`・`@*名前()`・`@$経路()`、リテラルと演算は説明なし。算術・連結の警告が使っている。
  - `WORD.dynamic_key(value, var_path, via)`（`pasta/word.lua` 168–183 行）: nil は `undefined variable: '経路'`、空文字列は `empty variable`、文字列・数値以外は `unsupported value type` の警告を出して nil を返す。nil 以外も弾くため、仮定 A7（型は揃えない）のままだと Call にはそのまま使えない。
  - `ACT_IMPL.talk` の未定義変数の警告（`act.lua` 205–214 行、`act:talk - undefined variable: 'var.x'`）。

#### 1.4 関連するテスト・スナップショット

- 生成形: `tests/transpiler/snapshot_test.rs`（`dynamic_call_local_var`・`dynamic_call_global_var`・`dynamic_call_fn_call`・`dynamic_call_binary_expr`・`scene_with_call`・`multiple_scenes`・`tail_call_optimization`）、`final_regression_test.rs`（`kind_dynamic_call`・`kind_scene_call`・`kind_multiple_scenes`・`fixture_sample`・`fixture_tail_call_optimization`・`fixture_zero_cost_all_syntax`）、`zero_cost_regression_test`、`tests/fixtures/sample.expected.lua`、`src/code_gen/element_gen_tests.rs`・`scope_gen_tests.rs`・`expr_gen_tests.rs`、`tests/transpiler/record_wiring_element_test.rs`・`source_map_seam_test.rs`。
- `tests/transpiler/dynamic_call_test.rs` は生成コードに `tostring(var.target)` が含まれることを直接検証している（44–75 行・199 行付近）。nil の生成形を変えると書き換えが要る。
- ランタイム: `tests/lua_specs/act_impl_call_test.lua`・`act_init_scene_global_record_test.lua`・`choice_select_test.lua`・`act_find_act_handler_test.lua`、`tests/runtime/local_scene_call_test.rs`。
- 別のグローバルシーンへ Call して戻る流れを辞書から通しで検証するテストは見当たらない（**Missing**）。

#### 1.5 マニュアルの該当箇所

| 章 | 箇所 | 現在の記述 | 生成スキル |
| -- | ---- | ---------- | ---------- |
| `book/src/grammar/call-jump.md` | 60 行・243–247 行「任意の式の呼び出しと nil」 | nil は `"nil"` を検索キーにする。ガードは DSL からは起きない | `pasta-ghost-authoring`（`gen-skill-refs.mjs` 39 行） |
| `book/src/lua/script-api.md` | 298–321 行 `call` | nil のキーは検索しない。「呼んだ先のシーン関数が `init_scene` を呼ぶと、戻った後も実行中のシーンは呼んだ先のまま」（315 行） | `pasta-lua-coding`（同 57 行） |
| `book/src/lua/shiori-events.md` | 313 行 | 「直前に実行したグローバルシーン（通常は選択肢を出したシーン）の配下から探す」 | `pasta-lua-coding`（同 55 行） |
| `book/src/internals/internal-modules.md` | 34 行（`last_global_scene`）・147–153 行（`init_scene`）・309 行 | 上書きだけで戻さないこと、Call 後は呼び出し先のものになることを明記 | `pasta-lua-coding`（同 58 行） |
| `book/src/internals/transpiler.md` | 212 行（Call の生成形・`tostring(式)`）・233 行（連結の例）・244–255 行（末尾呼び出し） | 現行の生成形 | 対応表に無い（再生成の対象外） |
| `book/src/internals/shiori.md` 319 行・`execution-model.md` 181・186 行 | 選択肢の検索の親、Call は同じコルーチン内の通常の関数呼び出し | 記述が変わる場合だけ更新 | — |

### 2. 要件と資産の対応

| 要件 | 既存の資産 | ギャップ |
| ---- | ---------- | -------- |
| R1 途中の Call 後の名前解決 | `init_scene`・`ACT_IMPL.call`・`generate_call_scene`（`is_tail_call` を既に受け取る） | **Missing**: 戻す処理。**Constraint**: `ACT_IMPL.call` の `return handler(self, ...)` と生成コードの `return act:call(…)` の末尾呼び出しを壊せない |
| R1.5 中断をはさむ場合 | Call は同じコルーチン内の通常の呼び出し（`execution-model.md` 186 行） | 戻す処理を「呼び出しの後」に置けば、再開後に戻った時点で実行される。中断中に別イベントが `STORE.last_global_scene` を書き換える点は範囲外 |
| R1.9（A1）Lua からの直接呼び出し | `act:call` は公開 API | 生成コード側で戻す案では効かない。ランタイム側で戻す案なら同じ部品を公開できる |
| R2 選択肢の探索範囲 | `choice_select.lua` は `STORE.last_global_scene` を 1 つだけ見る。`act:choice` のトークンは `target`・`display` だけ（`act.lua` 276–279 行） | **Missing**: 戻す処理。**Constraint**: 探索範囲は応答 1 つにつき 1 つしか持てない。呼び出し元と呼ばれた側の両方が選択肢を出すと、どちらか一方のローカルシーンにしか届かない（A2） |
| R3（A3）式の関数呼び出し | `call_expr`（act・プロキシの 2 か所） | **Missing**: 戻す処理。末尾呼び出しではないため、ランタイムで前後を挟める |
| R4 末尾の Call・既存挙動 | `tests/fixtures/tail_call_optimization.pasta` と対応スナップショット | 深さが増えないことを実行で確かめるテストは未確認（**Unknown**。生成形のスナップショットだけの可能性） |
| R5 nil | 到達できないガード、`dynamic_ref_args`・`operand_desc` | **Missing**: 生値を渡す生成形と、説明（変数の経路・関数の表記）を渡す口。**Constraint**: `act:call(global_scene_name, key, attrs, ...)` の引数の並びは公開 API で、`attrs` は後続 spec が使う |
| R6 nil 以外の不変 | 現行は `tostring` 後に `find_handler("scene", key)` | 文字列化の位置を動かすと、真偽値・テーブル・空文字列の扱いが変わらないことの確認が要る（**Research Needed**: 空文字列を検索キーにしたときの現行の結果） |
| R7 マニュアル | 上表 1.5 | 文章の更新と `gen-skill-refs.mjs` の再生成 |
| R8 テスト | 上記 1.4 | 辞書から通しで動かすテストの追加。スナップショット更新 |

### 3. 実装の選択肢

#### 3.1 シーン文脈を戻す位置

**案 A: 生成コードが途中の Call の後に戻す文を出す（brief の推奨）**

- `generate_call_scene` が `is_tail_call == false` のとき、`act:call(…)` の後に `act:init_scene(SCENE)`（または小さな `act:restore_scene(SCENE)`）を出力する。
- ✅ ランタイムの変更が最小。末尾の Call は生成形が変わらない。
- ❌ Call 1 行が Lua 2 文になる。ソースマップは「1 項目 = 出力 1 行」を前提に行と span を対応させているため（`element_gen.rs` 245–249 行）、2 行目の対応づけか、1 行に 2 文を書く形を決める必要がある。デバッガのステップ表示にも影響しうる。
- ❌ Lua から `act:call` を直接呼ぶ場合（A1）と、式の関数呼び出し（A3）には効かない。A3 は式の中なので後ろに文を足す形にできず、別の手当てが要る。
- ❌ `init_scene(SCENE)` で戻すと「呼び出し元の SCENE に設定し直す」動きになる。Lua ブロックの関数が自分で別の文脈を作っていた場合の「元の値に戻す」とは厳密には違う（DSL の行に対しては同じ結果）。

**案 B: ランタイムに「呼んで戻す」口を足し、途中の Call はそれを呼ぶ**

- 途中の Call の生成形を別のメソッド（例: 文脈を保存 → ハンドラを呼ぶ → 文脈を戻す）にし、末尾の Call は今の `return act:call(…)` のままにする。`act:call` 自体は末尾呼び出しのまま変えない。
- ✅ Call 1 行 = Lua 1 行のまま。ソースマップ・デバッガへの影響が無い。
- ✅ 同じ「保存して戻す」部品を `call_expr`（A3）で使える。Lua から直接使う口としても公開できる（A1）。
- ✅ 保存・復元なので、呼び出し前の値に正確に戻る。
- ❌ ランタイムのメソッドが 1 つ増える。途中の Call を含むスナップショットがすべて変わる（案 A でも変わる）。
- ❌ 呼ばれた側がエラーで抜けた場合は戻らない（コルーチンごと失敗するため実害は無い見込み。設計で確認）。

**案 C: `ACT_IMPL.call` の中で常に保存・復元する**

- ❌ `return handler(self, ...)` が末尾位置でなくなり、末尾の Call の連鎖でスタックが深くなる。brief の制約に反するため不採用の見込み。記録のために残す。

#### 3.2 選択肢の探索範囲（A2）

- **単純に戻す**: `last_global_scene` も `current_scene` と一緒に戻す。呼ばれた側が出した選択肢は、呼ばれた側のローカルシーンに届かなくなる（グローバルシーンへのフォールバックだけ効く）。現行では逆に、呼び出し元が出した選択肢が呼び出し元のローカルシーンに届かない。末尾の Call で移った先が出す選択肢は、どちらでも今までどおり届く（共通メニューを最後に呼ぶ書き方は影響を受けない）。
- **選択肢ごとに記録する**: 選択肢を出した時点のグローバルシーンを選択肢ごとに覚え、選んだときにそれを使う。両方のシーンの選択肢が正しく届く。`act:choice` のトークン・さくらスクリプトの組立・`choice_select.lua` に手が入り、brief の Out of Boundary（`choice_select.lua`）に触れる。中規模。
- **名前解決だけ戻す**: `last_global_scene` は現行のまま。R2 を落とすことになり、brief の Desired Outcome と合わない。

#### 3.3 nil の判定の位置（A4 が「ガードを生かす」の場合）

**案 N1: 生成コードは式の値をそのまま渡し、ランタイムが nil を判定してから文字列にする**

- 動的コールの生成形から `tostring` を外し、警告用の説明（変数の経路・関数の表記）を一緒に渡す。ランタイムは nil なら警告して戻り、それ以外は今と同じ文字列化をしてから検索する。
- 説明を渡す口の候補: (a) 動的コール専用のメソッド（`expr_fn_var`・`act:word(値, 経路)` と同じ型。`act:call` の引数の並びを変えない）、(b) 使っていない第 1 引数 `global_scene_name` の再利用、(c) `attrs` に入れる（後続の `call-attribute-filter` と衝突しうる）。
- ✅ 既存のガードが生きる。Call 1 行 = Lua 1 行のまま。末尾呼び出しを保てる（専用メソッドの最後を末尾位置にする）。
- ❌ `dynamic_call_*` のスナップショットと `dynamic_call_test.rs` の `tostring(…)` の検証が変わる。

**案 N2: 生成コードで nil 以外だけを文字列にする**

- 生成コードに nil 判定の式・文を出す。
- ❌ 式を 2 回書くと関数呼び出しが 2 回評価されるため、一時変数と複数行が要る。ソースマップと末尾呼び出しの形が崩れやすい。警告の文言を生成コード側に埋めることになる。

**「現行を正とする」場合**: コード変更は、到達できないガードを残すか消すかだけ（Lua からの直接呼び出し用として残すのが現行のマニュアルと合う）。マニュアルは現状のまま。R5 を置き換える。

#### 3.4 組み合わせ

- 途中の Call 用のランタイムの口（案 B）と動的コール用の口（案 N1-a）は、どちらも「生成コードが呼ぶメソッドを分ける」変更で、`generate_call_scene` の 1 か所（静的／動的 × 末尾／途中の 4 通り）に集まる。後続の `scene-attribute-store`・`call-attribute-filter` も同じ場所を触るため、メソッドの分け方と引数の並びは設計で先に固めたい。

### 4. 規模とリスク

- **規模: S〜M**。仮定どおり（A2 は単純に戻す）なら S（生成 1 か所・ランタイム数十行・テスト・マニュアル 5 章前後）。A2 を「選択肢ごとに記録」にすると M。
- **リスク: Low〜Medium**。既存パターンの延長で外部依存なし。Medium の要因は、スナップショットの広い更新、公開 API（`act:call`）の引数との兼ね合い、A2 の決定が既存の辞書の選択肢の届き方を変えうること。

### 5. 設計フェーズへの申し送り

- **決めること**:
  - 文脈を戻す位置（案 A / 案 B）。ソースマップとデバッガへの影響、A1・A3 をどこまで含めるかで決まる。
  - 選択肢の探索範囲の扱い（3.2）。要件ディスカッションの A2 の結論に従う。
  - nil の説明を渡す口（3.3 の a / b / c）。後続 spec の `attrs` の使い方と衝突しない形。
  - 警告の文言（接頭辞・経路の表記）。既存の `act:talk - undefined variable: 'var.x'`・`act:word - undefined variable: '…'` に合わせる。
- **Research Needed**:
  - 空文字列・真偽値・テーブルを検索キーにしたときの現行の結果（R6.3 を固定するテストの期待値）。
  - 末尾の Call の連鎖で深さが増えないことを実行で確かめる既存テストの有無。無ければ R4.1 用に追加する。
  - 途中の Call の生成形を変えたときの、デバッガのステップ動作（`pasta_lua/src/debug/`）への影響。
  - 呼ばれた側がエラーで抜けた場合に文脈が戻らないことの影響（コルーチンが失敗で終わる前提の確認）。
- **並走条件**: Wave 3 は本 spec だけ。`element_gen.rs`・`act.lua` を先に触った `dsl-codegen-runtime-safety`・`act-token-grouping-fix`・`scene-identity-format` は main に入っている（本ワークツリーの HEAD で確認）。

### 6. 重複セッションの検討内容の統合（2026-10-05）

同じ spec を別セッション（ブランチ `claude/call-execution-correctness-6012ae`、コミット `ecf5ba40`）でも起草していた。そちらの要件・ギャップ分析から、本書に無かった事実と判断を取り込んだ。以後は本ブランチを正とする。

#### 6.1 要件へ取り込んだもの

- U28 の再現手順を受入基準にした（要件 1.11）。
- Call のターゲットが Lua の関数で、その中から別のグローバルシーンが呼ばれる場合（要件 1.10）。
- 呼ばれた側が選択肢を出して中断し、中断中に選ばれる場合は現行どおり呼ばれた側から探す（要件 2.7）。
- 末尾の Call: 連鎖は少なくとも 10 万回、戻り値の透過、生成コードの不変（要件 4.1・4.8・4.9）。
- 演算の結果が nil のときは Call の警告を重ねない（要件 5.6）。`act:talk`・`act:arith`・`act:concat` は、値が nil で説明も無いとき「内側の失敗の伝播」として黙る。この既存の慣行に合わせ、起草時の仮定 A6（重ねて出す）を取り下げた。
- nil に関するどの場合も 500 を返さない（要件 5.11）。
- 破壊的変更は `fix!` とリリースノートで扱う（Boundary Context）。

#### 6.2 追加の事実

- `act:call` の 4 番目以降の引数はすべて呼ばれた側へ渡る。警告用の説明を後ろに足す場所は無い（3.3 の案 N1-a を推す理由）。
- `STORE.last_global_scene` は `STORE` のリセット（`store.lua` 103 行）でも書かれる。テストは `tests/lua_specs/store_last_global_scene_test.lua` にもある。
- スナップショットの数: `act:call(` を含むもの 14 件、`tostring(` を含むもの 6 件、途中の Call（行頭が `act:call(`）を含むもの 3 件（`tail_call_optimization` 2 件と `fixture_sample`）。
- マニュアルの追加の該当箇所: `grammar/call-jump.md` 34・38 行（「値を文字列に変換して検索キーにする」）、`internals/registry-search.md` 220 行（`init_scene` が `current_scene` を設定する）。
- `crates/pasta_shiori/tests/support/scripts/pasta/act.lua` は古い形の別実装（テスト用の支援スクリプト）。変更の対象かどうかは設計で確認する。

#### 6.3 実装の選択肢の追加

- **案 D（3.1 への追加）: 文脈を字句的な引数に変える**。生成コードが名前解決のたびに自分の `SCENE` を渡し、ランタイムは `current_scene` でなく渡された値で 1・2 段目を引く（`act:call` の未使用の第 1 引数はこの形の名残り）。
  - ✅ 上書きという原因そのものが無くなる。式の関数呼び出しの経路（要件 3）も同時に解ける。
  - ❌ `act:word`・`act:expr_fn`・アクタープロキシなど、すべての生成コードと公開 API の引数が変わる。`dsl-codegen-runtime-safety` の領分に踏み込む。規模 L。選択肢の探索範囲は別に扱う必要がある。不採用の見込み。記録のために残す。
- **案 N2 の補足**: 生成コードで nil を判定する形は、末尾の Call の生成形（`return act:call(…)`）を変えるため、要件 4.9 と衝突する。

#### 6.4 Research Needed の追加

- 動的コールの値が空文字列のとき、`SCENE.search("")` が前方一致で任意のシーンに当たる可能性（仮定 A7 の判断材料）。
- 末尾の Call の 10 万回の連鎖を DSL だけで書くテストの形（動的コールと Lua の関数で終了条件を作る）。
- `pasta_sample_ghost` などの同梱の辞書に、戻った後に呼ばれた側のローカルが見えることへ依存した書き方が無いか。
- `crates/pasta_shiori/tests/support/scripts/pasta/act.lua` が Call の経路のテストに使われているか。

### 7. 要件ディスカッションの決定

- **#1 選択肢の探索範囲（2026-10-05）**: 3.2 の「選択肢ごとに記録する」に確定した。規模は M。`act:choice` のトークン・さくらスクリプトの組立・`choice_select.lua` が対象に入る（brief の Out of Boundary を上書き）。設計で決めること:
  - 記録の持ち方（選択 ID と出したグローバルシーンの対応をどこに置き、いつ捨てるか。選択は後の別リクエストで届く）。
  - 1 つの応答で、複数のシーンが同じジャンプ先名の選択肢を出した場合の扱い。
  - `STORE.last_global_scene` の役割（記録の無い選択 ID の探索範囲として残す。要件 2.9）と、途中の Call の後にこれも戻すかどうか。
  - Lua から `act:choice` を直接呼んだ場合に記録するシーン（呼んだ時点のシーン文脈）。
- **#2 式の関数呼び出しの経路（2026-10-05）**: 対象に含める（要件 3）。生成コードは変えず、ランタイムの式の関数呼び出しの口（`call_expr`。act とプロキシの 2 か所）で前後を挟む形を出発点にする。
- **議題にせず閉じたもの（仮定 A1）**: `act:call` 自体は変えない（中で文脈を戻すと末尾の Call が壊れるため、brief の制約から決まる）。Lua から使える「戻す呼び出し口」を公開するかは、3.1 の案の選択に従って設計で決める。
- **#3 nil の方向（2026-10-05）**: 値が nil なら呼ばない（ガードを生かす）。加えて、さくらスクリプトの側でも失敗が分かる表記を出す（開発者の指示）。現行のランタイムに、失敗をさくらスクリプトへ出す仕組みは無い（警告は `log.warn` だけ。未定義の変数・単語・関数は空文字で展開する）。表記の形と範囲は続く議題で決める。
- **#4 失敗表記の形（2026-10-05）**: バルーンに見える文字列を Call 行の位置に出す。設計で決めること: 文言（ログの警告と同じ変数・関数の表記を使う）、どのトークンとして積むか（発言中のアクターのバルーンに出る形）、発言より前・アクター未確定の位置に Call 行がある場合の出し方、マニュアルへの記載場所。
- **後続 spec の起票（2026-10-05）**: 失敗の出力の一本化を `failure-output-unification` として起票した（brief のみ）。本 spec は Call 行の失敗表記だけを持つ。設計では、失敗表記の出力を 1 つの関数にまとめ、後続がそのまま載せ替えられる形にする。アクション行の未定義の参照などへ広げることは後続が持つ。
