# ギャップ分析: ponytail-lua-debug

## Summary

- **Feature**: `ponytail-lua-debug`
- **Discovery Scope**: Extension（既存の `crates/pasta_lua/src/debug` の直しと撤去。新しいファイルは無い見込み）
- **測った所**: ワークツリーの HEAD `ae04a28b`（main。brief の 2026-11-02 の再測定の基準 `491cc23c` の後は roadmap の 2 コミットだけで、持ち場のソースは同じ）。2026-11-02。
- **Key Findings**:
  - 監査の 1〜12 番は **すべて残っている**。brief の行番号は、2 か所（2・6 番の `stepping.rs L382-387`、11 番の `enable.rs L246-256`）が古い。ほかは今の行と合う（下の表）。
  - 監査のやり直しで、同じ領域に **新しい指摘が 13 個** 増えた。正しさの側が 4 つ（止まっていないときの `threads` が応答されない・受け口の無い要求が応答されない・対応表が無いと `.pasta` のブレークポイントが「確かめ済み」で死ぬ・UNC の URI で位置からシーンを引けない）、無駄の側が 8 つ、説明の誤りが 1 つ。
  - `runtime/exec.rs` の `debug_source_mode()` は **拾う**。値は変えず、説明（「`attach` の値も映る」は誤り）だけ直す（要件 8.4）。消すと 5 つの結合テストの書き直しになり、得るものが無い。
  - 規模は **17〜20 タスク**の見込み。brief の分割の基準（20 を超えたら 2 つに分ける）にはかからない。持ち越しの推奨を全部採ると上の端（20）に寄る。
  - 危険は中くらい。1・2 番はコルーチンと行フックをまたぐ状態を直すので、振る舞いを固めるテストを先に置く必要がある。ほかは既存の型の中で閉じ、外のクレート（`pasta_shiori` など）は消す API を使っていない。

## 監査のやり直し（2026-10-19 の控えとの突き合わせ）

`/ponytail:ponytail-audit` の観点（正しさ・壊れ方・負荷・テストの無い危ないコード・速さ・無駄）で、`crates/pasta_lua/src/debug` の本番のコード（テストを除いて約 8,080 行・36 ファイル）を読み直した。プラグインのスキルの手順（`~/.claude/plugins/.../ponytail-audit/SKILL.md`）を読み、その観点でコードを読んで確かめた。想定した負荷は「作者 1 人が自分の PC でゴーストを 1 体デバッグする」（待ち受けは 127.0.0.1・1 接続）。cargo test と clippy は走らせていない（テストの本数は `#[test]` を数えて 337。brief の 333 本から 4 本増えている）。「使われない」の判断は、`crates/` 全体の grep による。

### 控えの指摘の今

| # | 指摘 | 今 | 今の場所 |
| - | ---- | -- | -------- |
| 1 | Lua 表示の間、`.pasta` のブレークポイントが死ぬ | 残る | `wiring/inbound.rs:194`（`source_map.pasta_active() && is_pasta_source(..)`。分岐は 185-201）。`.lua` 提示では `.pasta` のパスが `breakpoints.set_breakpoints` へ行き、`chunk` に `.pasta` のパスが入る → フックのチャンク名と一致しないまま `verified: true` |
| 2 | ステップが抜けない | 残る | `session/stop_loop.rs:118-121`（`Disconnect` が `mode` を戻さない）・`:206-207`（`recv()` の `Err` も同じ）・`session/mod.rs:155`（`thread: ThreadId`）・`types.rs:198-210`（`ThreadId(usize)` は `lua_State` の番地）・`stop_loop.rs:287-296`・`session/stepping.rs:95-97`（`cur_thread != thread` で飛ばすだけで、終わったコルーチンを見ない） |
| 3 | 大きい整数の表示が誤る | 残る | `inspect.rs:381-391`（`n as i64` は 385 行）。brief の再測定のとおり |
| 4 | 知らない DAP 要求に応答しない | 残る | `dap/decode.rs:303`（`_ => Decoded::default()`）。マニュアル `internals/debug.md:207`「その他: なし（無視する）」 |
| 5 | 行ごとの割り当て | 残る | `breakpoints.rs:120-133`（毎行ロックを取り、フックのチャンク名と全ブレークポイントのチャンク名を `canonicalize_chunk_name` で作り直す。0 件でもフック側は作る）・`stop_loop.rs:24-34`（`source_and_line` が毎行チャンク名を `String` に写す）・`:250-253`（`.pasta` 提示で毎行 `resolve_current_pasta`）・`source_map/mod.rs:550-553`（`resolve_lua_to_pasta` が毎行チャンク名を作り直す）・`source_map/mod.rs:316-330`（Windows では `replace` と `to_lowercase` の 2 回の割り当て） |
| 6 | ステップ中、thread を比べる前に全スタックを計算する | 残る | `stop_loop.rs:296` → `session/stepping.rs:32-37`（`current_thread_and_depth` が `capture_stack(..).len()` を先に計算する）。比べるのは `stepping.rs:95` |
| 7 | 止まったクライアントで、片付けが固まる | 残る | `transport/mod.rs:436`（フレームの途中では読み取りの時間切れを外す）・`transport/framing.rs:72`（`read_line` に上限が無い）。片付けは `handle.rs:162-167` → `transport/mod.rs:299-301` → `:509` の読み取りスレッドの join で待つ |
| 8 | テストだけ・誰も使わないコード | 残る（1 つは言い方が古い） | `types.rs:218-226`（`LineEvent`。`mod.rs:54` で pub の再輸出）・`types.rs:65-67`（`StopReason::Entry`・`Pause`。作るのは `dap/codec.rs:78-79` の写しとテストだけ）・`error.rs:23-24`（`DebugError::Protocol`）・`types.rs:310`（`SessionEvent::Error`）と `stop_loop.rs:337-340`（`report_error` は `#[cfg(test)]`）・`hook.rs:68`・`:106-125`・`:163`・`:202-221`（`HookHandle`・`PanicCause`・`record_panic_cause`。`enable.rs:200` は返り値を捨てる）・`stop_loop.rs:123-132`（`SetBreakpoints` の分岐。bridge は転送しない）・`playscene.rs:31`・`source_map/scene_index.rs:39`（`#![allow(dead_code)]`）。`insert_file` は **本番から呼ばれている**（`source_map/scene_join.rs:130`）が、働きが無い（下の N5） |
| 9 | scopes の往復が何もしない | 残る | `dap/decode.rs:157-187`（その場で答えたうえで `SessionCommand::Scopes` も送る）・`stop_loop.rs:169-176`（`SessionEvent::Scopes` を作る）・`dap/encode.rs:57-64`（捨てる） |
| 10 | 表示の形の写しが 2 つ | 残る | `session/mod.rs:215`（`source_mode`）・`:224`（`shared_mode: Option<SharedSourceMode>`）・`:278-313`（`with_source_map`・`with_shared_mode`・`effective_mode`）。`enable.rs:196-198` は常に `Some` を渡す |
| 11 | transport の「無効」の道がテストだけのもの | 残る（行番号が古い） | `transport/mod.rs:147-160`（`start(None)`）・`:236-261`（`shutdown`・`join`。`#[allow(dead_code)]`）・`enable.rs:123-125`（`unreachable!`。brief の `L246-256` は古い） |
| 12 | `DebugConfig::resolve` の引数が 8 つ | 残る | `debug/config.rs:97-136`（`#[allow(clippy::too_many_arguments)]` は 97 行）。`attach_source_mode` は本番で常に `None`（`config.rs:171`・`:189`）。`file_source_mode`・`file_sidecar` は `file` から導ける（`config.rs:198-208`） |

brief の再測定の「`stepping.rs L382-387`」（2・6 番）は、`session.rs` を `session/{mod,anchor,stepping,stop_loop}.rs` に分ける前の行番号である。今の `session/stepping.rs` は 167 行で、該当は `current_thread_and_depth`（32-37 行）。

### 新しく見つけた指摘（同じ領域）

正しさの側（作者の手元で困る）:

- **N1. 止まっていないときの `threads` が、次に止まるまで応答されない**（Should fix・4 番と同じ種類）
  - `dap/decode.rs:143-149` は `threads` を `SessionCommand::Threads` としてセッションへ送るだけで、答えない。セッションがコマンドを読むのは停止ループ（`stop_loop.rs:78`）の中だけなので、VM が動いている間の `threads` は次の停止まで待たされる。
  - 例：アタッチして `configurationDone` を送った直後、VSCode はスレッドの一覧を取りに `threads` を送る（【要確認】VSCode の `debugSession.ts` が初期化の後に `fetchThreads` を呼ぶ）。ブレークポイントに当たるまで応答が無く、コールスタックの欄が空のまま残る。
  - 直し：スレッドは固定（ID 1・`main`。`stop_loop.rs:181-188`）なので、`scopes` と同じくその場で答える。`SessionCommand::Threads`・`SessionEvent::Threads`・`PendingKind::Threads`・`encode_threads` の往復が消える（9 番と同じ形の撤去）。要件 4.4。
- **N2. 受け口が無いと、`pasta/playSceneAt`・`pasta/reloadShiori` に応答しない**（Should fix・4 番と同じ種類）
  - `wiring/inbound.rs:344`・`:412` の `let sink = kick_sink?;` は、受け口が無いと応答を返さずに要求を飲み込む（コメントの「inert」）。
  - 例：`PastaLuaRuntime` を `pasta_shiori` 以外のホスト（受け口を渡さない組み込みやテストの道具）で動かし、VSCode で「▶ シーンを実行」を押すと、VSCode が応答を待ち続ける。本番の `pasta_shiori` は常に受け口を渡す（`actor/instance.rs:150`）ので、作者の手元では起きにくい。
  - 直し：4 番と同じ `error_response(seq, cmd, "unsupported")` を返す。要件 4.3。
- **N9. 対応表が無いと、`.pasta` のブレークポイントが `verified: true` のまま当たらない**（1 番と根が同じ）
  - `wiring/resolver.rs:95-98` は、対応表が無いと `breakpoints.set_breakpoints` へ落ちる。`chunk` に `.pasta` のパスが入り、決して当たらないのに「確かめ済み」と返す。
  - 例：デバッグを有効にしたが、ローダーがソースマップを作れなかった（全ファイルの変換に失敗した、など）とき、VSCode のブレークポイントは塗りつぶしの丸のまま止まらない。
  - brief の Approach（「表示の形でなく対応表の有無で決める」）をそのまま入れると、対応表が無いときはこの道に落ちる。`verified: false` にするのが正直である。要件 1.4（持ち越し 1）。
- **N13. UNC の URI では、位置からシーンを引けない**（Nice to have・正しさ）
  - `playscene.rs:141-162` の `strip_file_scheme` は、`file://` の後の最初の `/` までを捨てる。`file://server/share/ghost/master/dic/会話.pasta`（ネットワークのフォルダにあるゴースト）は `/share/ghost/...` になり、ホスト名が消えてシーンが見つからない。
  - VSCode の UNC のファイルの URI はこの形になる。ゴーストをネットワークのフォルダに置く作者は少ないが、直しは小さい（Windows ではホスト名を `\\server` として残す）。要件 4.6（持ち越し 8）。
  - 【要確認】ローダーが UNC のゴーストのフォルダで作る `.pasta` のキー（`canonicalize_chunk_name` の後で `//server/share/...`）と、直した後の URI の正規化が一致すること。

無駄の側（挙動を変えない撤去）:

- **N3. `Decoded::reload_shiori` は本番で読まれない**（`dap/decode.rs:92`・`:290-293`）。wiring は要求の名前の文字列で見分ける（`inbound.rs:405`）。読むのはテスト（`dap_play_scene_tests.rs`）だけ。要件 7.8。
- **N4. 死んだコードを隠す抑止の属性が、brief の 4 か所のほかに 6 か所ある**。`dap/mod.rs:99`・`:104`（`Decoded`・`RELOAD_SENTINEL` の `pub use`）・`session/stepping.rs:17-18`（doc のリンクのためだけの `use SourceMap`）・`source_map/mod.rs:518-521`（`scene_index()` は使い手が 0）・`:638-639`・`:647-648`（再輸出）。brief の分は `hook.rs:111`・`:121`・`playscene.rs:31`・`scene_index.rs:39`・`transport/mod.rs:242`・`:256`。要件 7.6 で全部を 0 にする。
- **N5. `insert_file` は呼ばれているが、働きが無い**。`scene_join.rs:130` が毎ファイル呼ぶが、空の項目があっても無くても `scene_at` は `None` を返す（`scene_index.rs:123-152`。`files.get(..)?` か、空の `range` か）。その上のコメント「未検出を正しく返すため」は誤り。brief の「誰も使わない」は言い方が古い。要件 7.7。
- **N6. `DebugConfig::from_file` はテストだけが使う**（`config.rs:180-191`）。本番は `from_env` だけ（`loader/mod.rs:214`・`runtime/runtime_config.rs:225`）。使うのは `debug_mod_tests.rs:468`・`tests/loader/config_test.rs` の 5 か所。12 番の直しと一緒に整理する。要件 7.9（持ち越し 5）。
- **N7. `DebugHandle` の encoder の join handle は、捨てるためだけに持つ**（`handle.rs:69`・`:175` の `let _ = self.encoder_handle.take();`）。`enable.rs:245-250` で作った時点で捨てても同じ。`DebugHandle::new` の引数は 6 つで、clippy の `too_many_arguments`（7 を超えると出る）にかからないのに許可が付いている（`handle.rs:101`）。要件 7.9。
- **N8. `capture_stack`・`capture_variables` の `lua` の引数は使われない**（`inspect.rs:123`・`:242` の `let _ = lua;`）。呼び手は `stop_loop.rs:144`・`:154`・`stepping.rs:35` とテスト。要件 7.9。
- **N11. テストだけが使う対応表の公開の問い合わせ**（Nice to have）。`ChunkSourceMap::new`・`len`・`is_empty`・`lua_lines_for_pasta`（`source_map/mod.rs:90-92`・`:128-135`・`:152-158`）と `MapBuilderSink::chunk_name`（`:231-233`）。`lua_lines_for_pasta` は、本番の逆引き（`SourceMap::resolve_pasta_to_lua`）と同じことを、毎回の全件走査でするもう 1 つの道である。pub だが、`crates/` の外の使い手は無く、README のとおり予告なく消せる。要件 7.10（持ち越し 7）。
- **N12. `canonicalize_pasta_file` は `canonicalize_chunk_name` をそのまま呼ぶだけ**（`source_map/mod.rs:346-349`。呼び手は `mod.rs` の 3 か所と `scene_index.rs` の 2 か所）。要件 7.10。

説明の誤り:

- **N10. `PastaLuaRuntime::debug_source_mode()` の説明が誤り**（`runtime/exec.rs:78-88`）。「`DAP attach 引数 > env > ...` を映す」と書くが、読むのは `enable` のときの `DebugConfig::source_mode` で、そこには `attach` の値が入らない（`from_env` は `None` を渡す）。実行時の切り替えも映らない。テスト `tests/runtime/runtime_toggle_e2e_common.rs:324-326` のコメントは正しく「BAKED 解決を返す」と書いている。要件 8.4。

### 指摘にしなかったもの（記録）

- **切断した後に再びアタッチできない**：待ち受けは 1 回の起動につき 1 接続（`transport/mod.rs:369` で listener を捨てる）。`troubleshooting.md:34`・`:72` に書いてある決まりで、指摘にしない。
- **切断した後もブレークポイントと行フックが残る**：再接続できないので、残ったブレークポイントに当たっても `stop_loop` はイベントの送り先が無く、`recv()` がすぐ `Err` を返して進む。費用（行フック）は残るが、デバッグを有効にしている間の費用で、2 番の直し（Running に戻す）の後は止まらない。
- **動いている間の提示モードの切り替えは、次の停止で `stopped` を重ねて送る**：`RefreshPresentation` はコマンドの通り道に積まれ、次の停止で `stopped` をもう 1 回送る。VSCode はコールスタックを取り直すだけで、誤った結果にならない。
- **`DebugHandle` の片付けの 30ms の待ち**（`handle.rs:149`）：`terminated` を送り切るための待ちで、デバッグが有効なときの unload だけにかかる。上限があり、指摘にしない。
- **`runtime/exec.rs` の `debug_source_mode()` を消すこと**：本番で使われないが、`runtime_toggle_e2e_*`・`source_map_handoff_test.rs` の 5 ファイルが「ホストの側から見た初期の提示モード」を確かめるのに使う。ランタイムを組み立てて確かめる公開の API の 1 つで（`ponytail-lua-loader` の「指摘にしなかったもの」と同じ扱い）、説明だけ直す。
- **`setBreakpoints` の行番号の `as u32`**（`dap/codec.rs:101`）：巨大な行番号は折り返すが、送り手は 127.0.0.1 の VSCode だけで、起きる場面が無い。
- **シーンの外（`pasta_scripts` のメインスレッドの一番外の関数）の最後の行でのステップ**：一番外の関数が Rust へ戻ると、次の SHIORI 要求の最初の行が同じスレッド・同じ深さになり、そこでステップが止まりうる。作者が `pasta_scripts` の中をステップすることはまれで、`.pasta` 提示では対応の無い行として通り過ぎる。【Research Needed】設計で起きる条件を確かめ、起きるなら 2 番の直し（終わったら Running）と同じ形で拾うか、`shortcut:` の注釈にするかを決める。

## brief の読み方（食い違いの解き方）

- 「ステップは `ThreadId` の代わりに `mlua::Thread` を持つ」（Approach）は直し方の案であり、要件には書かない。要件 2 は「終わったら continue と同じ」「番地の使い回しで止まらない」「切断で Running」を振る舞いで書いた。
- 「`play_scene_at_error` も `error_response` に寄せる」（Approach）は、要件 4.5 の「失敗の応答は 1 つの形」で書き、理由の文は変えない。
- 「ブレークポイントの訳は、対応表の有無で決める」（Approach）をそのまま入れると、対応表が無いときに N9 の道へ落ちる。要件 1.4 で `verified: false` を推奨の形で書いた（持ち越し 1）。
- brief の「Out of Boundary」にある `debug_source_mode()` は「拾うかはギャップ分析で決める」なので、説明の直しだけ拾った（要件 8.4）。
- 8 番の `insert_file` は「誰も使わない」ではなく「呼ばれるが働きが無い」。消す結論は同じ。

## マニュアル

| 頁 | 行 | 直す中身 | 要件 |
| -- | -- | -------- | ---- |
| `book/src/internals/debug.md` | 72 | `DebugConfig` の行の `from_file`（N6 で消すなら） | 7.9・8.2 |
| 同 | 75 | `DebugError` の `Protocol` | 7.1・8.2 |
| 同 | 96 | 型の行の `ThreadId` | 2・8.2 |
| 同 | 136・142・156 | `Transport::start` の `None` の道・セッションへの `cfg.source_mode` の注入（写し） | 7.3・7.4・8.2 |
| 同 | 160-170 | 片付け（止まったクライアントでも終わる・ヘッダの上限） | 6・8.2 |
| 同 | 196-211 | 要求ごとの扱いの表（`threads`・`scopes`・「その他」・受け口の無い要求）と `SessionEvent::Error` の行 | 4・7.2・8.1 |
| 同 | 224 | `HookHandle` | 7.1・8.2 |
| 同 | 228-246 | 行ごとの判定（正規化の時機）とスレッドの比べ方（`lua_State` の番地） | 2・5・8.2 |
| 同 | 258-274 | 停止ループの表（`Disconnect`・`Scopes`・`Threads`・`SetBreakpoints`・チャネルの切断） | 2.5・7.1・7.2・8.2 |
| 同 | 280・284-287 | `should_pause` の説明と、`setBreakpoints` の適用の表（「`pasta_active` で」→ 対応表の有無で・対応表の無い `.pasta`） | 1・5・8.2 |
| 同 | 294 | `capture_variables` の数の表記（必要なら） | 3 |
| `book/src/debug/source-level.md` | 118 | 約束は変えない（要件 1 で本当にする）。対応表が無いときの `verified: false` を 1 行足すなら同じ頁か `troubleshooting.md` | 1.4・8.3 |

- `internals/debug.md:207`「その他: なし（無視する）」は roadmap の 2026-11-02 の棚卸がこの spec へ回した行（`roadmap.md:221`）。
- `book/src/debug/*.md` のうち `vscode-setup.md`・`dev-actions.md` などの VSCode 拡張の側の節は `ponytail-editor-tooling` が触る。この spec は、上の表の行だけを触る。

## Requirement-to-Asset Map

| 要件 | 今の部品 | ギャップ |
| ---- | -------- | -------- |
| 1 ブレークポイントの訳 | `wiring/inbound.rs:185-214`・`wiring/resolver.rs:87-149`・`breakpoints.rs` | **Missing**：分岐の条件（提示モード→対応表の有無）、対応表が無いときの `verified: false` |
| 2 ステップの状態 | `session/mod.rs:139-172`（`RunMode::Stepping`）・`stop_loop.rs`・`stepping.rs`・`types.rs:198-210` | **Missing**：コルーチンの同一性を番地でなく参照で持つ・終わったら Running・切断で Running。**Constraint**：`RunMode` は `RefCell` の中で `clone` される。参照を持つ型が `!Send` でも、セッションは VM のスレッドの中だけで使うので構わない（【Research Needed】mlua 0.12 の `Thread` の比べ方と状態の読み方） |
| 3 数の表示 | `inspect.rs:381-391` | **Missing**：`format!("{n}")` へ寄せる（Rust の `f64` の表示は `3.0` を `3`、`1e20` を `100000000000000000000` と書く） |
| 4 応答 | `dap/decode.rs`・`dap/mod.rs:205-229`・`wiring/inbound.rs:328-433`・`playscene.rs:141-162` | **Missing**：共通の失敗の応答・`threads` のその場の応答・受け口の無い要求の応答・UNC の URI |
| 5 行ごとの費用 | `breakpoints.rs:75-213`・`types.rs:127-154`・`stop_loop.rs:24-34`・`:238-327`・`source_map/mod.rs:550-553` | **Missing**：登録のときの正規化・0 件のときの早い戻り・チャンク名の作り直しを避ける。**Constraint**：`Breakpoint` は pub の型で、`register` のキー（提示ソース）と判定のキー（実行座標）の 2 段を保つ |
| 6 片付け | `transport/mod.rs:337-510`・`transport/framing.rs:65-133` | **Missing**：フレームの途中の時間切れ・ヘッダの行の上限。**Constraint**：DAP のフレームは TCP の断片に分かれうる（今は途中で時間切れを外すことで断片を正しく読む） |
| 7 撤去 | 上の表の 8〜12 番と N3〜N8・N11・N12 | **Missing**：撤去そのもの。**Constraint**：pub の型（`LineEvent`・`StopReason`・`DebugError`・`ChunkSourceMap` の問い合わせ）を消す。README の断り書きのとおり、予告は要らない |
| 8 マニュアル・説明 | 上の「マニュアル」・`runtime/exec.rs:78-88` | **Missing**：行の書き直し |
| 9 テスト | `debug/*_tests.rs`・`*_e2e*.rs`（約 14k 行）・`tests/runtime/runtime_toggle_e2e_*` | **Constraint**：新しいテストにも `PASTA_DEBUG` の中和（`#[ctor]`）が要る。x86 でも走らせる |

## Implementation Approach Options

### Option A: 既存の部品の中で直す（brief の Approach そのまま）

- 1 番：`route_command` の分岐を `source_map.source_map.is_some() && is_pasta_source(..)` にし、`translate_pasta_breakpoints` の対応表の無い道を `verified: false` にする。
- 2 番：`RunMode::Stepping` に `ThreadId` の代わりにコルーチンの参照を持たせ、`on_line_impl` の先頭で「終わっていたら Running」。`stop_loop` の `Disconnect` と `Err` で `Running` に戻す。6 番は比べる順を入れ替えるだけ。
- 3 番：1 行。
- 4 番：`DapAdapter` に失敗の応答を 1 つ足し、`decode_request` の `_` と `play_scene_at_error`・受け口の無い道がそれを使う。`threads` は `scopes` と同じくその場で答える。
- 5 番：`Breakpoint` に正規化した `chunk` を登録のときに入れ、`should_pause` は 0 件なら早く戻り、比べるのは正規化済みどうし。フックの側の正規化は 1 行に 1 回（0 件なら 0 回）。
- 6 番：`read_frame` を、時間切れを保ったまま断片を読み足す形にするか、フレームの途中にも上限の時間を置く。ヘッダの行は `take(上限)` で読む。
- 7 番：消すだけ。テストは本番の道へ移す。
- ✅ 新しいファイルが要らない。差分の大半が削除になる（brief の「Lean」の向き）。
- ❌ 1・2 番は行フックをまたぐ状態の直しで、e2e のテスト（`wiring_pasta_*_e2e*.rs`）の書き足しが要る。

### Option B: 停止の判定を新しい部品へ切り出す

- `should_pause`・ステップの判定・アンカーをまとめた「停止の判定」の型を作り、行ごとの費用と状態の抜けをそこで一度に直す。
- ✅ 判定の単体テストが書きやすい。
- ❌ 新しい継ぎ目を足す。監査の結論（死んだ継ぎ目と写しを消す）の向きに反し、差分が増える。brief の Approach に無い。

### Option C: 混ぜる

- 正しさ（1〜4・6 番と N1・N2・N9・N13）は Option A で直し、行ごとの費用（5 番）だけ、判定の型の中に正規化済みの表を持たせる小さな整理を許す。
- ✅ 5 番の正規化の置き場所を設計で選べる。
- ❌ 5 番の整理が `Breakpoint`（pub）の形を変えるなら、テストの書き直しが増える。

推す形は **Option A**。5 番の正規化を `Breakpoint` に持たせるか `BreakpointSet` の中だけで持つかは、設計で決める（pub の型を変えない方が差分が小さい）。

## Effort and Risk

- **Effort: L（1〜2 週）**。1 つずつの直しは小さい（各 5〜60 行）が、数が多い（監査の 12＋新しい 13）。撤去の波及（テストの移し替え・マニュアル十数か所）がある。
- **Risk: Medium**。
  - 2 番は、コルーチンの参照を持つことで GC との関わりが変わる（ステップの間、そのコルーチンが回収されない）。中断のまま捨てられたコルーチンの限界は brief のとおり注釈にする。
  - 6 番は、フレームの断片を正しく読むことと、片付けの上限の時間の両立が要る。Windows でソケットの `shutdown` が実行中の受信を解かない（`transport/mod.rs:326-330`）ことに気をつける。
  - ほかは既存の型の中で閉じ、外のクレートは消す API を使っていない（`grep` で確かめた）。
- **タスク数の見積もり: 17〜20**。
  - 1 番（訳の分岐・対応表の無いとき）1〜2、2 番（コルーチンの同一性と終わり・切断）2、6 番は 2 番に含める、3 番 1、4 番（失敗の応答・未対応・受け口・`threads`・UNC）2、5 番 2、7 番 1〜2、8 番（死んだ型・`HookHandle`・分岐・抑止の属性・`insert_file`・`reload_shiori`・小物）2〜3、9 番 1、10 番 1、11 番 1、12 番（`from_file` を含む）1、対応表の問い合わせ（N11・N12）1、マニュアルと説明 1、最後の確かめ 1。
  - brief の見積もり（14〜18）より少し多い。新しい指摘の分である。20 を超えない見込みなので、分けない。設計で 20 を超えたら、brief のとおり正しさ（1〜4・6・7 番と N1・N2・N9・N13）と無駄（5・8〜12 番と N3〜N8・N10〜N12）の 2 つに分ける。

## 設計への申し送り

- **推す形**: Option A。
- **決めること**
  - 2 番：コルーチンの参照の持ち方（`RunMode` の中か、セッションの別の欄か）と、「終わった」の判定の場所（毎行の入口か、ステップの判定の中だけか）。番地の使い回しで偽の停止が起きないことを確かめるテストの組み方（コルーチンを終わらせて GC を回し、新しいコルーチンを作る）。
  - 3 番：`format!("{n}")` に寄せると、負のゼロ（`-0.0`）は `-0` と表示される（今は `n as i64` を通して `0`）。Lua の値として負のゼロが作者の目に触れることはまれなので、要件 3 には書いていない。そのままにするか、`0` にそろえるかを決める。
  - 4 番：失敗の応答の `message` の文（`unsupported` だけか、要求の名前を足すか）。`body.error` を付けるか（DAP では任意）。
  - 4 番：`threads` をその場で答えると、`SessionCommand::Threads`・`SessionEvent::Threads`・`PendingKind::Threads`・`ThreadInfo`・`encode_threads` のどれが残るか。
  - 5 番：正規化した `chunk` を `Breakpoint`（pub）に持たせるか、`BreakpointSet` の内側の表だけに持つか。`.pasta` 提示での `resolve_lua_to_pasta` の作り直しを避ける方法（前の行のチャンク名を覚える・正規化済みのキーを引く口を足す）。
  - 6 番：フレームの途中の読み取りの上限の時間と、ヘッダの行の上限の長さ。
  - 7 番：`DebugConfig::listen` の `Option`（`enabled` と同じことを表す）を残すか。`Transport::start` の引数を `SocketAddr` にすると、`enable.rs:123-125` の `unreachable!` が消える。
  - 7 番：`hook::install` の返り値（`HookHandle` を消した後は `()`）。
- **Research Needed**
  - mlua 0.12 の `Thread` の等しさの比べ方（参照の同一性か）と、終わったコルーチンの状態の読み方（`status()`）。行フックの中で `lua.current_thread()` を取る費用。
  - VSCode が初期化の後に `threads` を送ること（N1 の前提）。
  - UNC のゴーストのフォルダで、ローダーが作る `.pasta` のキーと、直した後の URI の正規化が一致すること（N13）。
  - メインスレッドの一番外の関数の最後の行でのステップが、次の SHIORI 要求で止まるか（「指摘にしなかったもの」の最後）。

## 要件ディスカッションへの持ち越し

- 要件ディスカッション（2026-11-02）で、2（2026-10-19 の決定のとおり）と 3（DAP はすべての要求に応答を求める）は議題にせず、推奨のとおり先にクローズした。残りの 1・4〜9 は開発者が一括で承認し、全部を推奨案のとおり確定した（片付けの上限は 1 秒）。

1. **対応表が無いときの `.pasta` のブレークポイント**（要件 1.4・N9）：`verified: false` で返す、で良いか。
   - 推奨：はい。止まらない座標を「確かめ済み」と言わない。例：対応表が作れなかったゴーストで `会話.pasta` の 12 行に張ると、VSCode の丸が灰色（未確認）になる。
2. **未対応の応答の文**（要件 4.1）：`message: "unsupported"` の 1 語で良いか。
   - 推奨：はい（2026-10-19 の決定のとおり）。例：一時停止のボタンを押すと、VSCode が「unsupported」と通知する。
3. **受け口の無い `pasta/playSceneAt`・`pasta/reloadShiori`**（要件 4.3・N2）：今の「何も返さない」をやめ、未対応と同じ失敗の応答を返して良いか。
   - 推奨：はい。DAP はすべての要求に応答を求める。本番の `pasta_shiori` では起きないので、作者への影響は無い。
4. **止まっていないときの `threads`**（要件 4.4・N1）：その場で固定のスレッドを返して良いか。
   - 推奨：はい。スレッドは 1 つに固定されており、止まるまで待たせる理由が無い。`scopes` と同じ形で往復の道も消える。
5. **`DebugConfig::from_file`**（要件 7.9・N6）：テストだけが使うので消し、テストを `resolve` へ移して良いか。
   - 推奨：はい。12 番の引数の整理と同じタスクで行う。
6. **止まったクライアントでの片付けの上限**（要件 6.1）：1 秒で良いか。
   - 推奨：はい。今の接続の無いときの片付けは数十 ms で、1 秒は「固まらない」をテストで確かめるのに十分な余裕である。
7. **テストだけが使う対応表の問い合わせ**（要件 7.10・N11・N12）：この spec で消して良いか。
   - 推奨：はい。監査の 8 番と同じ種類で、`lua_lines_for_pasta` は本番の逆引きの写しである。テストは `pasta_for_lua`・`resolve_pasta_to_lua` へ移す（1 タスク）。
8. **UNC の URI**（要件 4.6・N13）：この spec で直して良いか。
   - 推奨：はい。同じ領域の正しさの指摘で、直しは数行（Windows でホスト名を `\\server` として残す）。例：`\\nas\ghosts\emo\ghost\master\dic\会話.pasta` を開いて「▶ シーンを実行」を押すと、今は「カーソル下にシーンがありません」になる。
9. **古いタスク番号のコメント**（記録）：コードのコメントには、完了した spec のタスク番号や「平らな `wiring.rs` と同じ中身」のような経緯が多い。直しで偽になったコメントだけを直し、コメントの一斉の書き直しはこの spec の範囲に入れない、で良いか。
   - 推奨：はい。監査の指摘ではなく、一斉に書き直すと差分が大きくなり、直しの差分が読みにくくなる。
