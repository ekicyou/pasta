# Requirements Document

## Project Description (Input)
**誰が困っているか**: pasta.dll でゴーストを作る作者と、pasta の Lua ランタイム（`crates/pasta_lua/pasta_scripts`）を読み直す開発者。

**今の状況**: 2026-10-19 の `/ponytail:ponytail-audit` で、この領域に次の指摘が見つかった。pasta.toml の `[ghost].hour_margin` を数でなく文字列（`"30"`）で書くと、話す時刻に毎秒スクリプトのエラーのバルーンが出続ける。Lua の値に改行が入ると SHIORI の応答が割れ、バルーンが切れる。続きのトークの中の `get_property` が、元の要求の時刻から期限を数えるため、すぐ時間切れになる。加えて、シーンのコルーチンを包む関数の写し（`recommend.lua` を足して 5 つ）、LuaJIT では届かない `coroutine.close` の分岐、`STORE.co_callback` の余計な旗、空の関数を包む `xpcall`、素通しの `ActorWordBuilder`、毎秒の `get_config` などが、読む人の手間を増やしている。

**何を変えるか**: 数でない `hour_margin` はほかのキーと同じく既定（30）に戻す。応答の見出しの値の改行は空白にする。続きのトークの期限は呼んだ時刻から数える。写しと死んだ分岐を無くし、振る舞いは変えない。ギャップ分析で同じ領域の監査をやり直し、増えた指摘も残らず直す。詳しくは `brief.md`。

## Introduction

この spec は、pasta の Lua ランタイム（`crates/pasta_lua/pasta_scripts/**`）の監査の指摘を、残らず直す。監査は 2026-10-19 に 1 回走らせ、2026-11-02 に同じ領域でやり直した（やり直しの結果と、控えとの違いは `research.md`）。

指摘には 2 種類ある。

- **挙動を変える直し**: ゴーストの作者に見える不具合を直す。決めごとは、この文書の各要件に書く。マニュアルとスキルの references を、同じ変更で直す。
- **挙動を変えない整理**: 写し・死んだ分岐・素通しの包み・使い手の無い部品を消す。作者に見える振る舞い（応答の文字列・バルーン・警告・セーブ）は 1 バイトも変えない。

下の「監査の指摘の一覧」の番号（指摘 1〜21）を、各要件から指す。1〜9 は brief の控え、10〜21 は 2026-11-02 のやり直しで増えたものである。

### 監査の指摘の一覧

| 番号 | 中身 | 種類 | 要件 |
| ---- | ---- | ---- | ---- |
| 1 | 数でない `[ghost].hour_margin` で、話す時刻に毎秒エラー | 挙動を変える | 1 |
| 2 | 応答の見出しの値の CR/LF で応答が割れる | 挙動を変える | 2 |
| 3 | シーンのコルーチンを包む関数の写し（`scene.lua`・`choice_select.lua`・`external_control.lua`。`kick.lua` の 2 つは `debug-mcp-runtime` がファイルごと消すので除く） | 整理 | 4 |
| 4 | 届かない `coroutine.close` の分岐（`event/init.lua`・`store.lua`）と、余計な旗 `STORE.co_callback` | 整理 | 4 |
| 5 | `SHIORI.load`・`SHIORI.unload` が空の関数を `xpcall` で包む | 整理 | 4 |
| 6 | 素通しの包み `ActorWordBuilder` | 整理 | 4 |
| 7 | 続きのトークの中の `get_property` の期限が、元の要求の時刻から数えられる | 挙動を変える | 3 |
| 8 | 設定の読み直しが毎秒走る・古いコメント（`virtual_dispatcher.lua` の「前 co_scene を閉じる」） | 整理 | 4 |
| 9 | `ACT_IMPL.find_handler` の素通し・`SCENE.register` と `create_global_table` の写し | 整理 | 4 |
| 10 | 名前の引き当てと「見つからない」の表記の部品の写し（`scene_play.lua`・`external_control.lua`・`recommend.lua`） | 整理 | 4 |
| 11 | 何も出さずに区切り続けるシーンで、要求が返らずベースウェアが固まる | 挙動を変える | 5 |
| 12 | 関数を呼んだ後、エラーの表記からローカルシーン名と `.pasta` の行が消える | 挙動を変える | 5 |
| 13 | ACT の段の関数が ACT を返すと、台詞に `table: 0x…` が出る（プロキシの段では出ない） | 挙動を変える | 5 |
| 14 | おすすめメニューの値が `SHIORI/` で始まると、応答全体と取り違えられる | 挙動を変える | 2 |
| 15 | 応答の部品（`RES.ok`・`RES.err`・`RES.warn`）が、呼び手の見出しの表を書き換える | 挙動を変える | 2 |
| 16 | 既定の `main.lua` の使用例が、在しない関数を呼ぶ | 整理 | 6 |
| 17 | 何も確かめない Rust のテスト（`test_ontalk_hour_margin_skip`） | 整理 | 4 |
| 18 | `ACT_IMPL.yield` の不要な遅延 `require`・`fn_succeeded` と `call_expr` の写し（`act.lua`・`actor.lua`）・`co_exec` の重なった検査 | 整理 | 4 |
| 19 | 選ばれることの無い代わりのバッファ（`buf.lua` の FallbackBuffer） | 整理 | 4 |
| 20 | 実行時の使い手の無い `pasta.lua_version` | 整理 | 4 |
| 21 | テストと内部の頁にしか使い手の無い取り出し口（`WORD.get_global_words`・`get_local_words`・`get_actor_words`・`SCENE.get_global_name`） | 整理 | 4 |

## Boundary Context

- **In scope**:
  - 上の指摘 1〜21 の修正と、その振る舞いを固めるテスト（`crates/pasta_lua/tests/lua_specs/**`、`crates/pasta_lua/tests/shiori/**` のうち、この領域を確かめるもの）。
  - 挙動を変える直しに合わせたマニュアルの直し（`book/src/internals/shiori.md` の時計とトークの節、`book/src/lua/shiori-events.md` の設定の表と RES の表、`book/src/reference/pasta-toml.md` の `hour_margin` の行、`book/src/internals/execution-model.md`、`book/src/internals/internal-modules.md`）と、`gen-skill-refs` が作る references。
  - `.kiro/specs/novel-checkpoint-rollback/brief.md`・`.kiro/specs/shiori4-v4-lua-api/brief.md` の `STORE.co_callback` の言及の直し（実装のとき）。
- **Out of scope**:
  - 空の `pasta.areka`（`pasta/areka/init.lua`）。0.6.0 の `novel-areka-contract` の置き場として残す（2026-10-19 開発者の決定）。
  - Rust 側（`crates/pasta_lua/src/**`。`ponytail-lua-loader`・`ponytail-lua-transpiler`・`ponytail-lua-debug` が持つ）。`apply_shiori_defaults` が型を見ないことも、Rust 側は変えない（Lua 側で既定に戻す）。
  - `scriptlibs/`（同梱の luacheck）。
  - `sakura_builder.lua` の書記素の扱い（`talk-grapheme-clusters`）。
  - README と `steering/tech.md` の「pasta の部品で、外部から使う安定した面ではない」の断り書き（`ponytail-lua-loader` が書いた）。
  - 正時の手前でランダムトークを見送ったとき、次のトーク時刻を決め直さない今の決まり（`internals/shiori.md` に書かれている。見送ったトークは、正時の後の最初の空いた秒に出る）。この spec では変えない（決定 1）。
- **Adjacent expectations**:
  - 同じウェーブの `ponytail-shiori` とは、`internals/shiori.md`・`execution-model.md` の別の節を触る。後から入る側が rebase で合わせる。`pasta/act.lua` はこのウェーブではこの spec だけが触る。
  - `chain-talk-interrupt`（完了）が決めた続きの決まりを保つ。台詞を出さなかったシーンは待っている続きに触れない・台詞を出した割り込みと指名した再生とエラーだけが続きを捨てる・中断の台詞は反応の代わりに出る。`EVENT.drive` の手順の順は `chain_talk_interrupt_test.lua` が守る。
  - `ontalk-suppression`（完了）が足した、自動トークを止めている間の門と `restart_timers` の振る舞いを保つ。
  - `reg-handler-chain`（完了）が足した、ハンドラの鎖の振る舞いを保つ。
  - 0.5.0 の属性の 3 本（`scene-attribute-store` ほか）と `scene-count` は、この spec が入った後の `scene.lua`・`act.lua`・`entry.lua` の上に乗る。

## Requirements

### Requirement 1: 数でない `hour_margin` は既定に戻る（指摘 1）
**Objective:** As a ゴーストの作者, I want `[ghost].hour_margin` を書き間違えても、ランダムトークが止まらずエラーも出ないこと, so that 設定の書き間違いでゴーストが毎秒エラーのバルーンを出し続けない

#### Acceptance Criteria
1. If `[ghost].hour_margin` が数でない値（文字列・真偽値・表など）である, the Lua ランタイム shall 正時までの残りを比べるときに、既定の 30 秒を使う。
2. If `[ghost].hour_margin` が数でない値である, the Lua ランタイム shall 毎秒の時刻の知らせ（OnSecondChange）で、スクリプトのエラーの表記を応答に出さない。
3. If `[ghost].hour_margin` が数でない値である, the Lua ランタイム shall ランダムトークを、間隔の設定どおりに出し続ける。
4. If `[ghost].hour_margin` が NaN または無限大である, the Lua ランタイム shall 既定の 30 秒を使う（`talk_interval_min`・`talk_interval_max` と同じく、NaN と無限大は数として扱わない。ASSUMPTION A1）。
5. When `[ghost].hour_margin` が有限の数である, the Lua ランタイム shall その値を、切り捨てや下限の補正をせずにそのまま使う（今の決まりを保つ）。
6. When `[ghost].hour_margin` が書かれていない, the Lua ランタイム shall 今と同じく 30 秒を使う。

### Requirement 2: 応答の組み立てが割れない（指摘 2・14・15）
**Objective:** As a ゴーストの作者, I want Lua の値や単語に改行が入っても SHIORI の応答が割れないこと, so that バルーンが途中で切れず、ベースウェアが応答を読み違えない

#### Acceptance Criteria
1. When 応答の見出しの値に CR または LF が含まれる, the Lua ランタイム shall その CR と LF を、1 文字ごとに半角の空白 1 つに置き換えてから応答を組み立てる（CR LF の組は空白 2 つになる。失敗の表記の `FAILURE.escape` と同じ置き換え。ASSUMPTION A2）。
2. The Lua ランタイム shall 改行の置き換えを、すべての応答（`RES.ok`・`RES.err`・`RES.warn`・204・作者が見出しの表を渡す応答・停止とエラーの応答）に等しく効かせる。
3. When 応答の見出しの値に CR と LF が含まれない, the Lua ランタイム shall 今と 1 バイトも違わない応答を返す。
4. When `act.var` の値や単語の値に改行が入っていて、それがトークの値に入る, the Lua ランタイム shall `Value` の見出しを 1 行のまま返し、末尾の `\e` を失わない。
5. When おすすめメニュー・ポータルメニューの項目の名前や URL、キャプションの単語が `SHIORI/` で始まる, the Lua ランタイム shall それを `Value` の値として返し、応答全体として扱わない。
6. When 作者が見出しの表を `RES.ok`・`RES.err`・`RES.warn` に渡す, the Lua ランタイム shall 渡された表の中身を書き換えない（同じ表を後の応答に使い回しても、前の `Value` や理由の見出しが混ざらない）。
7. The Lua ランタイム shall 見出しの名前と並びの順を、今と同じにする（見出しの名前の中の改行は、この spec では扱わない。ASSUMPTION A3）。

### Requirement 3: 続きのトークの `get_property` の期限は呼んだ時刻から数える（指摘 7）
**Objective:** As a ゴーストの作者, I want チェイントークの続きの中で `get_property` を呼んでも、すぐに時間切れにならないこと, so that 続きのトークでもベースウェアのプロパティを読める

#### Acceptance Criteria
1. When チェイントークの続きが、後の要求で再開されてから `get_property` を呼ぶ, the Lua ランタイム shall 待ちの期限を「その `get_property` を呼んだ時点の時刻 + 待ちの秒数」にする。
2. When シーンの最初の区切りの中で `get_property` を呼ぶ, the Lua ランタイム shall 今と同じ期限（その要求の時刻 + 待ちの秒数）にする。
3. When 続きを再開してから、待ちの秒数より短い時間で応答（OnPastaCallBack）が届く, the Lua ランタイム shall その応答で続きを再開し、時間切れの表記を出さない。
4. When 続きを再開してから、待ちの秒数を過ぎても応答が届かない, the Lua ランタイム shall 今と同じ時間切れの扱い（時間切れの文で再開）をする。
5. The Lua ランタイム shall 期限を、仮想時刻の設定し直しに左右されない時計（今の `CLOCK.steady` と同じ数え方）で数える。
6. The Lua ランタイム shall 続きの中の期限のほかの振る舞い（日時の変数・`act.req` の読み方）を、今のままにする（決定 2。直すのは期限だけ）。

### Requirement 4: 振る舞いを変えない整理（指摘 3・4・5・6・8・9・10・17・18・19・20・21）
**Objective:** As a pasta の Lua ランタイムを読む開発者, I want 写し・死んだ分岐・素通しの包み・使い手の無い部品が無いこと, so that 直すときに 1 か所を読めば済み、同じ直しを何か所にもしなくてよい

#### Acceptance Criteria
1. The Lua ランタイム shall シーンの関数をコルーチンに包み、終わりに組み立てた結果を返す手順を、1 か所にまとめる（指摘 3。`kick.lua` は 0.5.0 の `debug-mcp-runtime` がファイルごと消すので数に入れず、触らない。おすすめメニューの、組み立てを呼ばない動かし方は写しではないので残す）。
2. The Lua ランタイム shall LuaJIT 2.1 で届かない `coroutine.close` の分岐を持たない（指摘 4）。
3. The Lua ランタイム shall コールバック待ちに入ったかを、グローバルの旗を介さずに、続きの決め直しへ渡す（指摘 4）。
4. The Lua ランタイム shall この整理の後も、`chain-talk-interrupt` の続きの決まり（台詞を出さないシーンは続きに触れない・中断の台詞が出た回は続きを残さない・手順の順）を保つ。
5. If 中断の台詞が出た回に、同じ再開がコールバック待ちを登録していた, the Lua ランタイム shall 後でそのシーンをコールバックで再開してチェイントークで区切ったとき、続きを取りこぼさない（今の旗の残り方による取りこぼしの見込みを、旗を無くすことで消す）。
6. The Lua ランタイム shall `SHIORI.load` と `SHIORI.unload` を、`xpcall` で包まずに、今と同じ戻り値を返す形にする（指摘 5）。
7. The Lua ランタイム shall アクターの単語の作成を、素通しの包みを介さずに行う（指摘 6）。
8. The Lua ランタイム shall 毎秒の時刻の知らせのうち、ランダムトークを出さない秒では、設定（セーブと pasta.toml）を読み直さない（指摘 8）。
9. The Lua ランタイム shall 振る舞いと合わない古いコメント（前の続きを「閉じる」という説明）を持たない（指摘 8）。
10. The Lua ランタイム shall `find_handler` を `find_act_handler` と同じ関数にし、`SCENE.register` は全体のシーンの表を作る部品を使う（指摘 9。インスタンスでの上書きは今と同じく効く）。
11. The Lua ランタイム shall シーンの名前の引き当て、名前の形の検査、「見つからない」と Reference の型の失敗の表記を、シーンの再生・外からの関数の呼び出し・おすすめメニューで 1 組の部品にまとめる（指摘 10）。
12. The Lua ランタイム shall 指摘 18 の小さな写しと重なり（不要な遅延 `require`・`fn_succeeded` の写し・`co_exec` の 2 重の検査）を持たない。
13. The Lua ランタイム shall 指摘 19・20・21 の、実行時の使い手の無い部品を消し、内部の頁とスキルの references の記述も消す（決定 3）。
14. The pasta のテスト shall 何も確かめないテスト（指摘 17）を持たない。同じ振る舞いは、それを確かめる Lua のテストで守る。
15. When この要件の整理を入れる, the Lua ランタイム shall ゴーストの作者に見える振る舞い（応答の文字列・バルーン・警告のログ・セーブの中身）を変えない。

### Requirement 5: 新しく見つかった、作者に見える不具合の直し（指摘 11・12・13）
**Objective:** As a ゴーストの作者, I want 書いたシーンの小さな誤りで、ベースウェアが固まったり、エラーの場所が分からなくなったり、台詞に内部の値が出たりしないこと, so that 書き間違いの原因を自分で見つけられる

#### Acceptance Criteria
1. If シーンが、何も出さずに区切ることを 1 つの要求の中で決まった回数を超えて続ける, the Lua ランタイム shall 区切りの再開をやめ、スクリプトのエラーの表記（どのシーンかが分かるもの）を応答に出す。黙って空の応答にはしない（指摘 11・決定 4。回数の上限は設計で決める。ASSUMPTION A6: 上限は普通のシーンが届かない大きさ）。
2. While 何も出さずに区切る回数が上限より少ない, the Lua ランタイム shall 今と同じく、その区切りを飛ばして再開を続ける。
3. When ローカルシーンの中の行が関数（`＠関数（）`・`＠＊関数（）` の展開・単語の関数・アクターの行からの呼び出し）を呼んだ後、同じローカルシーンの後の行で実行時のエラーが起きる, the Lua ランタイム shall エラーの表記に、ローカルシーン名と `.pasta` のファイルと行を、関数を呼ぶ前と同じく出す（指摘 12。`＞` の呼び出しは今も戻しているので、その振る舞いは変えない。今の「空になる」振る舞いを固めている `act_call_restore_test.lua` の確かめは、直した後の振る舞いに書き換える。決定 5）。
4. When ACT の段で見つかった関数（`＠＊関数（）`・`＠関数（）`・単語の関数）が ACT やアクターのプロキシそのものを返す, the Lua ランタイム shall それを空の結果として扱い、台詞に `table: 0x…` を出さない（プロキシの段と同じ決まり。指摘 13）。
5. When ACT の段で見つかった関数が ACT とプロキシ以外の値を返す, the Lua ランタイム shall 今と同じ値を返す（複数の戻り値も含めて）。

### Requirement 6: マニュアル・スキル・使用例をそろえる
**Objective:** As a ゴーストの作者, I want マニュアルとスキルの説明が、直した後の振る舞いと合っていること, so that 書かれたとおりに書けば動く

#### Acceptance Criteria
1. When 挙動を変える直し（要件 1・2・3・5）を入れる, the pasta のマニュアル shall 同じ変更で、その振る舞いの説明（設定の表の `hour_margin` の型と既定に戻る決まり、RES の表の改行の置き換え、続きの中の `get_property` の期限、何も出さない区切りの上限、ACT の段の戻り値の扱い）を直す。
2. When マニュアルの章を直す, the pasta の開発者 shall `gen-skill-refs` を走らせ、スキルの references を章と同じ中身にする（`gen-skill-refs.mjs --check` が通る）。
3. When 整理（要件 4）で部品や旗を消す, the pasta のマニュアル shall 内部の頁（`internal-modules.md`・`execution-model.md`）から、消した部品と旗（`STORE.co_callback` など）の記述を消し、今の手順を書く。
4. The 既定の `main.lua` shall 使用例に、実在する関数だけを使う（指摘 16）。
5. When `STORE.co_callback` を消す, the pasta の開発者 shall `novel-checkpoint-rollback` と `shiori4-v4-lua-api` の brief の、`co_callback` を前提にした記述を直す。

### Requirement 7: 直しの確かめ方
**Objective:** As a pasta の開発者, I want 1 つ直すごとに振る舞いが確かめられること, so that 整理で振る舞いが変わっていないと言え、変わったときに 1 つ戻せば済む

#### Acceptance Criteria
1. The pasta の開発者 shall 各直しの前に、その振る舞いを固めるテストを書く（挙動を変える直しは、直す前に落ちて直した後に通るテスト。整理は、直す前も後も通るテスト）。
2. When 1 つの直しを入れる, the pasta の開発者 shall `cargo test -p pasta_lua`（Lua の spec を含む）と luacheck が通ることを確かめてから、1 つのコミットにする。
3. The pasta のテスト shall 要件 1 の数でない `hour_margin`（文字列・真偽値・NaN・無限大）、要件 2 の改行を含む値、要件 3 の後の要求で再開した続きの期限、要件 5 の 3 つの不具合を、それぞれ確かめる。
4. If テストの実行で `sample.generated.lua` に改行の差分だけが残る, the pasta の開発者 shall その差分を戻し、コミットに混ぜない。

## Assumptions

- **A1**: NaN と無限大の `hour_margin` も「数でない」と同じく既定の 30 に戻す。`talk_interval_*` の解決と規則をそろえるため（TOML は `nan`・`inf` を書ける。無限大だとランダムトークが二度と出ない）。
- **A2**: CR と LF は 1 文字ごとに空白 1 つにする（`FAILURE.escape` と同じ）。CR LF の組を空白 1 つにまとめない。
- **A3**: 置き換えるのは見出しの値だけで、見出しの名前は扱わない。名前はランタイムと作者のコードの定数で、改行が入る道が見つかっていない。
- **A6**: 何も出さない区切りの上限は、普通のシーンが届かない大きさ（例: 1 要求で 10,000 回）にし、数は設計で決める。

## 決定事項（2026-11-02 要件ディスカッションで、全議題を推奨案で確定）

1. 正時の手前で見送ったランダムトークが、正時の後の最初の空いた秒に出る今の決まりは変えない。マニュアル（`internals/shiori.md`）とテスト（`virtual_dispatcher_spec.lua`）に書かれた決まりで、brief の Desired Outcome に無い。
2. 続きのトークで直すのは `get_property` の期限だけ。日時の変数や `act.req` は再開した要求のものに替えない（brief の Approach「期限は、CLOCK が OnSecondChange で見た直近の時刻から計算する」に沿う、見える変化のいちばん小さい直し）。
3. 指摘 19・20・21（代わりのバッファ・`pasta.lua_version`・使い手の無い取り出し口）は消す。作者が `scripts/` から呼ぶ API は `book/src/lua/script-api.md` の範囲で、これらは内部の頁（`internals/`）にしか書かれていない。
4. 指摘 11 の上限に達したときは、シーン名を添えたスクリプトのエラーの表記を出す。黙ると作者が書き間違いに気付けない（書き間違いの原因が分かる版の方針）。
5. 指摘 12 の直しに合わせて、`act_call_restore_test.lua` の「restore_scene の後はローカル名と行の対応は空」を固める確かめを書き換える。今の振る舞いを写しただけのテストで、エラーの場所が消えるのは不具合である。
