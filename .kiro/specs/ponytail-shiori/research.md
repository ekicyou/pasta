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
