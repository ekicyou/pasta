# ギャップ分析: ontalk-suppression

- **対象**: `requirements.md`（2026-10-22 生成）と、worktree の base（main `dc46f446`）のコード
- **分類**: 既存の拡張（Extension）。新しいモジュールは要らない見込み
- **規模**: S（1〜3 日）。**リスク**: 低〜中（判定の分岐は小さいが、再開した後の時報の扱いに端の場合がある）

## 1. まとめ

- 止めるスイッチの置き場所・判定の分岐の位置・名前の登録の先例は、すべて既存のコードにそろっている（`STORE` のフラグ、`virtual_dispatcher.lua` の `dispatch`、`shiori/entry.lua` の `close_ghost`／`ゴースト終了`）。新しい仕組みは要らない。
- 一番の穴は **再開した後の時報**（要件 3.2・3.3）。止めている間に `check_hour` を呼ばないと、次の正時の時刻（モジュールの局所変数 `next_hour_unix`）が古いまま残り、再開した直後の最初の機会に「過ぎた時報」が出てしまう。再開の時点で決め直す手当てが要る。
- ランダムトークの数え直し（要件 3.1）は、局所変数 `next_talk_time` を `0` に戻せば、今の初回の分岐（`check_talk` L194〜197）が「その時点から」数え直す。再開の関数から `virtual_dispatcher` のこの変数に届く口が要る。
- `pasta_check` は `GLOBAL_NAMES` を 4→8 にし、書き写しのテスト（`builtin_names_are_listed`）を直すだけで、実行時との一致のテスト（`builtin_names_match_runtime`）は実行時の `GLOBAL` を列挙するので自動で追従する。
- マニュアルは Scope の 4 ページのほかに、内部設計の 2 ページとスキルの references（生成物）と手書きの SKILL.md に同じ名前の列挙がある。

## 2. 要件と既存の資産の対応

| 要件 | 既存の資産 | 状態 |
| ---- | ---------- | ---- |
| 1.1〜1.3 停止・再開の関数と別名 | `shiori/entry.lua` L18〜27（`GLOBAL.close_ghost`／`GLOBAL["ゴースト終了"]`。非 ASCII はブラケット記法） | **Missing**（先例どおりに 4 つ足す） |
| 1.4 `＠` の形 | 単語参照の L4（`GLOBAL[key]` の完全一致）。`＠ゴースト終了` で既に動いている | 既存で満たす（テストだけ） |
| 1.5 出力を加えない | 関数が `act` に何も積まなければよい | 既存で満たす |
| 1.6 呼んだ時点で効く | シーンのコルーチンの中で関数が走った時点でフラグが変わる | 既存で満たす |
| 1.7 シーンの外から呼べる | `act` に頼らない作りにすれば満たす。`external-control-events` が頼る | **Constraint**（下の 4.3） |
| 2.1・2.2 OnTalk・OnHour を出さない | `dispatch`（L227〜271）の `check_hour`・`check_talk` の前 | **Missing**（分岐を 1 つ足す） |
| 2.3・2.8 続きを出さない | 続きは `check_talk` L213〜218 の `STORE.co_scene` からだけ流れる。キックの 2 回目以降も同じ経路 | `check_talk` を呼ばなければ自動で満たす |
| 2.4 キックの最初の出力は出す | `KICK.try_dispatch`（L257〜261）は `check_hour` より前 | 分岐をキックの後に置けば満たす |
| 2.5 ほかのイベントは今どおり | マウス・選択肢は `REG` のハンドラで、`dispatch` を通らない | 既存で満たす |
| 2.6 コールバックの時間切れ・仮想時刻の読み取り | `second_change.lua` L26（`CALLBACK.sweep`）は `dispatch` の前、L37（`CLOCK.take_probe`）は `dispatch` が `nil` のとき | 止めている間 `dispatch` が `nil` を返せば自動で満たす |
| 2.7 続きを捨てない | 捨てるのは `set_co_scene`（`event/init.lua` L100〜136）。`dispatch` が何も返さなければ触れない | 既存で満たす |
| 3.1 ランダムトークの数え直し | `check_talk` の `next_talk_time == 0` の初回の分岐 | **Missing**（再開の時点で `0` に戻す口） |
| 3.2・3.3 過ぎた時報を出さない・次の正時は出す | `check_hour` の `next_hour_unix`。止めている間は進まない | **Missing／Research Needed**（下の 4.1） |
| 3.4 止めていないときの再開は何もしない | — | **Missing**（「止めていたか」を見て分岐） |
| 4.1・4.2 入れ子なし | 真偽の 1 つ | **Missing**（些細） |
| 4.3・4.4 セッションの間だけ | `STORE` はセッション常駐で `pasta.save` と別。unload で Lua の実行環境ごと破棄 | 既存で満たす |
| 4.5 全リセットで解く | `STORE.reset`（`store.lua` L83〜106）。呼ぶのは今はテストだけ | **Missing**（1 行） |
| 5.1〜5.3 既定の挙動を変えない | `virtual_dispatcher_spec`・`_thread_test`・`_kick_force_test`・`_kick_hook_test`・`second_change_thread_test` | フラグの既定を偽にすればバイト不変 |
| 5.4 Lua のテスト | `crates/pasta_lua/tests/lua_specs/`（`virtual_dispatcher_kick_force_test.lua` の作り方が手本）＋一覧 `init.lua` に 1 行 | **Missing** |
| 6.1・6.2 検査が組み込みとみなす | `crates/pasta_check/src/validate/builtins.rs` L6 `GLOBAL_NAMES: [&str; 4]`・`is_builtin` は完全一致 | **Missing**（4→8） |
| 6.3 検査のテスト | `tests/validate_scenes_test.rs` L252 `builtin_names_are_listed`（書き写し）、`tests/validate_parity_test.rs` L606 `builtin_names_match_runtime`（実行時の `require 'pasta.global'` を列挙）、L633 `static_call_lookup_matches_runtime` | 書き写しを直す。一致テストは自動で追従。L633 に 1 行足すとよい |
| 6.4 マニュアル | `grammar/call-jump.md` L213（5 段の表の例）・L326〜334（特別な呼び出しの表と `＠` の形の段落）、`lua/shiori-events.md` L696〜（仮想ディスパッチャ。`dispatch` の判定の順 1〜5、L747「ブロックされた OnHour は、ブロックが解けた最初の OnSecondChange で発行される」）、`lua/script-api.md` L621〜624、`debug/validate.md` L155 | **Missing** |
| 6.5 チェイントークの注意 | `grammar/call-jump.md` のチェイントークの節（L340〜365 あたり） | **Missing** |
| 6.6 内部設計のページ | `book/src/internals/shiori.md` L94・L299、`book/src/internals/registry-search.md` L254 | **Missing** |
| 6.7 スキルの references | `.claude/skills/pasta-ghost-authoring/references/call-jump.md`・`.claude/skills/pasta-lua-coding/references/{script-api,shiori-events}.md`（マニュアルから生成） | 作り直し |
| 6.8 手書きの SKILL.md | `.claude/skills/pasta-ghost-authoring/SKILL.md` L207（特殊 Call の一覧） | **Missing**（編集は分類器に拒まれることがある） |

## 3. 今のコードの要点

- **発行の順**（`dispatch`）: `act.req.date` の有無 → `STORE.kick_force` の消費 → Status の抑止 → キック（`KICK.try_dispatch`）→ 時報（`check_hour`）→ ランダムトーク（`check_talk`。続きもここ）。止める分岐を **キックの後・時報の前** に置けば、要件 2.1〜2.4・2.6 を 1 か所で満たす。
- **状態**: `next_hour_unix`（`CLOCK.now` の時計）・`seen_epoch`・`next_talk_time`（`CLOCK.steady` の時計）は `virtual_dispatcher.lua` の局所変数。テスト用に `_reset`・`_get_internal_state` がある。
- **循環参照の方針**: `virtual_dispatcher.lua` は `CLOCK` だけを先に読み、`pasta.store` などは関数の中で遅延ロードする。`store.lua` はほかのモジュールを読まない。`shiori/entry.lua` は `pasta.shiori.event`・`kick` などを先頭で読んでいる。
- **名前の解決**: Call は 5 段（1 シーンテーブル完全一致 → 2 ローカルシーン前方一致 → 3 act のメソッド → 4 `GLOBAL` 完全一致 → 5 グローバルシーン前方一致）。ローカルシーンに `自動トーク停止…` で始まる名前があると、そちらが先に当たる。これは `ゴースト終了` と同じ既存の決まりで、この spec では変えない。
- **読み込みの順**: `main.lua` → `entry.lua` → シーン辞書。`entry.lua` が `GLOBAL` に登録するので、`main.lua` で同じ名前に代入しても上書きされる（`script-api.md` L624 と同じ注意が 4 つの名前にも当てはまる）。
- **luacheck**: `crates/pasta_lua/.luacheckrc` の `max_cyclomatic_complexity = 15`。`dispatch` に分岐を 1 つ足しても余裕はある見込み（設計で確かめる）。非 ASCII の識別子はブラケット記法。

## 4. 穴と制約

### 4.1 再開した後の時報（Missing／Research Needed）

止めている間 `check_hour` を呼ばないと `next_hour_unix` が進まない。再開した後の最初の機会に `current_unix >= next_hour_unix` となり、止めている間に過ぎた時報が出てしまう（要件 3.2 に反する）。さらに、再開した後のシーンの表示中は Status の `talking` で止まるので、その間に来た正時は「抑止が解けた最初の機会に出す」今の決まり（要件 5.2）に従って出すべきである。手当ての案:

- **H1. 再開の時点の時刻で決め直す**: 再開の関数が `CLOCK.now(act.req)` を読み、`next_hour_unix <= その時刻` なら `calculate_next_hour_unix(その時刻)` に進める。要件どおりに最も正確。ただし `CLOCK.now` は `req.date` が要る（`clock.lua` L185〜189）。シーンの外から `act` 無しで呼ばれたとき（要件 1.7・`external-control-events`）の扱いが要る。
- **H2. 止めている間も「過ぎたかどうか」の記録だけ進める**: 止めている間の tick で、時報を出さずに `next_hour_unix` だけ進める。ただし Status で止まっている tick では `dispatch` が早く返るので、選択肢の表示中に正時を過ぎ、選択の後のシーンで再開すると、過ぎた時報が出る（要件 3.2 に反する）。単独では足りない。
- **H3. 再開で `next_hour_unix = 0` に戻す**: 初回の分岐（L149〜153）が「次の正時を決めて発行しない」ので、手当ては 1 行。ただし、再開から Status の抑止が解けるまでの間に正時を過ぎると、その時報も出ない（要件 3.3 の端の場合。再開した後の台詞の表示中に正時をまたぐときだけ）。
- **H4. 再開の時点の印を残し、`check_hour` が見る**: 再開の関数は「再開した」印（と、取れれば時刻）だけを残し、次の `check_hour` が「印の時刻より前の正時」を捨てる。H1 の正確さを保ちつつ、時刻が取れないときは H3 相当に落ちる。

**Research Needed**: シーンの外（`external-control-events` の NOTIFY など）から呼ばれる `act` に `req.date` があるか。無いときに H3 の端の場合を受け入れるか。

### 4.2 再開の関数から局所変数に届く口（Constraint）

`next_talk_time`・`next_hour_unix` は `virtual_dispatcher.lua` の局所変数で、外から書けない。案:

- **R1. ディスパッチャに公開関数を足す**（例: 再開を受ける関数）。`entry.lua` の `GLOBAL` の関数がそれを呼ぶ。`entry.lua` から `virtual_dispatcher` を読むのは循環しない（ディスパッチャは `CLOCK` だけを先に読む）。
- **R2. フラグだけを `STORE` に置き、ディスパッチャが「止めていた→解けた」の変わり目を見て決め直す**。関数は最小になるが、停止と再開が 1 つのシーンの組み立ての中で続けて呼ばれる（brief の例のとおり）と、ディスパッチャが止めている状態を一度も見ないので、数え直しが起きない（要件 3.1 に反する）。単独では足りない。
- **R3. `STORE` に再開の通し番号（や時刻）を置き、ディスパッチャが前に見た値と比べる**（`CLOCK.epoch` と `seen_epoch` の先例と同じ形）。関数は `STORE` だけを触り、ディスパッチャは自分の局所変数を自分で決め直す。依存の向きが今と同じに保てる。

### 4.3 シーンの外から呼ぶ（Constraint）

- `GLOBAL` の関数は `act` を第 1 引数に取る決まり（DSL の Call がそう呼ぶ）。効き目は `act` の中身に頼らない作りにすると、要件 1.7 と `external-control-events` の両方を満たす。4.1 の H1 だけは `act.req.date` に頼るので、無いときの落としどころが要る。

### 4.4 止めた区間の中のチェイントーク（Constraint）

- 続きを再開する道は `check_talk` だけなので、止めている間に `＞チェイントーク` で区切ると、その続き（と、その中の `＞自動トーク再開`）には再起動するか、ほかのシーンが続きを捨てるまで届かない。
- **決定（2026-10-22 要件ディスカッション）**: 止めている間のチェイントークは、自動トークの切り替えに干渉するスクリプトの誤りとみなす。ログに警告を出し、止めている状態を解く（要件 2.8）。設計では、チェイントーク（`yield`／`チェイントーク`。`entry.lua` の `GLOBAL`）の側で止めている印を見て、再開と同じ手当て（4.1・4.2 で選ぶもの）を呼ぶ。警告のログの出し方（`@pasta_log` の warn など）も設計で決める。

### 4.4b つつき・撫での反応（2026-10-22 要件ディスカッションで追加。要件 2.9〜2.11）

- 既定のハンドラは `pasta_scripts/pasta/shiori/event/mouse.lua`。つつき（`on_poke`）は Status を見ずに 1〜8 段（`find_shortcut`）→ 9 段目（`exec_event_scene(act, "OnMouseDoubleClick")`）。撫で（`on_touch`）は `M.check_stroke(..., dispatcher.is_blocked(req.status))` で「撫でた」を決め、決めたときだけ 1〜8 段、その後 9 段目（`OnMouseMove`）。
- 手当ては 2 か所: `on_poke` で止めている印が立っていれば `find_shortcut` を飛ばす。`on_touch` で `check_stroke` の `blocked` に止めている印を足す（Status の抑止と同じく判定の状態も Off に戻る）。印は `STORE` から読む（`mouse.lua` は既に `pasta.store` を読んでいる）。
- `REG` にゴーストが先に置いたハンドラは `REG.OnMouseDoubleClick or on_poke` で既定が使われないので、自動で対象外（要件 2.11）。
- 既存のテスト `tests/lua_specs/mouse_test.lua` ほかは、印の既定を偽にすれば不変。

### 4.5 既定の挙動のバイト不変（Constraint）

- フラグの既定を偽（未設定は `nil`）にし、偽のときは分岐が素通りすれば、今の `dispatch` のテスト 5 本はそのまま通る。`STORE.kick_force` の先例（「通常経路ではバイト不変」）と同じ作り。

### 4.6 ほかの spec とファイルの共有

- `virtual_dispatcher.lua`・`store.lua`・`shiori/entry.lua`・`builtins.rs` は、後から入る `recommend-menu-scenes`（`entry.lua`・`builtins.rs`）、`novel-talk-flow`・`novel-checkpoint-rollback`・`scene-exact-match`・`ponytail-lua-scripts`（`virtual_dispatcher.lua` ほか）と共有する。この spec が先に入る前提で、あちらが合わせる。
- `scene-play-event` は `virtual_dispatcher.lua` に触らない前提（brief の再測定の注意）。止めている間、シーン再生の続きも止まる（要件 2.3）。
- テストの一覧 `tests/lua_specs/init.lua` への 1 行は、同じウェーブのほかの spec と並べてよい（棚卸 2026-10-22 の扱い）。マニュアルの `lua/shiori-events.md`・`grammar/call-jump.md`・`debug/validate.md` は節が別。

## 5. 実装の案

### 案 A: 既存のファイルを広げる（推奨の候補）

- `store.lua`: 止めている印（既定は偽）と、R3 を採るなら再開の通し番号。`STORE.reset` で片付ける。
- `shiori/entry.lua`: `GLOBAL.pause_auto_talk`・`GLOBAL.resume_auto_talk` と、ブラケット記法の別名 2 つ。
- `virtual_dispatcher.lua`: `dispatch` のキックの後に「止めていれば `nil`」の分岐。再開を受けて `next_talk_time` を `0` に戻し、時報の次の時刻を決め直す（4.1・4.2 の案から選ぶ）。
- `builtins.rs`: `GLOBAL_NAMES` を 8 つに。テスト 2 本（書き写し・静的な Call の検索に 1 行）。
- Lua のテスト 1 ファイル＋一覧に 1 行。マニュアル 6 ページ、references の作り直し、SKILL.md に 1 行。
- ✅ 先例（`kick_force`・`close_ghost`）どおりで、触る行が少ない。❌ `virtual_dispatcher.lua` の責任（発行の判定）に「止める」が 1 つ増える（小さい）。

### 案 B: 止める・再開を受け持つ小さなモジュールを新しく作る

- 例: `pasta/shiori/event/auto_talk.lua` に関数と状態を置き、`entry.lua` は別名を登録するだけ、ディスパッチャはそのモジュールに問い合わせる。
- ✅ 責任が分かれ、`novel-talk-flow` が「常に止める」を足す場所がはっきりする。❌ ファイルが 1 つ増え、ディスパッチャの局所変数の決め直しには結局ディスパッチャ側の口が要る。規模（5 タスク前後）に対して重い。

### 案 C: 混ぜる

- 状態と関数は案 A のまま `STORE`／`entry.lua` に置き、時報と間隔の決め直しは R3（通し番号を見てディスパッチャが自分で決め直す）にする。依存の向き（`entry.lua` → `STORE` ← ディスパッチャ）が今と同じで、`entry.lua` からディスパッチャを読まずに済む。

## 6. 規模とリスク

- **規模: S** — 先例のある小さな分岐と名前の登録。テストとマニュアルの分量が主。
- **リスク: 低〜中** — 判定の分岐は小さい。再開した後の時報（4.1）の端の場合と、シーンの外から呼ぶときの時刻の取り方だけ、設計で決める必要がある。

## 7. 設計フェーズへの申し送り

- 時報の決め直し（4.1 の H1／H3／H4）と、再開の関数から決め直しに届く口（4.2 の R1／R3）を決める。推奨の候補は **H4＋R3**（印を `STORE` に置き、ディスパッチャが次の `check_hour`／`check_talk` で自分で決め直す。時刻が取れるならそれを使い、取れなければ H3 相当）。
- 止める分岐を `dispatch` のキックの後・時報の前に置くことを、判定の順の表（`shiori-events.md`）に合わせて書く。
- 止めていないときに再開を呼んだら何もしない（要件 3.4）ために、「止めていたか」を見て分岐する。
- Lua のテストの観点: 既定で素通り／停止で OnTalk・OnHour・続きが出ない／キックの最初の出力は出る／`CLOCK.take_probe` が返る／再開で間隔の数え直し／止めている間に過ぎた正時を出さない／再開した後の次の正時は出す／2 回停止→1 回再開で解ける／止めていないときの再開で時刻が変わらない／`STORE.reset` で解ける／`＠` の形と `＞` の形の両方。
- **Research Needed**: シーンの外から呼ばれる `act` の `req.date` の有無（`external-control-events` の呼び方）。

---

# 設計フェーズの調査と決定（2026-10-22）

## Summary

- **Feature**: `ontalk-suppression`
- **Discovery Scope**: Extension（軽い調査。`design-discovery-light.md`）。外部のライブラリ・API は足さないので、Web の調査はしない。
- **参照したもの**: steering（`product.md`・`tech.md`・`structure.md`）、スキル `kiro-spec-design` の rules（design-principles・discovery-light・synthesis・review-gate）、`pasta-lua-coding` の考え方（`GLOBAL`・`STORE`・`@pasta_log`）、完了 spec `touch-poke-shortcuts` の設計書（書き方の手本）。
- **Key Findings**:
  - すべての SHIORI リクエストに `req.date` がある（`crates/pasta_shiori/src/lua_request.rs` の `parse_request` L53 が常に `date` を置き、`X-Pasta-Time` があれば置き換える）。§4.1・§7 の **Research Needed は解決**: シーンの中・`REG` ハンドラ・`external-control-events` の口のどれから呼んでも、`act.req.date` から時刻が取れる。取れないのは `act` が `nil` や `req` の無い Lua の単体テスト・直接の呼び出しだけ。
  - **チェイントークの続きの `act` は古い**: 続きを再開するとき `EVENT.drive(co, act, act)` は新しい act を渡すが、`ACT_IMPL.yield`（`act.lua` L663）は `coroutine.yield` の戻り値を捨てるので、シーンの中の `act` は最初のリクエストのまま。続きの中で `CLOCK.now(act.req)` を読むと古い時刻になる。時報の決め直しで「戻す」と、すでに出した正時を二重に出しうる。
  - チェイントークの登録は `entry.lua` でなく `pasta/global.lua`（`GLOBAL.yield`・`GLOBAL["チェイントーク"]`）。呼ぶ先は `ACT_IMPL.yield` だけで、ランタイムの中で `act:yield()` を呼ぶのは `GLOBAL.yield` だけ。
  - `pasta_check` は `＠` の単語参照の飛び先を検査しない（`references.rs` は Call だけを見る）ので、`＠自動トーク停止` は今も報告されない。`scenes.rs` L70 の `scene-shadowed-by-builtin` は `is_builtin` を使うので、4 つの名前と同じ名前のグローバルシーンに新たに警告が出る（既存の決まりの帰結）。
  - `dispatch` の循環的複雑度は今 8 の見込み（if 5・and/or 2・基本 1）。門を 1 つ足して 9。上限 15 に余裕がある。

## Research Log

### 時刻の取り方（§4.1 の Research Needed）

- **Context**: 再開の時点の時刻で時報を決め直す案（H1・H4）は `act.req.date` に頼る。シーンの外から呼ばれたときに時刻があるか。
- **Sources Consulted**: `crates/pasta_shiori/src/lua_request.rs`（`parse_request`・`lua_date`）、`pasta/shiori/clock.lua`（`M.now`・`M.steady`）、`.kiro/specs/completed/external-control-events/brief.md`（`OnPastaCallFunction` の `act:call` で呼ぶ）。
- **Findings**: リクエストの表には常に `date` がある。`external-control-events` は `REG` ハンドラの `act` で `act:call` するので、時刻は取れる。
- **Implications**: 時刻が取れない場合は SHIORI の経路では起きない。落としどころ（初回扱い）は単体テストと直接の呼び出しのためだけで足りる。

### 続きの中の古い act

- **Context**: 再開の関数・チェイントークの警告は、続き（2 回目以降の出力）の中でも呼ばれうる。
- **Sources Consulted**: `pasta/act.lua` L660〜667（`ACT_IMPL.yield`）、`pasta/shiori/event/init.lua` L190〜（`EVENT.drive`）、L239（`EVENT.drive(result, act, act)`）、`pasta/scene.lua`（`wrap_scene`）。
- **Findings**: 続きのシーンが見る `act` は最初のリクエストの act。`act.req.date` は古い。
- **Implications**: 時報の決め直しは「前へだけ進める」（今の `next_hour_unix` より後のときだけ置き換える）。止めている間に続きは動かない（門が `check_talk` を呼ばない）ので、古い act で再開するのは「同じ出力の中で停止と再開を続けて呼んだ」ときだけで、そのとき前へだけ進める決まりは何も壊さない。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| H1＋R1（採用） | 再開の関数がディスパッチャの `restart_timers(act)` を直接呼ぶ。トークは `next_talk_time = 0`、時報は再開の時刻から前へだけ進める。時刻が無ければ `next_hour_unix = 0` | 停止と再開が 1 つの組み立ての中で続いても確実に数え直す。状態を増やさない。時報が正確 | `entry.lua` → ディスパッチャの直接の依存（既に間接に依存していて向きは同じ） | §4.2 の R2 の欠点が出ない |
| H4＋R3（research §7 の推奨） | `STORE` に再開の通し番号と時刻を置き、ディスパッチャが次の判定で前に見た値と比べて決め直す | 関数は `STORE` だけを触る | 状態が 2〜3 個増える（通し番号・時刻・前に見た値）。続きの古い act の時刻を印に残すと同じ二重発行の問題が残る | 依存の向きの利点は、R1 でも向きが変わらないので小さい |
| H3 単独 | 再開で `next_hour_unix = 0` | 1 行 | 再開の後の台詞の表示中に正時をまたぐと、その時報が出ない（3.3 の端の場合） | 時刻が取れないときの落としどころとして採る |
| 案 B（新しいモジュール） | `auto_talk.lua` に関数と状態 | 責任が分かれる | ファイルが増え、時計の決め直しには結局ディスパッチャの口が要る | 規模に対して重い。採らない |

## Design Decisions

### Decision: 時計の決め直しは H1＋R1（前へだけ進める）

- **Context**: 3.1〜3.3。§4.1・§4.2。
- **Alternatives Considered**: 上の表の H4＋R3・H3 単独。
- **Selected Approach**: `GLOBAL.resume_auto_talk` が止めていたときだけ `dispatcher.restart_timers(act)` を呼ぶ。`next_talk_time = 0`。時刻が取れれば `calculate_next_hour_unix(now)` が今の値より後のときだけ置き換え、`seen_epoch = CLOCK.epoch()`。取れなければ `next_hour_unix = 0`。
- **Rationale**: 最小の状態で要件を満たす。古い act の時刻でも二重発行しない。
- **Trade-offs**: 時刻が取れないとき（単体テスト・直接の呼び出しだけ）は H3 の端の場合を受け入れる。
- **Follow-up**: 古い時刻の act での再開と、`act` が `nil` の再開をテストで押さえる。

### Decision: 関数は `entry.lua`、チェイントークの警告は `global.lua` の `GLOBAL.yield`

- **Context**: brief の Approach 1（`close_ghost` の先例）と、要件 2.8（`＞チェイントーク` の側で止めている印を見る）。
- **Alternatives Considered**: (1) 4 つの名前も `global.lua` に置く（`yield` と同じ場所）。(2) 警告を `ACT_IMPL.yield`（`act.lua`）に置き、Lua から直接の `act:yield()` も拾う。(3) 警告を `set_co_scene` に置き、続きができるすべての道を拾う。
- **Selected Approach**: 停止・再開は `entry.lua`（SHIORI 層。ディスパッチャを読む）。`GLOBAL.yield` は印を読み、真なら warn と `GLOBAL.resume_auto_talk(act)`（同じ表から呼び出しの時点で引く）。
- **Rationale**: `global.lua` から SHIORI 層のディスパッチャを読まずに済む。要件 2.8 の文言（`＞チェイントーク`・`＞yield`）にちょうど合う。(3) は `EVENT` の本体に触れ、境界が広がる。
- **Trade-offs**: Lua から直接 `act:yield()` を呼ぶ道は警告されない（Open Question 2）。

### Decision: 門の位置はキックの後・時報の前

- **Context**: 2.1〜2.4・2.6・2.7・5.2。
- **Selected Approach**: `dispatch` の `KICK.try_dispatch` の直後に `if STORE.auto_talk_paused then return nil end`。
- **Rationale**: 1 か所で時報・トーク・続きを止め、キックの最初の出力・Status の抑止・時間切れ・仮想時刻の読み取りは今どおり。

### Decision: つつき・撫では `mouse.lua` の 2 か所

- **Context**: 2.9〜2.11（§4.4b）。
- **Selected Approach**: `on_poke` は印が真なら `find_shortcut` を飛ばす。`on_touch` は `check_stroke` の `blocked` に印を `or` で足す。
- **Rationale**: 既に `mouse.lua` は `STORE` を読む。印が偽なら式の値が今と同じでバイト不変。

## Synthesis

- **Generalization**: 「止める」は 3 か所（ディスパッチャ・つつき・撫で）で同じ印を読むだけ。印の読み手を抽象化する層は作らない（読み手が 3 つで、どれも 1 行）。`novel-talk-flow` の「常に止める」も同じ印で表せる（インタフェースとしては印 1 つで足りる）。
- **Build vs. Adopt**: すべて既存の仕組みを使う（`STORE`・`GLOBAL`・`@pasta_log`・`_get_internal_state`・`gen-skill-refs.mjs`）。新しい依存なし。
- **Simplification**: 新しいモジュール（案 B）・再開の通し番号（R3）・止めているかを問い合わせる関数・切り替えのログは作らない。`restart_timers` だけをディスパッチャの公開関数に足す。

## Risks & Mitigations

- 古い act の時刻で時報を戻して二重に出す — 前へだけ進める決まりとテストで防ぐ。
- 4 つの名前と同じ名前のグローバルシーンを持つゴーストに、検査の警告が新たに出る — 既存の決まりの帰結。マニュアルの検査の一覧に名前を足すことで説明になる。
- 同じウェーブ・後から入る spec と `virtual_dispatcher.lua`・`store.lua`・`entry.lua`・`builtins.rs`・マニュアルを共有する — §4.6 のとおり、この spec が先に入り、あちらが合わせる。`dispatch` の判定の順を Revalidation Triggers に書いた。
- `.claude/skills/pasta-ghost-authoring/SKILL.md` の編集が分類器に拒まれる — 入った分と残りを示して開発者の許可を得る。
