# ギャップ分析: ponytail-shiori

## 測り方

- 日付：2026-11-02。base：`ae04a28b`（brief の再測定の `491cc23c` の後、`pasta_shiori` に変化なし）。
- 読んだもの：`crates/pasta_shiori/src` の全部（`v3.rs` を含む）、`Cargo.toml`、`README.md`、`tests/` の LEGACY を触るファイル（`ffi_*`・`load_failure_visibility`・`path_robustness_e2e`・`scene_kick_*`・`shiori4_v3_entry`・`actor_instance`・`multi_instance`）、`book/src/internals/shiori.md` の該当の節、ルートの `Cargo.toml` の `[profile.release]`。
- 走らせたもの：
  - `cargo test -p pasta_shiori`（`NoDefaultCurrentDirectoryInExePath` を外す）：43 のテストのバイナリがすべて通り、365 本が成功・0 本が失敗。
  - `cargo package --list -p pasta_shiori --offline --allow-dirty`：122 ファイル。`doc/shiori-sample.log`（8,517 行）・`src/util/parsers/test_data/logfile.txt`（3,322 行）・`tests/`（fixtures を含む）・`src/*_tests.rs`・`res/make_icon.py` が入る。
- 想定した負荷：SSP 1 つのプロセス、旧来の入口のインスタンスは 1 つ（版 3 は複数ありうる）、要求はおおむね 1 秒に 1 回（OnSecondChange）。SHIORI の呼び出しはホストの 1 本のスレッドから来る。

## 監査（`/ponytail:ponytail-audit` のやり直し）

**このクレートがすること**：pasta.dll の SHIORI の入口（旧来の HGLOBAL の `load`・`loadu`・`request`・`unload`・`DllMain` と、版 3 の `shiori_get_api`）を持つ。入口は Lua VM を直に触らず、インスタンスごとの専用スレッド（アクター）へ要求をキュー（mailbox）で送り、そのスレッドの上の `PastaShiori` が `pasta_lua` で VM を作って `SHIORI.request` を呼ぶ。負荷は上の想定のとおり。

### Must fix

1. **unload を呼ばずに外されると、SSP が 5 秒止まって落ちうる**（`src/actor/thread.rs` 255〜273 行、`src/windows.rs` 81〜83 行、`src/actor/lifecycle.rs` 105〜107 行）
   - **これは何か**：`DllMain` の FreeLibrary の detach は、アクターに Stop を送って完了の通知（ack）を 5 秒まで待ち、スレッドを手放す（join しない）。
   - **問題**：アクターは ack の**前**に `drop(shiori)` で VM を片付ける（260 行）。辞書が Lua の C モジュールを読んでいると、その解放（`FreeLibrary`）はローダーのロックを要るが、ロックは `DllMain` の側が持っている。アクターは 5 秒止まり、`DllMain` は打ち切って戻り、pasta.dll が外れる。ロックが空いた後、アクターは外れた pasta.dll のコードへ戻って access violation になる。C モジュールが無くても、ack の後のスレッドの終わり（Rust のスレッドの後始末）は pasta.dll のコードで走り、外れる時と競る（窓は狭いが在る）。`DllMain` の中でスレッドの終わりを待つと、スレッドの終わりもローダーのロックを要るので必ず打ち切りになる。
   - **直し**：「上限つきで待つ」だけでは閉じない。アクタースレッドが生きている間は pasta.dll が外れない形（スレッドがモジュールの参照を 1 つ持ち、`FreeLibraryAndExitThread` で終わる、など）が要る。下の「選択肢 D」を参照。
   - **飛ばすと**：`unload` を呼ばないホスト・呼び忘れの道で、ホストのプロセスごと落ちる。
2. **旧来の入口でフォルダが無いと、理由の無い 500 になる**（`src/shiori.rs` 102〜106 行）
   - **これは何か**：`PastaShiori::load` の最初の検査。
   - **問題**：`load_dir_path.exists()` が偽のとき、ロガーを作る前に `last_load_error` を立てずに `Ok(false)` を返す。以後の GET は `MyError::NotInitialized` の 500（`X-ERROR-REASON: Not initialized error`）になり、どのフォルダかが分からない。`error!` のログも、登録簿の項目にロガーが無いので捨てられる。例：`load`（ANSI）に ANSI で表せない文字のパスが来ると、`?` に化けた存在しないパスになる。
   - **直し**：`last_load_error` にフォルダのパスを入れて `Ok(false)` を返す（1〜2 行）。500 の理由は `Load error: …` の形で出る。
   - **飛ばすと**：作者は原因（文字化け・設置ミス）を辿れない。
3. **`Key:` の行を含む要求が、黙って 204 になる**（`src/util/parsers/req_parser.pest` 44 行、`src/actor/instance.rs` 198〜204 行）
   - **これは何か**：見出しの区切りの文法 `_tag = _{ ": " }`。
   - **問題**：`Reference0:` で行が終わると `key_value` が解けず、`req` 全体が解けない。旧来の入口では `determine_method` が `None` を返し、`ActorInstance::request` が `default_204()` を返す。ログは何も出ない。SSP の記録 `doc/shiori-sample.log` の 47 行（`Reference0:`・OnInitialize）・720 行（`Reference2:`・OnNotifyUserInfo）・1236 行（`Reference1:`・GET OnTranslate）に実例がある（brief は 3 つとも `Reference0:` と書いたが、720・1236 行は別の番号）。`Reference4: ` のように空白で終わる行（1807 行ほか）は今も解ける。版 3 では `Unreadable` で 400 になり、debug のログだけが出る。
   - **直し**：`_tag = _{ ":" ~ " "? }`（1 行）。値は `remain` が空を許すので `""` になる。解けない要求の警告は、旧来の入口では FFI のスレッドにログの文脈（`LoadDirGuard`）が無いので、張ってから出す（`v3.rs` の `reject` と同じ形）。
   - **飛ばすと**：OnInitialize などがゴーストに届かず、作者には原因の手がかりが無い。

### Should fix

4. **再読み込みのリーク検査の道具が、出荷のコードに入っている**（`src/actor/teardown.rs` 72〜105 行・189〜370 行、`Cargo.toml` 34〜37 行）
   - **これは何か**：`ReloadProbe`・`ReloadReport`・`LeakMetric`・`ResourceSample`・`sample_resources`・`user_objects_from`、それらだけが使う `teardown::teardown_actor`（いつも detach）と `TeardownReport::is_clean`。
   - **問題**：使うのはテスト（`tests/actor_reload_leak_test.rs`・`actor_teardown_test.rs` ほか）だけ。`windows-sys` の `Win32_System_Threading` も、通常の依存でこのためだけに有効にしている。リリースは LTO なので pasta.dll のバイナリには残らない見込みだが、クレートのコード・公開 API・パッケージには入る。
   - **直し**：テストの共通の部品（`tests/common/`）へ移し、機能は dev-dependencies へ移す。
   - **飛ばすと**：読み手が本番の後始末の道と、テストの道（いつも detach）を取り違える。
5. **kick の e2e のテストが、本番で kick が通らない道を試している**（`src/actor/lifecycle.rs` 136〜160 行、`src/actor/instance.rs` 217〜219 行）
   - **これは何か**：`lifecycle::kick_sink()` は旧来の置き場 `LEGACY` へ kick を積む窓口で、コメントに「試験と互換のために残す・本番の kick はここを通らない」とある。
   - **問題**：本番の kick は、インスタンスごとの `kick_sink_for`（mailbox の送信端の弱い参照）を通る。`tests/scene_kick_e2e_test.rs`・`scene_kick_gate_e2e_test.rs`・`scene_kick_multibeat_e2e_test.rs`・`scene_kick_preempt_e2e_test.rs` と `src/shiori_lifecycle_tests.rs` は、本番に無い道を通して確かめている。`ActorInstance::kick` も、この窓口とテストだけが使う。
   - **直し**：テストを本番の送り先（`kick_sink_for` と同じ道）へ寄せ、`kick_sink`・`kick_into` を消す。
   - **飛ばすと**：本番の kick の道が壊れても、テストは通り続ける。
6. **テストだけのディスパッチの写し**（`src/actor/marshaling.rs` 283〜297 行）
   - **これは何か**：`marshaling::marshal_request(tx, seq, request)`。コメントに「テスト用」とある。
   - **問題**：`ActorInstance::request`（`instance.rs` 198 行）と同じ判定の写し。brief の 8 番は「閉じた」としたが、写しの片方が残っている。使うのは `tests/actor_marshaling_test.rs` の 1 か所だけ。
   - **直し**：消して、テストは `ActorInstance` か `marshal_get`・`marshal_notify` を直に使う。
   - **飛ばすと**：判定を直すとき、写しを直し忘れる。
7. **204 を作る関数が 2 つ**（`src/shiori.rs` 408〜415 行、`src/actor/marshaling.rs` 102〜109 行）
   - **これは何か**：`PastaShiori::default_204_response`（ヘッダ 2 つ）と `marshaling::default_204`（ヘッダ 3 つ）。
   - **問題**：`SHIORI.request` が未定義のときだけ、`SecurityLevel` の無い 204 が返る。
   - **直し**：`default_204()` に寄せる（brief で受け入れ済み）。
   - **飛ばすと**：経路によってヘッダが違う 204 が混ざる。
8. **コードページの層が、ほぼテストだけの足場**（`src/util/hglobal/enc.rs` 全部、`src/util/hglobal/windows_api.rs` 4〜72 行・146〜225 行、`windows_api.rs` 123〜128 行）
   - **これは何か**：`Encoder`・`CodePage` のトレイト（実装が 1 つ）、OEM のコードページ、`string_to_multibyte`・`wide_char_to_multi_byte`・`WC_*`（`#[cfg(test)]`）。
   - **問題**：本番に要るのは `multi_byte_to_wide_char(CP_ACP, MB_ERR_INVALID_CHARS, bytes)` だけ。123〜128 行は `Vec::with_capacity` の後に `set_len` で未初期化の領域を作る（`#[allow(clippy::uninit_vec)]`）。roadmap の `string_to_multibyte` の件もここ。
   - **直し**：`to_ansi_str` から `multi_byte_to_wide_char` を直に呼び、`vec![0u16; len]` にする。トレイトとテストだけの変換を消す。
   - **飛ばすと**：使われない変換を読み、試し、直し続ける。
9. **テストの中のパーサの写しと、使われないテストデータ**（`src/util/parsers/req.rs` 14〜113 行、`src/util/parsers/test_data/logfile.txt`、`Cargo.toml` に `exclude` が無い）
   - **これは何か**：`ShioriRequest`（テスト専用の約 100 行の解析器）。`logfile.txt` はどのテストも読まない（`include_str!` は `shiori2-1.txt`・`shiori3-1.txt`・`shiori3-2.txt` だけ）。
   - **問題**：テストが出荷のパーサ（`lua_request::parse_request`）でなく、写しを確かめている。`logfile.txt`（58 KB）と `doc/`（SSP の記録）が crates.io のパッケージに入る。
   - **直し**：写しを消して `req_parser.rs` のテストと `lua_request` のテストで同じ性質を確かめる。`logfile.txt` を消し、`exclude = ["doc/"]` を足す。
   - **飛ばすと**：パーサを直すとき、写しのテストが本物と食い違っても気づかない。
10. **メソッドの判定が 2 つ**（`src/actor/marshaling.rs` 78〜129 行と 131〜165 行）
    - **これは何か**：旧来の入口の `determine_method`（`ShioriMethod`）と、版 3 の `classify_request`（`RequestKind`）。
    - **問題**：同じ文法で同じ判定を 2 度書いている。違いは SHIORI/2.x の扱いだけ（旧来は Lua へ届け、版 3 は 400）。
    - **直し**：`classify_request` 1 つにし、旧来の入口は `Shiori2` を、その要求のメソッドとして扱う（2.x の GET は今どおり Lua へ）。形は設計で決める。
    - **飛ばすと**：文法を直したとき、片方の判定だけが変わる。
11. **いつも 0 の `hinst` を 5 段通している**（`src/actor/lifecycle.rs` 45〜67 行、`src/actor/instance.rs` 107〜151 行、`src/actor/thread.rs` 133〜172 行、`src/shiori.rs` 27・99 行）
    - **これは何か**：`spawn_actor`→`spawn_actor_with_teardown_timeout`→`ActorInstance::spawn`・`spawn_with`→`spawn_actor_thread_with`→`PastaShiori::load` の `hinst`。
    - **問題**：呼び出しはすべて 0（`windows.rs` 192 行・`v3.rs` 59 行・テスト）。`PastaShiori.hinst` は書くだけで、読むのは単体テストだけ。Lua の `SHIORI.load(hinst, load_dir)` は 0 を受け取る。
    - **直し**：アクターの道から引数を抜き、`PastaShiori::load` だけが Lua へ 0 を渡す（テストが `PastaShiori::load` に値を渡す形は残すか、設計で決める）。
    - **飛ばすと**：読み手が「どこかで本物の hinst が来る」と探す。
12. **使われない部品と、実装が 1 つの抽象**
    - **これは何か**：`Reply`（variant が 1 つ・`mailbox.rs` 66〜70 行）、`_GRAMMAR` の `include_str!`（`req_parser.rs` 1〜2 行。pest_derive が文法の変化を見ている）、文法の `sp`・`eol`・`key_sep`（`req_parser.pest` 48〜50 行）、`ShioriString` の `unsafe impl Send/Sync`（`hglobal/mod.rs` 23〜29 行。スレッドを越えない）、`MyError::Poison` と `From<PoisonError>`（`error.rs` 16〜17 行・41〜45 行。本番に Mutex が無い）、`Shiori` トレイト（`shiori.rs` 10〜13 行。実装は `PastaShiori` だけ）。
    - **問題**：どれも使われないか、素通しである。`unsafe impl` は要らない約束を書いている。
    - **直し**：消す。`Shiori` のメソッドは `PastaShiori` の固有のメソッドにする（テストの `use pasta::Shiori` を外すだけ）。
    - **飛ばすと**：読む量と、`unsafe` の監査の量が減らない。
13. **アクターの公開 API に、テストだけの部品が混ざる**（`src/actor/mailbox.rs` 88〜153 行、`src/actor/thread.rs` 55〜88 行・133〜137 行・158 行）
    - **これは何か**：`ActorMsg::get`・`notify`・`kick` の作り手（本番は variant を直に作る）、`ActorThread::actor_thread_id`・`debug_local_addr`（本番は読まない。そのためだけに ready のチャネルが組を運ぶ）、`spawn_actor_thread`（`expect` で panic する包み）、`ActorMsg` の `#[non_exhaustive]`。
    - **問題**：使うのはテストだけ。
    - **直し**：テストだけの作り手と包みを消すか、テストの側へ移す。観測の値（スレッド ID・DAP の待受）は、テストが要るなら残し方を設計で決める。
    - **飛ばすと**：本番の形がどれか分かりにくいまま。

### Nice to have

14. **SHIORI/2.x の版の数の、起きない分岐**（`src/lua_request.rs` 95〜101 行）：文法は `ver = @{ ASCII_DIGIT }`（1 桁）なので、`nums < 0` と `nums > 9` は起きない。`nums + 20` だけにする。
15. **同じ前提を 2 度確かめる所と、古くなったコメント**（`src/shiori.rs` 162〜168 行と 304 行、389 行の `_runtime`、`src/util/hglobal/mod.rs` 60〜62 行・170〜176 行）：`request` がランタイムの有無を確かめた後、`call_lua_request` がもう一度確かめる。`MyError` に variant を足すと semver-major になる、というコメントは 2026-10-19 の決定（pub API を消してよい）と食い違う。

ほかに小さいもの（コメントの task 番号の名残り、テストからの FFI の呼び方が再輸出と `extern` の 2 通りある、など）は 6 件ほど省いた。

- **Verdict**：1 番（`DllMain` の道）を先に直す。2・3 番は 1〜2 行で直る。
- **Lean**：約 -4,100 行（`logfile.txt` 3,322 行、`teardown.rs` の約 230 行、`enc.rs`・`windows_api.rs` の約 230 行、`req.rs` の約 100 行、kick の窓口・写し・判定の約 120 行、ほか）、通常の依存の機能 -1（`Win32_System_Threading`）。パッケージから `doc/` の 8,517 行。
- **Not checked**：SSP の実機（`FreeLibrary` の道の再現）、i686 のビルド、`tests/` のうち LEGACY を触らないファイルの中身、`src/util/shiori.md`（SHIORI の仕様の控え。マニュアルから参照される）。

## brief の控えとの突き合わせ

| 番号 | brief（10-19 の控え・11-02 の再測定） | 今（ae04a28b） | 状態 |
| ---- | ------------------------------------- | -------------- | ---- |
| 1 | unload と FreeLibrary の競り。unload の道は閉じ、DllMain の道が残る | `instance.rs` 240〜261 行の `finish` が ack の後に join（unload の道）。`thread.rs` 260 行の `drop(shiori)` が ack の前。`lifecycle.rs` 105 行の `teardown_actor_detached` | unload の道は閉じた。DllMain の道は残る。さらに、C モジュールが無くても ack の後のスレッドの終わりが外れる時と競ることを確かめた（監査 1） |
| 2 | `shiori.rs` 103〜105 行 | 102〜106 行 | 残る |
| 3 | `req_parser.pest` 44 行 | 44 行。旧来は `instance.rs` 198〜204 行で黙って 204、版 3 は 400＋debug | 残る |
| 4 | 閉じた（`windows.rs` 119・139・232・277 行） | 同じ | 閉じた |
| 5 | ほぼ消えた。`tests/ffi_*` で確かめ直す | `windows_tests.rs` 30 行の `FFI_GLOBAL_STATE` 1 つ。LEGACY を触る統合テストのバイナリは、`#[test]` が 1 本か、錠（`multi_instance_test.rs` の `serial`）を取る。`actor_instance_test.rs`・`shiori4_v3_entry_test.rs` は LEGACY を触るのが 1 本だけ | 閉じた |
| 6 | `enc.rs` 102 行・`windows_api.rs` 126・191 行の `set_len` | `set_len` は `windows_api.rs` 126 行（本番）と 191 行（テストだけ）。`enc.rs` は 102 行がファイルの終わりで `set_len` は無い | 残る（場所を直した） |
| 7 | `teardown.rs` 197〜290 行 | 72〜105 行・189〜370 行（`is_clean`・`teardown_actor`・`sample_resources` を含む） | 残る（範囲が広い） |
| 8 | `marshal_request` の写しは閉じた。204 の写しが残る（`shiori.rs` 409 行・`marshaling.rs` 102 行） | 204 の写しは同じ。`marshaling.rs` 283〜297 行にテストだけの `marshal_request` が残る | 半分残る |
| 9 | 無くなった見込み | `request` は `*len` を 1 度だけ書く（`windows.rs` 304 行） | 閉じた |
| 10 | 残り 5 つ（`Reply`・`_GRAMMAR`・文法 48〜50 行・`unsafe impl`・`Poison`） | 同じ場所。`mailbox.rs` 67 行・`req_parser.rs` 2 行・`hglobal/mod.rs` 28〜29 行・`error.rs` 17 行 | 残る |
| 11 | `shiori.rs` 10 行 | 10〜13 行 | 残る |
| 12 | `logfile.txt`・`doc/`・`exclude` が無い | 同じ。`req.rs` のテストの写し（14〜113 行）も残る（再測定の「残る」から漏れていた） | 残る |

## 新しく見つかった指摘

| 番号 | 中身 | 場所 | 監査 | 要件 |
| ---- | ---- | ---- | ---- | ---- |
| N1 | 旧来の置き場へ kick を積む窓口（`kick_sink`・`kick_into`・`ActorInstance::kick`）はテストだけが使い、kick の e2e テストは本番に無い道を試している | `lifecycle.rs` 136〜160 行、`instance.rs` 217〜219 行 | 5 | 6.2・6.9 |
| N2 | テストだけのディスパッチの写し `marshaling::marshal_request` | `marshaling.rs` 283〜297 行 | 6 | 6.2 |
| N3 | いつも 0 の `hinst` を 5 段通し、`PastaShiori.hinst` は書くだけ | 上の 11 | 11 | 7.3 |
| N4 | メソッドの判定が 2 つ | `marshaling.rs` 78〜165 行 | 10 | 7.4 |
| N5 | 起きない分岐（2.x の版の数） | `lua_request.rs` 95〜101 行 | 14 | 7.5 |
| N6 | アクターの公開 API のテストだけの部品（`ActorMsg` の作り手・観測の getter・`spawn_actor_thread`・`#[non_exhaustive]`） | 上の 13 | 13 | 6.2 |
| N7 | 単一の実装の変換のトレイト（`Encoder`・`CodePage`） | `enc.rs` 26〜57 行 | 8 | 6.3 |
| N8 | 通常の依存の `Win32_System_Threading` はリーク検査だけが使う | `Cargo.toml` 34〜37 行 | 4 | 6.8 |
| N9 | 旧来の入口で解けない要求の警告は、FFI のスレッドにログの文脈が無いと捨てられる。版 3 の弾いた要求のログは debug | `instance.rs` 198 行、`v3.rs` 38〜42 行 | 3 | 3.6〜3.8 |
| N10 | パッケージに `tests/`（fixtures を含む）・`src/*_tests.rs`・`res/make_icon.py` も入る | `cargo package --list` | — | 6.10（Q7） |
| N11 | 同じ前提の 2 度の確認と、pub API の決定と食い違うコメント | 上の 15 | 15 | 7.8 |

## 要件と資産の対応

| 要件 | 今の資産 | ギャップ |
| ---- | -------- | -------- |
| 1.1・1.6〜1.8 | `ActorInstance::finish`（join）・`DllMain` のプロセスの終了の分岐・`teardown` | 無し（保つ）。テストは `actor_instance_test.rs`・`multi_instance_test.rs`・`windows_tests.rs` |
| 1.2・1.3 | `teardown_actor_detached`・`JoinPolicy::Detach`・`thread.rs` の片付けの順 | **Missing**：外されている間のモジュールの寿命の扱いが無い。**Unknown**：`FreeLibraryAndExitThread` と Rust の `JoinHandle::join`（スレッドが閉包から戻らないと結果の受け渡しが成り立たない）の両立。**Constraint**：`DllMain` の中ではスレッドの終わりを待てない |
| 1.4・1.5 | 版 3 は `PastaV3` の Drop が後始末。`DllMain` は LEGACY だけを片付ける | **Unknown**（Q1・Q2） |
| 2 | `PastaShiori::load`・`last_load_error`・`MyError::Load` | **Missing**：フォルダが無いときの理由（1〜2 行）。`load_failure_visibility_test.rs` に段を足せる |
| 3.1〜3.5 | `req_parser.pest`・`lua_request::parse_request`・`classify_request`・`determine_method` | **Missing**：文法の 1 行。`doc/shiori-sample.log` の 3 要求を固めるテスト（`doc/` はパッケージから外すので、テストに要る分は文字列で持つ） |
| 3.6〜3.8 | `v3.rs` の `reject`（debug）・`LoadDirGuard` | **Missing**：旧来の入口の警告と文脈。**Unknown**：pest の誤りから行と列を取る形（`pest::error::Error::line_col`） |
| 4 | `windows.rs` の型・`FFI_GLOBAL_STATE` | 無し（保つ） |
| 5 | `default_204`・`default_204_response`・`byte_invariant_test.rs` | **Missing**：寄せるだけ。`SHIORI.request` 未定義の 204 を固めるテストが要る |
| 6 | `teardown.rs`・`lifecycle.rs`・`marshaling.rs`・`hglobal/`・`req.rs`・`Cargo.toml` | **Missing**：テストの共通の部品の移し先（`tests/common/` は在る）。**Constraint**：統合テストは `pub` しか呼べない（`#[cfg(test)]` は効かない） |
| 7 | 上の 10〜14 の場所 | **Missing**：撤去だけ。**Constraint**：旧来の入口の 2.x の扱いを変えない |
| 8 | `book/src/internals/shiori.md` 93 行（`ReloadProbe`）・144 行（`hinst`）・183〜211 行（起動と失敗の理由）・213〜246 行（判定と 204）・271 行・282〜327 行（DllMain）・561 行（204 の表）・621〜629 行（不変条件）、`README.md` 27〜70 行 | **Missing**：書き直しだけ |
| 9 | CI・ローカルの関門 | 無し |

## 実装の選択肢

### 全体の進め方

- **A：今のファイルの中で直す（推奨の土台）**：撤去と 1〜2 行の直しが大半で、新しい部品はほぼ要らない。テストだけの道具は `tests/common/` へ移す。
  - ✅ 差分が小さく、1 つ直すごとに確かめて戻せる。
  - ❌ 統合テストが使う「テストのための口」を `pub` のまま残すか、どう隠すかを決める必要がある。
- **B：テスト用の機能（`test-support` の feature）を作り、テストだけの部品をその後ろへ置く**：dev-dependencies で自分自身をその機能つきで読む。
  - ✅ 移さずに出荷のビルドから外せる。
  - ❌ 新しい仕組み（機能・自分への依存）が増える。ponytail の「層を足さない」に反する。部品は移せるので要らない見込み。
- **C：A と B の混ぜ**：移せるもの（リーク検査・判定の写し・テストのパーサの写し）は移し、移せない観測の口（スレッド ID など）だけを残す。
  - 設計で、N6 の観測の値をテストが本当に要るか（`multi_instance_test.rs` はスレッドの数を ToolHelp で数えている）を見て決める。

### 選択肢 D：`DllMain` の道（要件 1.2・1.3）

- **D1：アクタースレッドが pasta.dll の参照を 1 つ持ち、終わるときに手放す（推奨の候補）**：起動時に `GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS)` で参照を足し、スレッドの最後に `FreeLibraryAndExitThread` で外して終わる。
  - ✅ スレッドが生きている間は pasta.dll が外れないので、外れた後に pasta.dll のコードが走ることが無い。`unload` を呼ばないホストの `FreeLibrary` は参照を 1 つ減らすだけで、すぐ戻る（止めない）。
  - ❌ `unload` を呼ばずに外されると、`DLL_PROCESS_DETACH` が来ず、アクターと VM はプロセスの終わりまで（または同じ DLL が読み直されて `load` が来るまで）残る。DLL のファイルもその間は開いたまま。`SHIORI.unload` と保存は走らない（Q1・Q9）。
  - ❌ **Research Needed**：スレッドが閉包から戻らずに終わると、`std::thread::JoinHandle::join` の結果の受け渡しが成り立たない（`unload` の道の join）。join の代わりにスレッドのハンドルを待つ、閉包の最後の段だけ別にする、などを設計で調べる。版 3 のインスタンス（Q2）と Linux・macOS（何もしない）の扱いも。
- **D2：`DllMain` の道では、アクターに VM を片付けさせずに ack させる**：Stop に「片付けない」の印を付け、`drop(shiori)` を `mem::forget` にする。
  - ✅ C モジュールの解放でローダーのロックを待たないので、5 秒止まらない。
  - ❌ ack の後のスレッドの終わり（pasta.dll のコード）と、外れる時の競りは残る。要件 1.2 を満たさない。
- **D3：初めの `load` で pasta.dll をピン留めする（`GET_MODULE_HANDLE_EX_FLAG_PIN`）**：プロセスの終わりまで外れない。
  - ✅ 1 行で落ちなくなる。
  - ❌ ゴーストを切り替えても pasta.dll はずっと残り、DLL の更新に SSP の再起動が要る。`unload` を正しく呼ぶ SSP の振る舞いまで変わる（要件 1.8 に反する）。

### 選択肢 T：テストだけの部品の行き先（要件 6）

- **T1：統合テストの共通の部品へ移す**（`ReloadProbe` 一式・テストのパーサの写しの代わりのヘルパ）。`teardown_via_sender` と `mailbox` は公開のまま使える。
- **T2：消して、テストを本番の道へ寄せる**（`marshaling::marshal_request`・`lifecycle::kick_sink`・`ActorMsg` の作り手・`spawn_actor_thread`）。
- **T3：`#[cfg(test)]` の単体テストへ移す**（クレートの中の非公開の部品を試すもの）。

## 規模とリスク

- **規模：M（3〜7 日）**。撤去と 1〜2 行の直しは S だが、`DllMain` の道（D1）は新しい形で、Rust の join との両立を調べる必要がある。タスクは 9〜11 の見込み（brief の 8〜11 より 1 つ多いのは、N1 の kick のテストの寄せ替え）。
- **リスク：Medium**。`DllMain` の道はローダーのロックとスレッドの寿命が絡み、単体テストで再現しにくい（`FreeLibrary` を呼ぶ統合テストか、SSP の実機）。ほかは特性化テスト（`byte_invariant_test.rs` ほか）が守る。

## 設計への申し送り

- 推奨：全体は A（T1・T2 を主に）、`DllMain` の道は D1。
- 決めること：
  - D1 と `JoinHandle::join` の両立（Research Needed）。版 3 のインスタンスへ広げるか（Q2）。
  - 解けない要求の警告の中身（Q5）と、版 3 の弾いたログの段（Q4）。
  - `hinst` を `PastaShiori::load` の引数に残すか（Q8。テストが Lua へ値を渡すため）。
  - N6 の観測の値の残し方。
  - 判定を 1 つにするとき、旧来の入口の SHIORI/2.x の扱い（Lua へ届ける）を保つ形。
- テストの順：挙動を変える直し（1・2・3・5）は、直す前の振る舞いを固めるテストを先に置く。`FreeLibrary` の道は、`LoadLibrary` で pasta.dll を読んで `load` の後に `unload` を呼ばずに `FreeLibrary` するテスト（C モジュールを読む辞書つき）を足せるかを調べる（Research Needed）。
- マニュアル：`book/src/internals/shiori.md` の上の表の行。`ponytail-lua-scripts` の節（403 行より後の Lua 側）は触らない。

## 要件ディスカッションへの持ち越し

> 2026-11-02 の要件のディスカッションで、Q1〜Q9 はすべて下の仮定のとおり確定した（requirements.md の末尾の「要件のディスカッションで決めたこと」）。

1. **Q1：`unload` を呼ばずに外されたとき、`SHIORI.unload` と保存を約束するか**
   - 仮定（要件 1.4）：約束しない。ホストを落とさないこと・止めないことを優先する。ホストの契約（`unload` を呼んでから外す）に反する道であり、`DllMain` の中では VM を安全に片付けられない。
2. **Q2：要件 1.2・1.3 を、版 3 のインスタンスにも及ぼすか**
   - 仮定（要件 1.5）：及ぼす。D1 はアクタースレッドごとの仕組みなので、入口によらず効く。
3. **Q3：`Key:value`（コロンの後に空白が無い）も受け付けるか**
   - 仮定（要件 3.3）：受け付ける。brief の文法の直し（`":" ~ " "?`）の帰結で、値は `value`。SHIORI の仕様は `: ` だが、受け付けて困る例は無い。
4. **Q4：版 3 の入口の、解けない要求のログを warn に上げるか**
   - 仮定（要件 3.7）：上げる（解けない要求だけ。SHIORI/2.x と、`notify` に来た GET は今の debug のまま）。brief の「解けない要求は、警告のログを残す」を両方の入口に当てる。
5. **Q5：解けない要求の警告に何を載せるか**
   - 仮定（要件 3.8）：解けなかった位置（行と列）と、要求の 1 行目。全文は載せない（OnNotifyUserInfo などは利用者の名前を含む。全文は今も 204 以外の応答のときだけ debug に出す）。
6. **Q6：アクターのテストだけの部品（N1・N2・N6）を、どこまで出荷のコードから外すか**
   - 仮定（要件 6.2）：N1・N2 と、N6 のうち作り手と `spawn_actor_thread` は外す。観測の値（スレッド ID・DAP の待受）は、テストが要るなら設計で残し方を決める。
7. **Q7：`tests/`・`src/*_tests.rs` と、短い上限を差し込む 1 行の口を残すか**
   - 仮定（要件 6.10）：残す。`tests/` をパッケージに入れるのは Rust の慣習で、brief の `exclude` は `doc/` だけ。上限の口は本番の関数がそのまま使う 1 行の委ねで、テストの足場の層ではない。
8. **Q8：いつも 0 の `hinst` を消すか**
   - 仮定（要件 7.3）：アクターの道から消す。Lua の `SHIORI.load(hinst, load_dir)` には 0 を渡し続ける（作者に見える約束は変えない）。`PastaShiori::load` の引数に残すかは設計で決める。
9. **Q9：`unload` を呼ばずに外されたとき、pasta.dll がメモリに残ってよいか**
   - 仮定（要件 1.2）：よい。D1 では、アクターと VM がプロセスの終わりまで（または同じ DLL が読み直されて次の `load` が来るまで）残り、DLL のファイルも開いたままになる。落ちるよりよい。`unload` を呼ぶ SSP では残らない（要件 1.8）。

---

# 設計フェーズの調査（2026-11-02）

## Summary

- **Feature**：`ponytail-shiori`
- **Discovery Scope**：Extension（既存の `crates/pasta_shiori` の中の直しと撤去。新しい外部の依存は無い）。進め方は light discovery（`design-discovery-light.md`）で、`DllMain` の道だけ std と Windows のローダーの振る舞いを深く調べた。
- **使ったスキル・規則**：`kiro-spec-design`（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）、`kiro-validate-design`（`design-review.md`）、ponytail（いちばん小さい完全な変更）。steering は `tech.md`・`structure.md` の `pasta_shiori` の行を読んだ。
- **Key Findings**：
  1. research の D1 の素の形（スレッドがいつも `FreeLibraryAndExitThread` で終わる）は、`std::thread::JoinHandle::join` と両立しない。閉包から戻らずに終わったスレッドを join すると、std が `expect("threads should not terminate unexpectedly")` で panic する（リリースは `panic = "abort"` なので、ホストごと落ちる）。
  2. 「参照を返すのは、join を終えた親か、見捨てられたスレッドのどちらか一方」と決める約束のチャネルを足すと、普段の道（unload → ack → join）は std のまま、見捨てた道だけがスレッド自身で参照を返して終われる。`Win32_System_Threading`（`WaitForSingleObject`）も要らない。
  3. D1 の下では、`FreeLibrary` による `DLL_PROCESS_DETACH` は、参照を持つアクタースレッドが 1 本も無いときにしか来ない。`DllMain` は何もしなくてよく、`JoinPolicy::Detach` と `teardown_actor_detached` は使い手を失う。

## Research Log

### std の join とスレッドの終わり

- **Context**：research の D1 の Research Needed（`FreeLibraryAndExitThread` と `JoinHandle::join` の両立）。
- **Sources Consulted**：ツールチェーン（rustc 1.99.1）の std のソース `library/std/src/thread/lifecycle.rs`・`join_handle.rs`・`os/windows/thread.rs`・`sys/thread_local/guard/windows.rs`。
- **Findings**：
  - スレッドの入口 `rust_start` は、閉包の結果を `their_packet.result` に書いてから `their_packet`（`Arc<Packet>`）を落とす。
  - `JoinInner::join` は OS のスレッドの終わりを待った後、`Arc::get_mut(&mut self.packet).expect("threads should not terminate unexpectedly")` で結果を取り出す。閉包から戻らずに `ExitThread` したスレッドでは `Arc` が 2 つのままなので、ここで panic する。
  - `JoinHandle::is_finished` も `Arc::strong_count == 1` を見るので、同じく真にならない。
  - `JoinHandle` は `AsRawHandle` を持つ。生のハンドルで `WaitForSingleObject` すれば終わりを待てるが、`windows-sys` の `Win32_System_Threading` が要る（要件 6.8 が通常の依存から外すと決めた機能）。
  - Windows のスレッドローカルの後始末は、今の std では FLS（`FlsAlloc` の後始末の関数）で行う。後始末の関数が DLL の中にあると、DLL が外れた後のスレッドの終わりで外れたコードを呼ぶので、std は DLL のときだけ `atexit` に `FlsFree` を登録し、DLL が外れるときに後始末を済ませる。コメントに「DLL を外してよいのは、ほかのスレッドが DLL の Rust のコードを走らせていないときだけ」とある。
- **Implications**：
  - スレッドを「いつも」`FreeLibraryAndExitThread` で終わらせる形は採れない（join が panic する）。生のハンドルを待つ形は、要件 6.8 の字面と食い違う。
  - 普段の道ではスレッドを閉包から普通に戻し、std の join を使う。参照を DLL の中から返すのは、もう誰も join しない（見捨てた）スレッドだけにする。その道で `ExitThread` が std の後始末を飛ばしても、見捨てた `JoinHandle` は既に落としてあり、残るのは `Packet` などの小さな `Arc` の漏れだけ。
  - D1 は、アクタースレッドが走っている間は DLL が外れないことを保証する。std の「外れるときに、ほかのスレッドが Rust のコードを走らせていない」の前提を、アクタースレッドについて満たす。

### Windows のローダーの振る舞い（D1 の前提）

- **Findings**：
  - `GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, 関数の番地)` は、その番地を含むモジュールの参照を 1 つ足す（`UNCHANGED_REFCOUNT` を付けない）。std 自身も `cleanup_is_unloadable` で同じ API を使っている。
  - `FreeLibraryAndExitThread` は、参照を返してから、DLL のコードへ戻らずにスレッドを終える。参照が 0 になれば、その中で `DLL_PROCESS_DETACH`（`lpReserved` が null）がそのスレッドの上で走り、DLL が外れる。
  - 参照が 0 にならない `FreeLibrary` は、`DllMain` を呼ばずにすぐ戻る。
  - 外れていない DLL を `LoadLibrary` し直すと、参照が増えるだけで `DLL_PROCESS_ATTACH` は走らない。static（`LEGACY`・`LOADU_INITIALIZED`・ログの登録簿・占有の札）はそのまま残る。
  - `FreeLibrary` は `Win32::Foundation`、`GetModuleHandleExW`・`FreeLibraryAndExitThread`・`GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS` は `Win32::System::LibraryLoader` にある（`windows-sys` 0.61）。`pasta_lua` が既に `Win32_System_LibraryLoader` を有効にしているので、バイナリは変わらない。

### unload を呼ばずに外された後の、次の load

- **Context**：議題 1 の「次の `load` まで残ってもよい」の中身。
- **Findings**（D1 の下）：
  - ホストの `FreeLibrary` は参照を 1 つ減らすだけで戻る。アクタースレッドは mailbox の `recv()` で待ったまま残り、VM・ログの登録・ディレクトリの占有も残る。
  - 同じパスの pasta.dll を読み直して `loadu` が来ると、`spawn_actor` の最初の `teardown_slot` が前のインスタンスに `Stop` を送る。前のアクターは普段どおり `SHIORI.unload` と保存をしてから ack し、join の後に参照を返す。新しいインスタンスは占有を普通に取れる（`ClaimMode::Exclusive`）。つまり、次の `load` のときに、遅れて前の後始末が走る。
  - `load`（ANSI）だけを呼ぶホストでは、前の回の `LOADU_INITIALIZED` が立ったまま残るので、`load` は何もせずに TRUE を返し、前のインスタンスが置き場に残る。`DLL_PROCESS_ATTACH` が走らないので、印を下ろす場所が無い。SSP は `loadu` を先に呼ぶので当たらない。
  - 版 3 のインスタンスは、ホストが手放したまま残る。同じディレクトリを版 3 で読み直すと、残ったインスタンスが占有を持っているので `SpawnError::DirInUse` になり、`load` は NULL を返す。生きた別のインスタンスと区別できないので、占有を奪えない。今（D1 の前）は、外れた DLL の下にスレッドが残り、何かのきっかけで落ちうる。

### FreeLibrary の統合テストを作れるか

- **Findings**：
  - `cargo test` は、ライブラリの単位を `cdylib` と `rlib` の両方で 1 度にビルドする。`target/debug/deps/pasta.dll` が、統合テストの実行ファイル（同じ `deps/`）の隣にできることを確かめた（このワークツリーの `deps/pasta.dll` と `libpasta.rlib` の時刻が同じ）。テストは `std::env::current_exe()` の親のフォルダから `pasta.dll` を `LoadLibraryW` できる。
  - テストの実行ファイルには rlib の pasta が静的に入っているが、`LoadLibraryW` した pasta.dll は別のモジュールで、static・LuaJIT・ログの購読者はすべて別になる。
  - 直す前の旧来の入口の道は、`DllMain` の中で 5 秒待った後に、外れた DLL の下でアクターが動いて落ちうる。テストのプロセスごと落ちるので、子のプロセス（同じテストの実行ファイルを、環境変数を付けて自分の `#[test]` 1 本だけで起こす）に閉じ込める。
  - Lua の C モジュールの代わりに、`[loader] ffi = true` のゴーストで `ffi.load` にシステムの DLL（例：`version`）を読ませる。VM の片付け（`DllPins` の Drop）がその DLL を `FreeLibrary` するので、ローダーのロックを待つ道は同じになる。`cl.exe` で DLL を作る `pasta_lua` の `dll_fixture.rs` の形は要らない。
  - 観測できる値：`FreeLibrary` にかかった時間、`GetModuleHandleW(パス)` が null か（外れたか）、子のプロセスの終わり方。直す前は「版 3 では外れる・旧来の入口と C モジュールでは 5 秒近くかかる」、直した後は「すぐ戻り・外れない」。`unload` の後の `FreeLibrary` では、直す前も後も外れる（参照の釣り合いの確かめ）。

### 判定を 1 つにする・警告の位置

- **Findings**：
  - 文法では `header = method ~ " " ~ (header3 | header2)` なので、平らにした組の並びは `get`／`notify` が先、`header2`／`header3` が後に来る。1 回なめれば、メソッドと版の両方が分かる。旧来の入口の 2.x は、今 `determine_method` が GET／NOTIFY を返すので、`Shiori2(メソッド)` として同じ道へ送れば変わらない。
  - 解けたのにメソッドが無いことは文法上起きない。起きても panic せず、読めない要求（`pest::error::ErrorVariant::CustomError`）として扱えばよい。
  - `pest::error::Error` は公開の欄 `line_col: LineColLocation`（`Pos((行, 列))` か `Span(始め, 終わり)`）を持つ。始めの行と列を使う。
  - `pest::error::Error<Rule>` は大きいので、`Result` の誤りに直に置くと clippy の `result_large_err` に当たる。`MyError::ParseRequest` と同じく `Box` に入れる。
  - 旧来の入口の判定は FFI のスレッドの上（`ActorInstance::request`）で走る。`ActorInstance` は `load_dir` を持つので、その場で `LoadDirGuard` を張れば、ゴーストのログのファイルへ出る（`v3.rs` の `reject` と同じ形）。フォルダが無くてロガーが登録されていないときは捨てられる（それでよい）。
  - SSP の記録の 720 行の OnNotifyUserInfo は、`Reference0`・`Reference1` に利用者の名前を持つ。テストで写すときは名前を仮の値に替える。

### テストだけの部品の行き先と、観測の値

- **Findings**（使い手を数えた）：
  - `ReloadProbe` 一式は `tests/actor_reload_leak_test.rs` だけが使う。そのファイルへ移し、`Win32_System_Threading` を dev-dependencies へ移す。
  - `teardown::teardown_actor`（いつも detach）は `tests/actor_teardown_test.rs` だけ。`teardown_via_sender` と `ActorThread::detach` の 2 行で置き換わる。`TeardownReport::is_clean` は 6 つのテストが使うが、`acked && anomaly.is_none()` の 1 式で置き換わる。
  - `marshaling::marshal_request`（写し）は `tests/actor_marshaling_test.rs` だけ。`marshal_get`／`marshal_notify` を直に使う。
  - `ActorMsg::get`・`notify`・`kick` の作り手は 5 つのテストが使う。variant を直に作るか、`marshal_get`・`marshal_notify` を使う。
  - `spawn_actor_thread`（`expect` の包み）は 9 つのテストが使う。`spawn_actor_thread_with(…, None, None).expect(…)` をテストが書く。
  - `actor_thread_id` は `actor_thread_vm_test`・`debug_preservation_test` が「テストのスレッドと違う」ことを確かめるだけ。VM（`!Send`）を閉包の中で作る形が型で保証しているので、要らない。`multi_instance_test` は ToolHelp で数えており、この値を使わない。
  - `debug_local_addr` は `debug_preservation_test` だけが使う。`multi_instance_test` の `listening_port` は、ゴーストのログの `debug backend listening addr=…` の行からポートを読んでいる。同じ読み方にすれば、ready のチャネルはロードの成否（`bool`）だけを運べばよい。
  - kick：本番の kick は DAP → `KickSink`（`kick_sink_for` の弱い送信端）→ `ActorMsg::Kick` → アクター。DAP からインスタンスの送り先までは `multi_instance_test` の `dap_kick_reaches_only_its_own_instance` が確かめている。シーンの kick の e2e 4 本は、`kick_sink_for` を `pub` にして、自分の mailbox の送り先をそのまま使えば、本番の送り先とアクターの道を通る。新しいテストの窓口は要らない。
  - `PastaShiori.hinst` を読むのは単体テストだけ。`tests/shiori_lifecycle_test.rs` は `PastaShiori::load(42, …)`・`load(999, …)` が Lua へ届くことを確かめているが、本番は必ず 0 を渡す。

## Architecture Pattern Evaluation（DllMain の道）

| 案 | 中身 | よいところ | 弱いところ |
| -- | ---- | ---------- | ---------- |
| D1-a | スレッドがいつも `FreeLibraryAndExitThread` で終わる | 形が単純 | std の join が panic する（`expect`）。採れない |
| D1-b | D1-a に加え、`unload` の道は join の代わりに生のハンドルを `WaitForSingleObject` で待つ | join を使わないので panic しない | `Win32_System_Threading` が通常の依存に要る（6.8 の字面と食い違う）。普段の道でも毎回 std の後始末を飛ばし、`Packet` を漏らす。非 Windows と道が分かれる |
| D1-c（採る） | 親がモジュールの参照を 1 つ取ってスレッドに持たせ、スレッドは ack の後に「join の約束」を待つ。約束が来れば普通に戻り、親が join の後に参照を返す。約束の送り手が落ちていれば（見捨てられた）、スレッドが `FreeLibraryAndExitThread` で参照を返して終わる | 普段の道は今の std の join のまま。見捨てた道だけが特別。`Threading` が要らない。OS で道が分かれない（非 Windows は参照が無く、戻るだけ） | 約束のチャネルが 1 つ増える。見捨てた道で小さな漏れが残る |
| D2 | `DllMain` の道で VM を片付けずに ack させる | 5 秒止まらない | ack の後のスレッドの終わりと外れる時の競りが残る（1.2 を満たさない） |
| D3 | 初めの `load` でピン留め | 1 行 | `unload` を呼ぶ SSP でも外れない（1.8 に反する）。議題 1 で採らないと確定 |

## Design Decisions

### Decision：DllMain の道は D1-c（約束のチャネルつきのモジュールの参照）

- **Context**：要件 1.2〜1.5、議題 1。
- **Alternatives Considered**：上の表の D1-a・D1-b・D2・D3。
- **Selected Approach**：`spawn_actor_thread_with` が、スレッドを起こす前に `GetModuleHandleExW(FROM_ADDRESS)` で参照を 1 つ取る。`ActorThread` は約束の送り手を持ち、スレッドは受け手を持つ。`ActorThread::join` は約束を送ってから join し、参照を `FreeLibrary` で返す。`ActorThread::detach`（と、join も detach もせずに落とすこと）は送り手を落とすだけで、参照はスレッドが返す。スレッドは片付けと ack を終え、持っている値をすべて落としてから約束を待つ。
- **Rationale**：参照を返すのはいつも 1 人に決まり、2 度返すことも、生きたスレッドの下で外れることも無い。普段の道の振る舞い（ack の後に join・ログ・`SHIORI.unload`・保存）は変わらない。
- **Trade-offs**：見捨てた道で、std の `Packet` と `Thread` の `Arc`（合わせて 200 バイトほど）が漏れる。見捨てたスレッドが永遠に戻らなければ、DLL はプロセスの終わりまで外れない（安全な側に倒れる）。
- **Follow-up**：子のプロセスの `FreeLibrary` テストで、直す前と後の振る舞いを確かめる。見捨てたスレッドが終わることを ToolHelp で確かめる。

### Decision：DllMain は何もしない・JoinPolicy を消す

- **Context**：D1-c の下では、`FreeLibrary` による `DLL_PROCESS_DETACH` は、参照を持つ `ActorThread` が 1 つも無いとき（＝置き場が空で、版 3 のインスタンスも無いとき）にしか来ない。
- **Selected Approach**：`DllMain` はどの通知でも 1 を返す。`teardown_actor_detached`・`JoinPolicy`（`Detach` だけが join しない道だった）を消し、`ActorInstance::shutdown` は「ack が届けば join、届かなければ手放す」だけにする。`LOADU_INITIALIZED` は DLL が外れれば static ごと消えるので、`DllMain` で下ろさない。
- **Rationale**：ローダーのロックの中で Rust の後始末（`arc_swap` のスレッドローカル・ログ）を走らせる道が無くなる。使い手の無い分岐を残さない。
- **あわせて**：`already_done`（受け口が既に閉じている）のときも join する。受け口を閉じたスレッドは、片付けを終えて約束を待つだけか、panic で閉包を抜けたかのどちらかなので、join は止まらない。見捨てる道（スレッドが参照を返して終わる道）を、ack が来ないときだけに絞る。

### Decision：判定は `classify_request` 1 つ・誤りは位置つきで返す

- **Selected Approach**：`classify_request(&str) -> Result<RequestKind, Box<ParseError>>`、`RequestKind = Get | Notify | Shiori2(ShioriMethod)`。旧来の入口は `Shiori2(m)` を `m` として扱う。解けなければ、両方の入口が `ActorInstance::warn_unreadable` で warn を 1 行出す。`determine_method` は消す。

### Decision：テストだけの部品は、使い手のテストへ移すか、本番の道へ寄せる

- **Selected Approach**：research の T1（`ReloadProbe` 一式を `tests/actor_reload_leak_test.rs` へ）と T2（`marshal_request` の写し・`kick_sink`・`kick_into`・`ActorInstance::kick`・`ActorMsg` の作り手・`spawn_actor_thread`・観測の getter を消す）。`#[cfg(test)]` へ移すもの（T3）は無い。kick の e2e は `kick_sink_for` を `pub` にして本番の送り先を使う。

### Decision：hinst は PastaShiori::load からも消す

- **Selected Approach**：`PastaShiori::load(load_dir)`。`call_lua_load` は Lua の `SHIORI.load` へ定数の 0 を渡す。`hinst` を値で確かめていたテストは 0 を確かめる。
- **Rationale**：本番は必ず 0。値を通せる口を残すと、テストが本番に無い値を確かめ続ける。

### Synthesis（一般化・採るか作るか・簡単にする）

- **一般化**：旧来の入口と版 3 の入口の違いは「2.x を Lua へ届けるか 400 か」だけなので、判定を 1 つにして、違いを呼び出し側の 1 行に置く。解けない要求の警告も、両方の入口が `ActorInstance` の 1 つの関数を呼ぶ。
- **採るか作るか**：スレッドの終わりとモジュールの寿命は、Windows の `GetModuleHandleExW`・`FreeLibraryAndExitThread`（この用途のための API）を採る。待ちの仕組みは既存の flume のチャネルを採り、生のハンドルの待ちは作らない。行と列は pest の `line_col` を採る。
- **簡単にする**：`JoinPolicy`・`teardown_actor_detached`・`DllMain` の分岐・`Encoder`／`CodePage`・`Reply`・`Shiori` トレイト・`hinst`・観測の getter を消す。新しく足すのは `ModuleRef`（約 40 行）と約束のチャネル（数行）だけ。

## Risks & Mitigations

- **D1-c は、std の DLL の後始末の作り（FLS と `atexit`）とローダーの振る舞いに頼る** — 子のプロセスの `FreeLibrary` テストで直す前と後を固める。SSP の実機（`ssp-ghost-test-launch`）でも任意で確かめる。
- **`pasta_lua` の debug のスレッド（`[debug] enabled` のときだけ）は、アクターの参照では守られない** — VM の片付けで join される前提。この spec の外（`pasta_lua`）。設計の残りの危険として書く。
- **版 3 で `unload` を呼ばずに外したホストが、同じディレクトリを読み直すと NULL になる** — 開発者の判断（設計のディスカッション）。落ちるよりよい。
- **`load`（ANSI）だけを呼ぶホストで、前の回の `LOADU_INITIALIZED` が残る** — SSP は `loadu` を呼ぶので当たらない。マニュアルに書く。
- **テストのファイルの書き換えが多い（約 25 本）** — 機械的な撤去は 1 つずつのコミットにし、各コミットで `cargo test -p pasta_shiori` を通す。`shiori4-v4-host` とは同じウェーブに置かない（要件の Adjacent）。

## References

- rustc 1.99.1 の std のソース：`library/std/src/thread/lifecycle.rs`（`JoinInner::join`）、`library/std/src/sys/thread_local/guard/windows.rs`（FLS と `atexit` の `FlsFree`）、`library/std/src/os/windows/thread.rs`（`JoinHandle` の `AsRawHandle`）。
- Microsoft の文書：`GetModuleHandleExW`（`GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS`）、`FreeLibraryAndExitThread`、`DllMain` のベストプラクティス（ローダーのロックの中で待たない）。
- `crates/pasta_lua/src/runtime/ffi_load/win32.rs`（`DllPins` の Drop が `FreeLibrary` する・`load_system_dll`）。

## 設計のディスカッションの決定（2026-11-02）

開発者が、設計の議題 1〜6 を推奨案のまま一括で承認した（design.md の Open Questions の 1〜8）。

- **D1-c を採る・`DllMain` は何もしない**：普段の `unload` の道を std の `join` のまま保ち、特別な道（`FreeLibraryAndExitThread`）を見捨てたスレッドだけに閉じ込められるため。素の D1 は、閉包から戻らないスレッドの `join` が panic する（std の `JoinInner::join`）ので採れない。D1-b は普段の道でも std の後始末を飛ばし、要件 6.8 と食い違う。
- **読み直しの残りの危険を受け入れる**：版 3 は手放されたインスタンスと生きたものを見分けられない。今は同じ場面で落ちうるので、NULL のほうがよい。SSP は `unload` を呼び、`loadu` で読み直すので当たらない。マニュアルに書く。
- **`hinst` は `PastaShiori::load` の引数からも消す**：テストの 42・999 は本番に無い値で、素通しの段を残さないため。
- **kick の e2e は `kick_sink_for` の送り先で通す**：本番の VM と同じ送り先で、DAP からインスタンスまでは `multi_instance_test` が確かめ済みのため。
- **観測の getter を消す**：スレッドが違うことは VM の型が保証し、DAP のポートはログから読めるため。
- **`free_library_test` を要件 1 の最初の関門にする**：D1-c は std の内部とローダーの振る舞いに頼り、単体テストでは再現できないため。
