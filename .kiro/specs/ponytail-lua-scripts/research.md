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
- 読んでいない所: テストと cargo は走らせていない。`appearance.lua` の着せ替えの規則は 1 行ずつは見ていない。Rust の `talk_to_script` が台詞の生の CR/LF を落とすかは確かめていない（要件 2.4 の台詞の道に効く）。速さは測っていない。
