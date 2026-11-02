# ギャップ分析: ponytail-lua-scripts

- 日付: 2026-11-02（ワークツリー `claude/kiro-start-ponytail-lua-scripts-8cb37c`、main `ae04a28b`）
- 方法: brief の「進め方」のとおり、この領域（`crates/pasta_lua/pasta_scripts/**`）に `/ponytail:ponytail-audit` の手順をやり直した。2 つに分けて読んだ（`pasta/shiori/**` と、`main.lua`・`pasta/*.lua`）。テストと cargo は走らせていない（読んだだけ）。主な指摘は、行を開いて確かめた（`virtual_dispatcher.lua` 55〜85・186〜255 行、`res.lua` 39〜60 行、`event/init.lua` 178〜202 行、`main.lua`、`act.lua` 660〜676 行、`shiori/act.lua` 315〜330 行、`recommend.lua` 155〜215 行）。
- 想定した負荷: 1 つのゴーストが 1 つのプロセスで、SSP から 1 秒ごとの OnSecondChange を順に受ける。

## 1. 控え（2026-10-19）との突き合わせ

| 番号 | 今の状態 | 今の場所 | 控えとの違い |
| ---- | -------- | -------- | ------------ |
| 1 | 残っている | `pasta/shiori/event/virtual_dispatcher.lua` 84 行 `hour_margin = ghost.hour_margin or 30`。206 行で比べ、211 行の決め直しより前で投げる | 行だけ違う（控えは L121-125・L245-247）。Rust の `apply_shiori_defaults`（`src/loader/config/mod.rs` 209〜211 行）は書かれていないキーだけを埋めるので、`"30"` は文字列のまま Lua に届く。id が `On` で始まるので、STOP が毎秒エラーの表記の応答にする |
| 2 | 残っている | `pasta/shiori/res.lua` 51〜55 行 `rc .. k .. SPLIT .. v .. CRLF` | 道が増えた: `get_property` の時間切れの文が `X-Error-Reason` に生で入る（`callback.lua` 185〜186 行）、`recommend.lua` の `check_text`（37〜39 行）は 1・2 の字を断るが CR/LF を通す |
| 3 | 残っている（数が変わった） | `scene.lua` 186〜196 行、`choice_select.lua` 25〜35 行、`kick.lua` 59〜71・80〜92 行、**`external_control.lua` 83〜86 行（新しい写し）** | `recommend.lua` 130 行の裸の `coroutine.create` は、組み立てを呼ばない意図した形（127〜128 行で `act.build`・`act.yield` を差し替える）で、写しではない。brief の「5 つ目の写し」は、`external_control.lua` に読み替える |
| 4 | 残っている | `event/init.lua` `set_co_scene` 111〜155 行（届かない分岐 114〜116・132〜135・147〜151 行。117 行の `co = nil` は残す）、`store.lua` 100〜105 行、`STORE.co_callback` は `callback.lua` 73・230 行で書き、`init.lua` 122〜138 行で読む。`EVENT.drive` は 258 行から、287 行で `consume_staged` の戻り値を捨てる | 新しい見込み: `_interrupted` の枝（294〜296 行 `set_co_scene(nil, false)`）は旗を消さない。同じ再開がコールバック待ちを登録していれば旗が残り、後でそのシーンを再開してチェイントークで区切ると、続きを取りこぼしうる（読んだだけ。再現していない）。`execution-model.md` 110 行の「抜けた時点で印は常に nil」と食い違う |
| 5 | 残っている | `entry.lua` 69〜84 行（load）・140〜151 行（unload） | なし。`error_handler`（56〜61 行）は `request` が使うので残す。`shiori_entry_test.lua` 107・111 行は戻り値だけを見る |
| 6 | 残っている | `actor.lua` 20〜53 行（`ACTOR_WORD_BUILDER_IMPL` 28〜42 行、`create_word` 48〜53 行） | なし。`_word_builder` を読むテストは無い。`internals/internal-modules.md` 273 行とスキルの references 268 行が触れている |
| 7 | 残っている | `shiori/act.lua` 324 行 `CLOCK.steady(self.req)`。`act.lua` 664〜675 行 `ACT_IMPL.yield` が再開の値を捨てる（`EVENT.fire` 344 行が新しい act を渡す） | 同じ根で、続きの中の `CLOCK.date(self.req)`（`shiori/act.lua` 157 行）も古い要求を読む。コールバックの道は refs の配列を渡す。続きの期限を固めるテストは無い（`get_property_test.lua` は最初の区切りだけ） |
| 8 | 残っている | `virtual_dispatcher.lua` 191 行（毎秒）・100 行（出す秒にもう 1 度）。古いコメントは 252 行 | なし。191 行を 200 行の早い戻りより後へ動かせば、毎秒の読み直しが消える |
| 9 | 残っている | `act.lua` 404〜412 行 `find_handler`、`scene.lua` 60〜67 行 `register` と 72〜79 行 `create_global_table`、137 行の `get_global_table(...) or create_global_table(...)` | なし。インスタンスでの上書き（`act_call_restore_test.lua` 343 行・`external_control_event_test.lua` 78 行）は別名にしても効く |

控えのうち、直ったものは無い。行番号は、前の再測定（brief の 2026-11-02 の節）と合っている。

## 2. やり直しで増えた指摘

番号は requirements.md の「監査の指摘の一覧」と同じ。

### 作者に見える不具合（挙動を変える）

- **11. 何も出さずに区切り続けるシーンで固まる**（`event/init.lua` 178〜202 行 `resume_until_valid`）。nil の区切りを上限なしに再開する。例: Lua のシーン `while not SAVE.ready do act:yield() end`（次の秒を待つつもり）。`act:yield()` は何も組み立てずに nil で区切り、ループが終わらず、要求が返らず SSP が固まる。直し: 1 要求の中の nil の再開を数え、上限を超えたらエラーにする。固めるテスト: 無い。
- **12. 関数を呼んだ後、エラーの表記の場所が欠ける**（`act.lua` 769〜782 行 `restore_scene`。呼び手は `act.lua` 443・469・554 行、`actor.lua` 202・274 行）。シーン名だけを戻し、ローカル名と行の対応を空にする。コメントの「呼ぶ前のローカル名は分からない」は誤りで、`call_restore`（795〜803 行）は 4 つとも控えて戻している。例: `＊天気・雨の日` の中で `さくら：＠今日（）です`（シーンに触れない関数）の後の行で実行時のエラー → 表記は `＊天気` だけで `dic\天気.pasta:N` が出ない。直し: 呼ぶ前に 4 つを控えて戻す形を 1 つにし、`call_restore` もそれを使う。固めるテスト: `act_call_restore_test.lua` 661〜674・676〜686 行が今の振る舞い（空になる）を固めているので書き換える。
- **13. ACT の段の関数が ACT を返すと `table: 0x…`**（`act.lua` 550〜562 行 `global_fn`、465〜478 行 `call_expr`、441〜444 行 `word`）。プロキシの段（`actor.lua` 177〜180・202 行の `drop_self`）は捨てるが、ACT の段は捨てない。例: `function GLOBAL.挨拶(act) return act:talk(nil, "やあ") end` で `さくら：＠＊挨拶（）` → バルーンに `table: 0x…`。`さくら：＠挨拶（）` は何も出ない。直し: 先頭の戻り値が ACT（かプロキシ）なら nil を返す（`drop_self` と同じ 1 行の決まり）。固めるテスト: 無い。`internal-modules.md` 293 行が今の非対称を「現状」として書いているので直す。
- **14. おすすめメニューの値が `SHIORI/` で始まる**（`recommend.lua` 160〜168・198〜205 行）。受け手が生の文字列を返し、`EVENT.fire`（`init.lua` 352 行）は `SHIORI/` で始まる文字列を応答そのものとして返す。例: 項目名やキャプションの単語が `SHIORI/…` → 符号化したメニューが応答全体になり、SSP が読めない。直し: `RES.ok(...)` で包んで返す。固めるテスト: 無い。
- **15. RES の部品が呼び手の表を書き換える**（`res.lua` 76〜77・120〜124・135〜136 行）。例: `local H = {...}` を `RES.ok(s, H)` に渡した後で `RES.no_content(H)` → 204 に前の `Value` が残る。直し: 書く前に写す。固めるテスト: 無い（`res_test.rs` は毎回新しい表を渡す）。

### 振る舞いを変えない整理

- **10. 名前の引き当てと失敗の表記の写し**: `external_control.lua` 30〜52 行と `scene_play.lua` 38〜60 行の `fail`・`quoted`・`is_name`・`bad_reference0` が同じ。`recommend.lua` 278〜284 行も `FAILURE.report` と `act:build()` の組を繰り返す。引き当ては `scene_play.lua` 69〜83 行、`external_control.lua` 73〜80 行、`recommend.lua` 119〜122・275〜284 行。`external_control_event_test.lua` と `scene_play_event_test.lua` が出力を守る。
- **16. 既定の `main.lua` の使用例**（12〜23 行）: 在しない `WORD.register_actor_dictionary` を呼ぶ。写した作者は読み込みで落ちる。直し: `WORD.create_actor("俺", "一人称"):entry("俺", "おれ", "オレ")` にするか、例を消す。
- **17. 何も確かめない Rust のテスト**: `crates/pasta_lua/tests/shiori/virtual_event_dispatch_test.rs` 307〜355 行 `test_ontalk_hour_margin_skip` は `return true -- Placeholder` で終わる。本当の確かめは `virtual_dispatcher_spec.lua` 930 行にある。消す。
- **18. 小さな写し**: `act.lua` 665〜670 行の遅延 `require`（`pasta.store`・`pasta.global` は先頭 9〜10 行で読み込み済み。`@pasta_log` だけ遅延のままでよい）、`fn_succeeded`（`act.lua` 453〜456 行と `actor.lua` 186〜189 行が同じ）と `call_expr` のほぼ写し（`act.lua` 465〜478 行と `actor.lua` 198〜211 行）、`scene.lua` 208〜219 行 `co_exec` の 2 重の検査。
- **19. 代わりのバッファ**（`buf.lua` 22〜66 行）: `string.buffer` は必ずある（`tests/string_buffer_availability_test.rs` が守る）ので選ばれない。使い手は `sakura_builder.lua` 11 行だけ。`buf_test.lua` の `new_fallback` の例と、`choice-line-layout` が足した「2 種類のバッファの出力が一致する」確かめが使っている。
- **20. `pasta.lua_version`**（78 行全部）: 実行時の使い手が無い。mlua は vendored の LuaJIT 2.1 に固定で、`get()` は常に 221。使うのは `lua_version_test.lua` と内部の頁・スキルの references だけ。
- **21. 使い手の無い取り出し口**: `word.lua` 99〜117 行（`get_global_words`・`get_local_words`・`get_actor_words`）、`scene.lua` 100〜105 行（`get_global_name`）。テストと内部の頁にだけ出る。`get_all_words` は Rust の `finalize.rs` が呼ぶので残す。

### 指摘にしなかったもの（決めごと・範囲の外）

- 正時の手前で見送ったトークの時刻を決め直さないこと（`virtual_dispatcher.lua` 204〜211 行）。監査は「時報のすぐ後に続けて話す」を挙げたが、`internals/shiori.md` 504 行に決まりとして書かれ、`virtual_dispatcher_spec.lua` 930〜940 行が守る。要件の決定 1 で「変えない」と決めた。
- `REG.OnSecondChange` を上書きすると `CALLBACK.sweep` も止まること（`shiori-events.md` に書かれている）。
- `X-Error-Reason` の `init.lua:`・`callback.lua:` の位置の前置き（`shiori_entry_test.lua` 339・356 行が守る）。
- 空の `shiori/init.lua`・`pasta/areka/init.lua`（残す決定）。
- `set_spot` が知らない名前を黙って無視すること（`dsl-codegen-runtime-safety` 2.8 の決定）。
- `appearance.lua`・`sakura_builder.lua` の Rust との写し（適合のテストで守られ、書かれている）。`sakura_builder.lua` に書記素の扱いのコードは無く、改行の割り付けは Rust の `talk_to_script`（`talk-grapheme-clusters` の持ち場）。
- 範囲の外で見えたもの: Rust の `finalize.rs` の `collect_words` が、`entry()` に文字列でも数でもない値が来たとき、どの単語かを言わずに読み込み全体を失敗にする（`ponytail-lua-loader` の領域で、完了済み。この spec では扱わない）。

## 3. 要件と資産の対応

| 要件 | 触る資産 | ギャップ |
| ---- | -------- | -------- |
| 1 hour_margin | `virtual_dispatcher.lua` `get_config`（55〜85 行の `resolve` がひな形）、`tests/shiori/virtual_event_config_resolution_test.rs`、`virtual_dispatcher_spec.lua` | Missing: 型の確かめ。`resolve` は floor と下限 10 の補正をするので、そのまま使うと「補正しない」決まり（`shiori.md` 505 行）が変わる。数かつ有限かだけを見る小さな確かめが要る |
| 2 応答 | `res.lua` `RES.build`・`RES.ok`・`RES.err`・`RES.warn`、`recommend.lua` の受け手、`FAILURE.escape`（置き換えの手本）、`res_test.rs`・`res_ok_test.lua` | Missing: 値の置き換え・表の写し。Constraint: 改行の無い値はバイト不変（既存のテストが守る） |
| 3 期限 | `shiori/act.lua` `get_property`、`pasta/act.lua` `ACT_IMPL.yield`、`clock.lua`（`observe` 149 行・`steady` 221 行）、`callback.lua` | Missing: 続きで今の時刻を得る道。Unknown: `CLOCK` が直近に見た要求の時刻を出す口が要るか（今は `steady(req)` だけ）。Research Needed |
| 4 整理 | `scene.lua`・`choice_select.lua`・`kick.lua`・`external_control.lua`・`scene_play.lua`・`recommend.lua`・`event/init.lua`・`callback.lua`・`store.lua`・`entry.lua`・`actor.lua`・`act.lua`・`word.lua`・`buf.lua`・`lua_version.lua`・`sakura_builder.lua`（`buf` の読み込みだけ） | Constraint: `chain_talk_interrupt_test.lua` の手順の順、`set_co_scene(co, keep)` の `keep` と `_interrupted` の枝を保つ。`act.lua` はこのウェーブでこの spec だけが触る |
| 5 新しい不具合 | `event/init.lua` `resume_until_valid`、`act.lua` `restore_scene`・`call_restore`・`word`・`call_expr`・`global_fn`、`actor.lua` の呼び手、`stop.lua`（表記の場所の読み） | Missing: 上限・4 つの欄の控え・ACT の段の正規化。Constraint: `act_call_restore_test.lua` の書き換え |
| 6 マニュアル | `book/src/internals/{shiori,execution-model,internal-modules}.md`、`book/src/lua/shiori-events.md`、`book/src/reference/pasta-toml.md`（`hour_margin` の行）、`gen-skill-refs`、`main.lua`、`novel-checkpoint-rollback/brief.md` | Constraint: `internals/shiori.md`・`execution-model.md` は `ponytail-shiori` と別の節。後から入る側が rebase |
| 7 確かめ | `cargo test -p pasta_lua`、luacheck（luarocks の lfs・argparse を `LUA_PATH`・`LUA_CPATH` に足す）、`NoDefaultCurrentDirectoryInExePath` を外す | なし |

## 4. 実装の進め方の選択肢

### 案 A: 今のファイルの中で直す（拡張）

- 新しいファイルを作らない。写しは、いちばん自然な持ち主へ寄せる（包む関数は `SCENE.wrap` として `scene.lua`、失敗の表記の部品は `FAILURE`、`fn_succeeded` も `FAILURE`）。
- 利点: brief の「新しいファイルは無い見込み」に合う。`require` の循環を増やさない（`FAILURE` と `SCENE` は今も両方から読まれている）。
- 欠点: `failure.lua`（235 行）に、イベントの名前の検査（`is_name`・`bad_reference0`）が入り、役目が少し広がる。

### 案 B: 共通の部品のファイルを 1 つ足す

- `scene_play.lua`・`external_control.lua`・`recommend.lua` が共有する名前の引き当てと失敗の表記を、`pasta/shiori/event/` の下の新しい小さなモジュールに置く。
- 利点: 3 つのイベントの共通の手順が 1 か所で読める。
- 欠点: ファイルが 1 つ増える。`REG` の読み込みの順（`event/init.lua` 91 行あたり）に気を配る。

### 案 C: 混ぜる

- 包む関数・`fn_succeeded`・`restore_scene` の控えは案 A（今の持ち主へ）、イベントの名前の引き当ては案 B。
- 利点: それぞれの役目に合う置き場。欠点: 決めることが増える。

推奨の見込み（決めるのは設計）: 案 A。消す行が多く、足す行が少ない。

## 5. 規模と危うさ

- 規模: **M**（3〜7 日）。12〜15 タスクの見込み（brief と同じ。増えた指摘 11〜21 は、整理の多くが数行ずつで、1 つのタスクにまとめられるものがある）。
- 危うさ: **Medium**。挙動を変える直しのうち、要件 3（続きの期限）と要件 5 の 12（場所の控え）は、チェイントーク・コールバック・STOP の表記にまたがる。整理の 4（旗を消す）は `chain-talk-interrupt` の手順の順に触れる。どれも既存のテストが厚く（Lua の spec 約 70 本）、1 つずつ戻せる。

## 6. 設計へ持ち越す調べもの（Research Needed）

1. 続きの期限の「今の時刻」の取り方: brief の案（`CLOCK` が OnSecondChange で見た直近の時刻）か、`ACT_IMPL.yield` で再開の act の `req` を受け取る形か。要件の決定 2 で「期限だけを直す」と決めたので、後者を採るなら日時の変数を変えない形にする。コールバックの道（refs の配列が来る）の扱いも決める。
2. `consume_staged` の真偽を `set_co_scene` へ渡す形が、`keep` の枝と捨てる枝の両方で今の決まりと同じになるか（brief の古い案「`if consume_staged then co = nil end; set_co_scene(co)`」は今の形に合わない）。`_interrupted` の枝の旗の残りの見込みを、テストで再現できるか先に確かめる。
3. nil の区切りの上限の数と、表記に出すシーンの名前の取り方（`STOP` が読む欄）。
4. `restore_scene` を 4 つの欄の控えに変えたとき、`STOP` の表記（`stop.lua` 67〜78・110〜115 行）がローカル名と行を正しく出すか。
5. 指摘 19 の代わりのバッファを消すとき、`choice-line-layout` が足した「2 種類のバッファで出力が一致する」確かめを、どう畳むか（消してよいか、`string.buffer` だけで残す確かめがあるか）。
6. `hour_margin` の数の確かめに、`resolve` を使うか小さな別の確かめにするか（補正をしない今の決まりを保つ）。
7. `kick.lua` の 2 つの包みを `SCENE.co_exec_registered` に寄せられるか（監査の案。`wrap_reload_func` は中身が違う）。

## 7. まとめ

- 控えの 9 件は、どれも残っている。直ったものは無い。行番号は 2026-11-02 の再測定と合う。
- 増えた指摘は 12 件（10〜21）。そのうち作者に見える不具合が 5 件（11 固まる・12 エラーの場所が欠ける・13 `table: 0x…`・14 `SHIORI/` の取り違え・15 表の書き換え）。
- brief の「`recommend.lua` が 5 つ目の写し」は誤りで、5 つ目は `external_control.lua`。brief の `consume_staged` の案は今の形に合わない（再測定の節のとおり）。
- 消せる見込み: 約 300 行（`pasta/shiori/**` で約 100 行、`pasta/*.lua` で約 200 行）と、48 行の Rust の空のテスト。
- 読んでいない所: テストと cargo は走らせていない。`appearance.lua` の着せ替えの規則は 1 行ずつは見ていない。Rust の `talk_to_script` が台詞の生の CR/LF を落とすかは確かめていない（要件 2.4 の台詞の道に効く。設計フェーズで確かめた。下の「そのほかに確かめたこと」）。速さは測っていない。

---

# 設計フェーズの調査（2026-11-02・`/kiro-spec-design -y`）

## Summary
- **Feature**: `ponytail-lua-scripts`
- **Discovery Scope**: Extension（既存の Lua ランタイムの中の直し。軽い調査。外の依存は足さない）
- **Key Findings**:
  - 旗 `STORE.co_callback` は `consume_staged` の真偽で置き換えられる。brief の案（`if consume_staged then co = nil end` の後に今の `set_co_scene(co, keep)`）は、`keep` の値をそのまま渡せば、今の場合分けのすべてで同じ `STORE.co_scene` になる。違うのは旗が残る場合だけで、そこが取りこぼし（要件 4.5）の原因だった。ギャップ分析の「brief の案は今の形に合わない」は、`keep` を捨てる読み方をしたときの話で、`keep` を保てば合う。
  - act のメソッドの名前の集まりは `pasta_check` の一致テスト（`crates/pasta_check/src/validate/builtins.rs` の `ACT_METHODS`・`tests/validate_parity_test.rs`）が照らす。メソッドを足す・消すと Rust の別の crate とマニュアルの `debug/validate.md` まで波及するので、`restore_scene`・`find_handler` は名前を残して中身を替える。
  - Lua の spec は 1 つの VM で全部走る（`tests/lua_unittest_runner.rs` が `init.lua` を 1 回読む）。モジュールの状態に「今の時刻」を持たせると、テストの間で持ち越される。続きの期限は、シーンの act が再開した要求を控える形にした。

## Research Log

### RN1: 続きの期限の「今の時刻」の取り方
- **Context**: ギャップ分析の Research Needed 1。
- **Sources Consulted**: `pasta/act.lua` 664〜675 行（`ACT_IMPL.yield`）、`pasta/global.lua`（`GLOBAL.yield` → `act:yield()`）、`pasta/shiori/event/init.lua` 343〜344 行（`EVENT.drive(result, act, act)`）、`callback.lua` 118・178 行、`second_change.lua`、`clock.lua` 145〜170 行（`observe`）、`tests/lua_unittest_runner.rs`。
- **Findings**:
  - チェイントークの区切りはすべて `ACT_IMPL.yield` を通る（`coroutine.yield` を呼ぶのは `act.lua`・`shiori/act.lua` の `get_property`・`recommend.lua` の `cut` の 3 か所だけ）。続きの再開の値は、`EVENT.fire` が作った新しい act である。
  - コールバックの再開の値は refs の配列（`try_route`）、時間切れは `(nil, reason)`（`sweep`）で、どちらも `get_property` の区切りに返る。
  - brief の案（`CLOCK` が OnSecondChange で見た直近の時刻）は、`CLOCK.observe` が `try_route` の後に呼ばれる（コールバックの要求を見ない）うえ、モジュールの状態がテストの間で持ち越される。
- **Implications**: act の欄 `_resume_req` に再開した要求を控える。`ACT_IMPL.yield` が再開の値の act から、`get_property` が 3 つ目の値から受け取る。`try_route`・`sweep` が要求を 3 つ目の値で渡す。`act.req` と日時の変数は替えない（決定 2）。

### RN2: 旗を消した受け渡しが今の決まりと同じになるか
- **Context**: Research Needed 2。
- **Sources Consulted**: `event/init.lua` 111〜155 行（`set_co_scene`）・258〜302 行（`EVENT.drive`）、`callback.lua` 61〜75 行、`shiori/act.lua` 68〜102 行（`SHIORI_ACT_IMPL.build`）・308〜356 行（`get_property`）、`sakura_builder.lua` 413〜423 行（`BUILDER.speaks`）。
- **Findings**:
  - `set_co_scene` の場合分け（待ちに入った／入らない × `keep` 真／偽 × `_interrupted`）を表にすると、旗の残る 1 つ（`_interrupted` の枝で `set_co_scene(nil, false)` が旗を消さない）を除いて、`co = nil` にしてから渡す形と一致する（design.md の System Flows の表）。
  - 旗が残る道: 同じ再開の中で、台詞の組み立て（`act:build()` を区切らずに呼ぶと `_on_speech` が中断を流し `_interrupted` が立つ）と `get_property` の予約が両方起きたとき。`get_property` の組み立ては `raw_script` だけで `BUILDER.speaks` は偽なので、`get_property` の区切りそのものでは `_interrupted` は立たない。よって DSL のふつうの書き方では起きず、Lua で `act:build()` を明示に呼んだときだけ起きる。
  - 旗が残った後、そのシーンを `OnPastaCallBack` で再開して台詞を出しチェイントークで区切ると、`set_co_scene(co, false)` が `co == STORE.co_callback` に当たり、`co_scene = nil` で終わる（続きの取りこぼし）。読んで確かめた（実行はしていない）。設計のテストは、直す前に落ちることを先に確かめる。
- **Implications**: 旗・`close` の分岐・同一の確かめを消し、`set_co_scene` は 3 行にする。失敗の枝は `set_co_scene(nil, false)`（区切りの上限で止めたコルーチンは suspended のまま捨てるため）。

### RN3: 何も出さない区切りの上限と、表記に出すシーンの名前
- **Context**: Research Needed 3。
- **Sources Consulted**: `event/init.lua` 178〜202 行、`stop.lua` 37〜39 行（`STOP.hold`）・67〜78 行（`STOP.subject`）・179〜210 行（`STOP.response`）、`failure.lua` 188〜207 行（`bind`・`act_of`）、`entry.lua` 101〜122 行。
- **Findings**: `EVENT.drive` の失敗の枝は `STOP.hold(co, value)` で `FAILURE.act_of(co)`（`init_scene` が結んだ act）を控え、`SHIORI.request` の `STOP.response` が `STOP.subject(act)`（書いた名前）を対象にする。`resume_until_valid` が `false, 文` を返せば、ほかの Lua のエラーと同じ道で、シーン名つきの表記になる。位置の接頭辞の無い文は `STOP.detail` がそのまま説明に使う。act を結んでいない Lua のコルーチンでは、今と同じく要求の ID が対象になる。
- **Implications**: 上限 10,000 回（A6）・文 `何も出さない区切りが 10000 回続いた`。新しい表記の種類は足さない（「スクリプトのエラー」）。

### RN4: `restore_scene` を 4 つの欄の控えに変えたときの STOP の表記
- **Context**: Research Needed 4。
- **Sources Consulted**: `act.lua` 194・216 行（`scene_names`）・769〜803 行（`restore_scene`・`call_restore`）、`actor.lua` 177〜211・253〜276 行、`stop.lua` 67〜115 行、`tests/transpiler/fallback_search_integration_test.rs` 239〜275 行、`crates/pasta_check/src/validate/builtins.rs`。
- **Findings**: `STOP.subject` は `act.scene_name`・`act.local_scene_name` を、`STOP.detail` は `act.line_map` を読むだけなので、4 つの欄を呼ぶ前の値に戻せば、関数を呼ぶ前と同じ表記になる。`scene_names` の読み手は今の `restore_scene` だけ。`pasta.actor` は `pasta.act` を読めない（循環）ので、プロキシは act のメソッドとして呼ぶ必要がある。メソッドを足すと `pasta_check` の一致テストが落ちる。
- **Implications**: `restore_scene(self, fn, ...)`（関数を呼んで 4 つを戻す）に替え、`call_restore` もこれを使う。`scene_names` を消す。Rust のテストの見せかけの act（2 か所）を新しい形に直す。

### RN5: 代わりのバッファを消すときの、出力一致の確かめの畳み方
- **Context**: Research Needed 5。
- **Sources Consulted**: `buf.lua`、`sakura_builder.lua` 11・395・437 行、`sakura_builder_test.lua` 1843〜2046・2863〜2876 行、`tests/string_buffer_availability_test.rs`、`book/src/internals/talk-output.md` 68・144・365 行、`internal-modules.md` 506〜540 行。
- **Findings**: `config.buffer_factory` を使うのは、代わりのバッファを差し込む一致の確かめだけ。`string_buffer_availability_test.rs` は `pasta.buf` と `pasta.lua_version` を使って `string.buffer` が在ることを確かめている。
- **Implications**: 一致の確かめと `buffer_factory` を消す。`sakura_builder.lua` は `require("string.buffer")` を直接使う。`string_buffer_availability_test.rs` は `string.buffer` を直接確かめる形に書き換え、在ることの守りは残す。

### RN6: `hour_margin` の数の確かめ
- **Context**: Research Needed 6。
- **Sources Consulted**: `virtual_dispatcher.lua` 50〜86 行、`tests/loader/config_defaults_test.rs` 364〜384 行、`book/src/internals/shiori.md` 505 行。
- **Findings**: `resolve` は切り捨てと下限 10 の補正をするので使えない（補正をしない今の決まりが変わる）。既定の 30 は Rust の SSOT のテストが Lua の文字列 `ghost.hour_margin or 30` で照らしている。
- **Implications**: 局所の `finite_or(v, default)`（数で、NaN でも ±無限大でもなければ v）を足し、SSOT のテストの照らす文字列を `finite_or(ghost.hour_margin, 30)` に直す。

### RN7: キックの 2 つの包みを `SCENE.co_exec_registered` に寄せられるか
- **Context**: Research Needed 7。
- **Sources Consulted**: `kick.lua` 51〜165 行、`scene.lua` 253〜264 行。
- **Findings**: `wrap_local_func` に渡る関数は `SCENE.get(親, ローカル)` か `SCENE.get_start(名前)`（＝`SCENE.get(名前, "__start__")`）で、型を確かめてから包む。これは `co_exec_registered(親 or 名前, ローカル or "__start__")` と同じ。`wrap_reload_func` は中身が違う（再読み込みのタグだけ）が、`SCENE.wrap(function(a) a:raw_script(…) end)` で同じ包みになる。
- **Implications**: どちらも寄せられる。`STORE.co_preempt` の印は、呼ぶ側の局所の `preempt` が付ける。

### そのほかに確かめたこと
- Rust の `talk_to_script` は台詞の生の CR・LF を落とさない（`src/sakura_script/mod.rs` の `talk_to_script_impl` は分かち書きとウェイトの差し込みだけ。`line_breaker.rs` 100 行は CR LF を幅 0 の塊として扱い、そのまま通す）。要件 2.4 は `RES.build` の 1 か所の置き換えで足りる。
- `act:talk` は `self` を返す（`act.lua` 239〜252 行）。指摘 13 の例（`return act:talk(nil, "やあ")`）は ACT を返す。
- `recommend_menu_test.lua` は受け手（`REG[id]`）を直接呼び、生の値を確かめている。受け手が常に `RES.ok` を返す形にすると多くの確かめを書き換えることになるので、`SHIORI/` で始まる値だけを包む形にした。
- `WordBuilder.entry` は値が 1 つ以上のときだけ登録して自身を返す（`word.lua` 29〜36 行）。`ActorWordBuilder.entry` と同じ。
- `shiori4-v4-lua-api/brief.md` の 20・43 行も `STORE.co_callback` を前提にしている（要件 6.5 は `novel-checkpoint-rollback` だけを挙げる）。
- `tests/loader/config_defaults_test.rs` の SSOT の照らしは `ponytail-lua-loader`（完了済み）の領域で、このウェーブで触る spec は無い。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 案 A（今のファイルの中で直す） | 写しを今の持ち主（`pasta.scene`・`pasta.failure`・`pasta.actor`）へ寄せる | 新しいファイルが無い。`require` の循環を増やさない。消す行が多い | `pasta.failure` に SHIORI のイベントの言い回しが入る | 採る |
| 案 B（共通の部品のファイルを足す） | イベントの名前の検査と失敗の表記を `pasta/shiori/event/` の新しいモジュールに置く | 3 つのイベントの共通の手順が 1 か所 | ファイルが増える。読み込みの順に気を配る | Open Question として残す |
| 案 C（混ぜる） | 包みは案 A、イベントの部品は案 B | 役目に合う置き場 | 決めることが増える | 採らない |

## Design Decisions

### Decision: 旗の代わりに `consume_staged` の真偽で `co` を外す
- **Context**: 指摘 4・要件 4.3〜4.5。
- **Alternatives Considered**:
  1. `set_co_scene(co, keep, waiting)` と引数を足す
  2. `EVENT.drive` で `if consume_staged then co = nil end` の後、今の `keep` を渡す
- **Selected Approach**: 2。
- **Rationale**: `set_co_scene` の引数を増やさず、場合分けの表のとおり今と同じ。
- **Trade-offs**: `co` という名前の変数を途中で `nil` にする（コメントで理由を書く）。
- **Follow-up**: 要件 4.5 の確かめが、直す前に落ちることを確かめる。

### Decision: 続きの期限は act が再開した要求を控える
- **Context**: 指摘 7・要件 3。
- **Alternatives Considered**:
  1. `CLOCK` に直近の時刻を持たせる（brief の案）
  2. act の欄 `_resume_req` に、再開した要求を控える（チェイントーク・コールバック・時間切れの 3 つの道）
- **Selected Approach**: 2。
- **Rationale**: モジュールの状態を足さない（テストの持ち越しが無い）。3 つの再開の道を全部覆い、コールバックで戻った後に続けて呼ぶ `get_property` も呼んだ時点から数える。
- **Trade-offs**: `try_route`・`sweep` が再開に 3 つ目の値を渡す。作者が上書きした OnSecondChange のハンドラが `sweep` に要求を渡さないと、静かな時間切れの後の期限だけは前の要求から数える。
- **Follow-up**: なし。

### Decision: `restore_scene` は名前を残して「関数を呼んで 4 つを戻す」形にする
- **Context**: 指摘 12・要件 5.3。
- **Alternatives Considered**:
  1. 新しいメソッド（例: `keep_scene`）を足し、`restore_scene` を消す
  2. `restore_scene(self, scene, scene_name, local_scene_name, line_map, ...)` と引数を増やす
  3. `restore_scene(self, fn, ...)` に替える
- **Selected Approach**: 3。
- **Rationale**: act のメソッドの名前の集まりが変わらないので、`pasta_check` とマニュアルの組み込みの一覧に触れない。呼び手は 1 行で済み、`call_restore` もこれを使える。
- **Trade-offs**: 名前は同じで引数の意味が変わる。内部の頁と、Rust のテストの見せかけの act が追従する。

### Decision: 戻り値の正規化は `pasta.actor` の `ACTOR.drop_self` 1 つ
- **Context**: 指摘 13・要件 5.4・5.5。
- **Selected Approach**: `r == act` か、メタテーブルが `PROXY_IMPL` の値なら `nil`。プロキシの型を知っているのは `pasta.actor` だけなので、そこに置き、`pasta.act` も使う（`pasta.act` は `pasta.actor` を読み込み済み）。
- **Trade-offs**: プロキシの段でも、別のアクターのプロキシを返すと空になる（今はそこで `table: 0x…`）。

### Decision: `SHIORI/` で始まる受け手の値だけを包む
- **Context**: 指摘 14・要件 2.5。
- **Alternatives Considered**: 1. 受け手が常に `RES.ok` を返す 2. `SHIORI/` で始まるときだけ包む 3. `EVENT.fire` の決まりを変える
- **Selected Approach**: 2。
- **Rationale**: ほかの値の戻り値が今と同じで、受け手を直接呼ぶ既存のテストがそのまま通る。3 は公開の決まりを変える。
- **Trade-offs**: `EVENT.fire` の判定と同じ形の判定を受け手に 1 つ持つ。

### Decision: RES は渡された表を写してから書く
- **Context**: 指摘 15・要件 2.6・2.7。
- **Alternatives Considered**: 1. 写してから足す 2. 書いて組み立ててから元に戻す 3. `RES.build` に追加の見出しの引数を足す
- **Selected Approach**: 1。
- **Rationale**: 組み立てがエラーで抜けても表が汚れない。3 は公開の `RES.build` の形を変える。
- **Trade-offs**: 作者が渡した表の追加の見出しの `pairs` の順が、元の表と違うことがありうる（今も順は決まっていない）。

## Synthesis（design-synthesis の 3 つの見方）

- **Generalization**: 「シーンの関数をコルーチンに包む」は 6 か所（`scene.lua`・`choice_select.lua`・`kick.lua` の 2 つ・`external_control.lua`、と `co_exec_registered`）の同じ形で、`SCENE.wrap` 1 つにする。「関数を呼んだ後に文脈を戻す」は 6 か所（`call_restore`・`word`・`call_expr`・`global_fn`・プロキシの 2 つ）を `restore_scene(fn, ...)` 1 つにする。「関数の戻り値の正規化」はプロキシの段と ACT の段を `ACTOR.drop_self` 1 つにする。「再開した要求を控える」は 3 つの再開の道を act の欄 1 つに集める。
- **Build vs. Adopt**: バッファは LuaJIT の `string.buffer` をそのまま使い、自前の代わりを消す。有限の数の判定は 1 行の局所の関数（`v ~= v` と `math.huge`）。新しい依存は無い。
- **Simplification**: 旗・`close` の分岐・同一の確かめ・`scene_names`・`buffer_factory`・`ActorWordBuilder`・`pasta.buf`・`pasta.lua_version`・4 つの取り出し口・空の `xpcall` を消す。プロキシと ACT の `call_expr` は、警告の文言が違うので 1 つにしない（引数が増えるだけ）。

## Risks & Mitigations
- 整理で見える振る舞いが変わる — 1 直し 1 コミットで、前後とも Lua の spec 全体と `cargo test -p pasta_lua` を通す。`pasta_check` の一致テストも 1 回走らせる。
- `ACT_IMPL.yield` の遅延 `require` をやめると、モジュールを読み直すテストで古い `STORE`・`GLOBAL` を掴む — `act.lua` は既に先頭の `STORE`・`GLOBAL` を使っている（`init_scene`・`global_fn`）ので、読み直しの組は今も `pasta.act` を含む。テストで確かめる。
- `.claude/skills` への書き込みを分類器が拒む（`gen-skill-refs`）— 拒まれたら入った分と残りを示して開発者に許可を求める。
- 同じウェーブの `ponytail-shiori`（`internals/shiori.md`・`execution-model.md`）・`ponytail-lua-debug`（`internals/debug.md`）・`ponytail-lua-transpiler`（`tests/transpiler/` の可能性）と、ファイルの別の所が重なる — ロードマップの約束どおり、後から入る側が rebase で合わせる。

## 設計で使ったスキルと規則
- `.claude/skills/kiro-spec-design/SKILL.md` と `rules/design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`（軽い調査・3 つの見方・レビューの関門）。
- `pasta-lua-coding`（`MODULE`／`MODULE_IMPL` の分け方、`STORE` で循環を避ける規約、EmmyLua の型注釈）。新しい関数の契約の書き方に使った。
- レビューの関門: 要件の ID 48 個がすべて設計に出ることを機械的に確かめた。境界・ファイルの計画・コンポーネントとファイルの対応を確かめた。1 回目の見直しで、コールバックの後の期限（当初は前提として残していた）を設計に取り込み、`Store` の欄を足した。

## 設計ディスカッションの決定（2026-11-02）

- 10 の議題をすべて推奨案で確定した。中身と理由は design.md の「設計の決定事項」。
- 決定 10（`kick.lua` を範囲から外す）は、discovery「Pasta DSL MCP移行検討」（PR#221）の申し送りによる。`debug-mcp-runtime` がキックごと消すので、写しを直してもすぐ捨てることになる。
