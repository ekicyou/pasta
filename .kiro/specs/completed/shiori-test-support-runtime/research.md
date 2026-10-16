# ギャップ分析: shiori-test-support-runtime

調べた日: 2026-10-16（ブランチ `claude/shiori-test-support-runtime-19efc6`、main 3071114a を取り込んだ後）

## 1. まとめ

- 写しを消しても、ふるまいを確かめるテストは 1 本も落ちない。実際に写しのコピーを外して `cargo test -p pasta_shiori` を走らせたところ、落ちたのは「`scriptlibs` が置かれること」を確かめる 1 本（`test_path_helpers_test.rs` の `deep_non_ansi_fixture_is_buildable_and_readable`）だけだった。写しにしか無い振る舞いに頼るテストは無い。
- 回避している 4 つのフィクスチャから `lua_search_paths` を外しても全部通る。既定の検索パスには `scripts` が入るが、写しが無ければ `scripts/` は空なので、本物のランタイムがそのまま読まれる。
- ゴーストの直下の `pasta_scripts/` へ本物をコピーする処理は、どれも効いていない。今のローダーの検索パスに `pasta_scripts`（直下）は無く、本物は `pasta.dll` が `profile/pasta/pasta_scripts/` へ自己展開したものが読まれる。この処理は brief が挙げた 3 か所ではなく、`tests/` の下に 10 ファイル、`src/` の中に 2 か所ある。全部のコピー元を存在しないパスに変えても、テストは全部通った。
- 直すのはテストのファイルだけで、規模は小さい（S）。危険も低い（Low）。決めることは、消す範囲（`scriptlibs` の写し・`src/` の中のコピー）と、フィクスチャの残りの行の扱いくらい。

## 2. 今の状態

### 2.1 写し（`crates/pasta_shiori/tests/support/`）

| 場所 | ファイル | 本物との違い |
| ---- | -------- | ------------ |
| `scripts/pasta/` | `act.lua`(209 行)・`actor.lua`・`global.lua`・`init.lua`・`scene.lua`・`store.lua`・`word.lua`・`areka/init.lua`・`shiori/init.lua`・`shiori/entry.lua`（計 10） | 9 ファイルは本物と違う（本物の `act.lua` は 726 行）。`shiori/entry.lua` は 204 を返すだけのテスト用の最小版で、本物には無い形 |
| `scriptlibs/lua_test/` | `expect.lua`・`test.lua`・`toDebugString.lua` | `test.lua`・`toDebugString.lua` が本物（`crates/pasta_lua/scriptlibs/lua_test/`）と違う。本物には `mocks.lua` もあるが写しには無い |

- 写しを手で直した跡がある（例: b7379789 の scene-identity-format で `scene.lua` の 2 行）。写しを持つ限り、本物を変えるたびに直すかどうかの判断が要る。
- 写しができた経緯は `TEST_COVERAGE.md`（4 節の「修正内容」）にある。当時は標準ランタイムがゴーストに同梱される形で、`shiori_lifecycle` のテストに完全なモジュールが要った。今は `pasta.dll` が自己展開するので、この理由は無くなっている。

### 2.2 共通部品（`tests/common/mod.rs`、172 行）

- `copy_fixture_into`（58〜72 行）が、先に `copy_support_dirs`（156〜172 行）で `tests/support/scripts`・`scriptlibs` を置き、後からフィクスチャを重ねる。`copy_fixture_to_temp`（49 行）はその宛先が一時ディレクトリの版。
- 説明の古いところ:
  - 38〜39 行「support files (scripts and scriptlibs) needed for SHIORI operation」— 本物は自己展開されるので、要らない。
  - 42 行・67 行「fixture-specific main.lua」— 実際に差し替えているのは `entry.lua` だけ。

### 2.3 共通部品を使うテストとフィクスチャ

| フィクスチャ | 自分の `scripts/` | 回避 | 使うテスト（共通部品経由） |
| ------------ | ----------------- | ---- | -------------------------- |
| `shiori_lifecycle` | `pasta/shiori/entry.lua` | なし（写しを読む） | `actor_log_context_test`・`ffi_loadu_test`・`ffi_logging_test`・`load_failure_visibility_test`・`path_robustness_e2e_test`・`shiori_lifecycle_test`・`shiori_test_env_test`・`test_path_helpers_test`（8 ファイル） |
| `codegen_runtime_safety` | なし | あり | `codegen_runtime_safety_e2e_test` |
| `actor_proxy_act_delegation` | なし | あり | `actor_proxy_act_delegation_e2e_test` |
| `call_execution_correctness` | なし | あり | `call_execution_correctness_e2e_test` |
| `scene_alias_ontalk` | なし | あり | `scene_alias_ontalk_e2e_test` |

- `shiori_lifecycle` は自分の `entry.lua` で写しの `entry.lua` を上書きしている。ほかの写し（`pasta/*.lua`）は、辞書から作った Lua が `require` するときに読まれる。ただし、このフィクスチャの入口は `@pasta_search` でシーンを探して、見つかれば固定の文を返すだけなので、ランタイムの違いがテストの結果に出ない。
- 回避している 4 つの `pasta.toml` は、`lua_search_paths` を「既定から `scripts` を除いたもの」にしている。同じ `[loader]` に `transpiled_output_dir = "profile/pasta/cache/lua"`（既定と同じ値）と `pasta_patterns = ["dic/*.pasta"]`（既定は `dic/**/*.pasta`。どのフィクスチャも `dic/` の直下だけなので結果は同じ）もある。

### 2.4 別のコピー処理（ゴーストの直下の `pasta_scripts/` へ本物をコピーする）

| ファイル | コピー元の行 | 使うフィクスチャ |
| -------- | ------------ | ---------------- |
| `tests/common/async_callback_support.rs` | 33 | `async_callback` |
| `tests/actor_thread_vm_test.rs` | 47 | `async_callback` |
| `tests/actor_marshaling_test.rs` | 74 | `async_callback` |
| `tests/actor_reload_leak_test.rs` | 37 | `async_callback` |
| `tests/actor_teardown_test.rs` | 44 | `async_callback` |
| `tests/byte_invariant_test.rs` | 137 | `async_callback` |
| `tests/ffi_actor_lifecycle_test.rs` | 53 | `async_callback` |
| `tests/ffi_extern_session_e2e_test.rs` | 210 | `async_callback` |
| `tests/debug_preservation_test.rs` | 66 | `debug_preservation` |
| `tests/actor_kick_test.rs` | 62 | なし（`pasta.toml` と `entry.lua` をその場で書く） |
| `src/shiori_request_tests.rs` | 43・471 | `pasta_lua/tests/fixtures/loader/minimal` ほか |

- ローダーの検索パス（`crates/pasta_lua/src/loader/config/mod.rs` の `default_lua_search_paths`）は `profile/pasta/save/lua`・`scripts`・`profile/pasta/pasta_scripts`・`profile/pasta/cache/lua`・`scriptlibs`。ゴーストの直下の `pasta_scripts` は入っていない。`.lua` の探索も `.pasta` のパターンから作る（`dic/` の下だけ）ので、直下の `pasta_scripts/` は読まれない。
- 本物は、読み込みの Phase 2.5（`crates/pasta_lua/src/loader/mod.rs` の `sync_pasta_scripts`）で `profile/pasta/pasta_scripts/` へ自己展開される。テストが本物で動いているのは、このコピーのおかげではなく、自己展開のおかげ。
- 説明の古いところ:
  - `async_callback_support.rs` 18〜20 行「`copy_fixture_to_temp` でサポート＋フィクスチャをコピーした後」— 実際には呼んでいない。
  - 各ファイルの「scripts/ は pasta_scripts/ より先に検索される」は、順序の説明としては正しいが、指している `pasta_scripts/` が直下のもので、読み込み先と違う。
- 各テストファイルに、同じ形のディレクトリのコピー関数（`copy_dir_recursive`）が、`tests/` に 15 個（共通部品の 2 個を含む）、`src/` に 1 個ある。テストのバイナリごとに `mod common` を取り込むかどうかが違うので、こうなっている。

### 2.5 共通部品を使わないもの（触らない）

- `scene_kick*_e2e_test.rs`（4 ファイル）と `shiori_sample_ghost_test.rs` などは、サンプルゴースト `hello-pasta` を丸ごとコピーして使う。写しも直下の `pasta_scripts/` も通らない。

### 2.6 ほかの文書から写しを指しているところ

- `.kiro/specs/scene-attribute-store/brief.md` 176 行: 「`pasta_shiori` の結合テストは古いランタイムの写しを読む。写しは直さない（`shiori-test-support-runtime` の持ち場）」。この spec が入ると古くなる。完了のときの申し送りで直す。
- `TEST_COVERAGE.md` 260 行・`.kiro/specs/review-improvement-loop/tasks.md` 849 行・完了した spec の文書: 過去の記録なので書き換えない。

## 3. 試したこと（2026-10-16）

環境: `NoDefaultCurrentDirectoryInExePath`・`PASTA_DEBUG`・`PASTA_DEBUG_PORT`・`PASTA_LOG` を外して `cargo test -p pasta_shiori --no-fail-fast`。

| 回 | 変えたこと | 結果 |
| -- | ---------- | ---- |
| 基準 | なし | 313 通過・0 失敗（37 バイナリ） |
| 実験 | (1) `copy_fixture_into` から `copy_support_dirs` の呼び出しを外す（写しを置かない）／(2) 4 つのフィクスチャから `lua_search_paths` の行を消す（既定の検索パスに戻す）／(3) 2.4 の表の 12 か所のコピー元を存在しないパスに変える（直下の `pasta_scripts/` を作らない） | 312 通過・1 失敗。落ちたのは `test_path_helpers_test::deep_non_ansi_fixture_is_buildable_and_readable` の「scriptlibs が設置されていない」（50 行）だけ |

- 実験の変更は全部 `git checkout --` で戻した。`pasta_lua` のテストは走らせていないので、`sample.generated.lua` は変わっていない。
- 分かったこと:
  - 写しにしか無い振る舞いに頼るテストは無い。古いランタイムの結果を期待値にしているテストも見つからない（要件 4.2 は、今のところ当てはまるものが無い）。
  - 直すのは `test_path_helpers_test.rs` の 1 か所（`scriptlibs` が置かれることの確かめ）。`scripts` の確かめは、`shiori_lifecycle` が自分の `scripts/pasta/shiori/entry.lua` を持つので、写しが無くても通る。
  - 直下の `pasta_scripts/` へのコピーは、`src/` の中を含めて、どれも効いていない。

## 4. 要件と今あるものの対応

| 要件 | 今あるもの | ギャップ |
| ---- | ---------- | -------- |
| 1.1・1.2 写しを置かない／本物だけで動く | `copy_support_dirs` が写しを置く | 足りない: コピーを外す。本物は自己展開で既に読まれている |
| 1.3 写しを持たない | `tests/support/scripts/`（10 ファイル） | 足りない: 消す |
| 1.4 `lua_test` の写しを持たない | `tests/support/scriptlibs/lua_test/`（3 ファイル。使うテストは無い） | 足りない: 消す（確定 1） |
| 1.5・2.3・5.2 同じ確かめで通る | 実験で確認済み（1 本を除く） | なし |
| 2.1・2.2 回避の設定とコメントを外す | 4 つの `pasta.toml` | 足りない: `lua_search_paths` とコメントを消す。残りの行は残す（確定 3） |
| 2.4 新しいフィクスチャは何も書かずに本物で動く | 写しを外せばそうなる | なし（要件 6.1 の説明で伝える） |
| 2.5 回避と関係ない設定を保つ | — | 制約 |
| 3.1・3.2 差し替えは最小限・本番の順で | `shiori_lifecycle`・`async_callback`・`debug_preservation` が `entry.lua` だけを持つ | なし |
| 3.3・3.4 写しにしか無い振る舞い・写しを残す理由 | 実験で見つからない | なし（当てはまるものが出たときの決まり） |
| 4.1 写しの有無を確かめるテストを直す | `test_path_helpers_test.rs` 46〜53 行 | 足りない: `scriptlibs` の確かめを、本来の意図（置けて読めること）に合わせる |
| 4.2・4.3 古い結果の期待値を直す・記録 | 見つからない | なし |
| 5.1 直下へのコピーをやめる（`tests/`） | 10 ファイル | 足りない: コピーの手順を消す |
| 5.3 `src/` の中のコピー | `src/shiori_request_tests.rs` 2 か所 | 足りない: 外す（確定 2） |
| 6.1〜6.4 説明を実際に合わせる | `tests/common/mod.rs` 38〜42・62〜67 行、`async_callback_support.rs` 18〜20・31・43 行、各テストの準備のコメント | 足りない: 書き直す |
| 7.1〜7.3 外部の挙動を変えない | テストのファイルだけを触る | 制約。ワークスペースのテストと clippy で確かめる |

調べ直しが要るもの（Research Needed）: なし。必要なことは実験で確かめた。

## 5. やり方の案

### 案 A: 消すだけ（今の部品をそのまま使う）

- `tests/support/` を丸ごと消す。`copy_fixture_into` から写しのコピーを外し、使わなくなる `copy_support_dirs` を消す。
- 4 つの `pasta.toml` から `lua_search_paths` と回避のコメントを消す。
- 10 ファイルと `src/` の 2 か所（確定 2）から直下の `pasta_scripts/` へのコピーを消し、説明を直す。
- `test_path_helpers_test.rs` の `scriptlibs` の確かめを直す。
- 良い点: 差分が小さく、ほとんどが削除。新しいファイルは要らない。
- 気になる点: コピー関数の重複（`tests/` に 15 個）は残る（ふるまいの問題ではない）。

### 案 B: 消したうえで、準備の部品を一つにまとめる

- 案 A に加えて、`async_callback`・`debug_preservation` を使うテストの準備（9 ファイル）を `tests/common` の `copy_fixture_to_temp` に寄せ、重複したコピー関数を消す。
- 良い点: 準備の手順が一か所になり、次に何かあっても直す場所が一つで済む。
- 気になる点: 触るファイルが増え、テストのバイナリごとに `mod common` の取り込みを足す必要がある。この spec の目的（本物だけで動かす）には要らない。

### 案 C: 写しを本物から作る

- 写しを消さず、ビルドかテストの前に本物からコピーして作る。
- 良い点: 写しの置き場所と順序を変えずに済む。
- 気になる点: 本物は既に自己展開されるので、作る意味が無い。`scripts/` に本物の写しを置くと、本番の「利用者のスクリプトが先」という順序を、テストが標準ランタイムで塞いでしまう。実験で写しが要らないと分かったので、brief の「残す理由が見つかった場合」に当たらない。

## 6. 規模と危険

- 規模: **S**（1〜3 日）。削除が中心で、直す確かめは 1 か所。タスクにすると 5〜6 本（写しとコピーの撤去、4 つのフィクスチャ、直下へのコピーの撤去、確かめの直し、説明の直し、通しの確認）。brief の見積もり（6〜8）より少し小さい。
- 危険: **Low**。実験で全部のテストが通ることを確かめた。本番のコードは触らない。

## 7. 設計に持ち越すこと

- 推す案: **案 A**。目的に要るのは削除だけで、実験で裏付けがある。
- 設計で決めること:
  - `test_path_helpers_test.rs` の確かめの直し方（`scriptlibs` の確かめを消すか、ゴーストの読み込みに要るもの（`pasta.toml`・`dic/`）の確かめに置き換えるか）。意図は「深い・ANSI で表せないパスに一時ゴーストを置けて、読めること」。
  - 共通部品の説明の書き方（新しいフィクスチャは差し替えるファイルだけを持てばよい、と伝える）。
  - 4 つの `pasta.toml` の残りの行は残す（確定 3）。
  - 直下の `pasta_scripts/` へのコピーを消した後、そのファイルの `copy_dir_recursive` がフィクスチャのコピーにも使われていなければ、使われない関数として残る（警告が出る）。そのファイルでは関数ごと消す。どのファイルが当たるかを設計で洗う。
- 完了のときにすること: `.kiro/specs/scene-attribute-store/brief.md` 176 行の「写しは直さない」の注記を、写しが無くなったことに合わせて直す（申し送り）。

## 8. 決めたこと（2026-10-16 の要件のディスカッションで、全部推す答えのとおりに確定）

1. **`tests/support/scriptlibs/lua_test/` の写しも消すか**。推す答え: 消す。使うテストが無く（実験で、置かなくても `scriptlibs` の確かめ以外は全部通った）、本物と中身も違う。brief の「範囲外」は `pasta_lua` 側の本物の `lua_test` のことで、この写しは含まない。
2. **`src/shiori_request_tests.rs` の直下へのコピー（2 か所）も外すか**。推す答え: 外す。brief は「`src/` の中は本物を使うので触らない」としたが、その本物は直下の `pasta_scripts/` へのコピーからではなく自己展開から来ている（実験でコピー元を消しても全部通った）。根が同じ効いていない手順なので、別の spec に回さずここで片付ける。ほかの spec は `pasta_shiori` を触らないので、重なりも無い。
3. **回避している 4 つの `pasta.toml` の、`lua_search_paths` 以外の `[loader]` の行（`transpiled_output_dir`・`pasta_patterns`）をどうするか**。推す答え: 残す。`transpiled_output_dir` は既定と同じ値、`pasta_patterns` は結果が同じで、どちらも写しを避けるための設定ではない。外すと「回避の取り外し」を超えて、テストの前提を変えることになる。
4. **重複したコピー関数（`tests/` に 15 個）をまとめるか（案 B）**。推す答え: まとめない。ふるまいの不具合ではなく、この spec の目的（本物だけで動かす）にも要らない。直下へのコピーを消すと、それぞれのコピー関数はフィクスチャのコピーにだけ使われ、役目ははっきりする。
5. **写しがまた置かれないための見張り（テストや検査）を足すか**。推す答え: 足さない。写しを消して、共通部品の説明に「本物は自己展開されたものを使う。フィクスチャは差し替えるファイルだけを持つ」と書けば足りる。

## 9. 設計の記録（2026-10-16、kiro-spec-design）

- 調べ方: 既存のしくみを広げる変更として、軽い調べ（design-discovery-light）にした。外の依存は増えないので、外部の調べは要らない。使ったスキル・決まり: `kiro-spec-design`（design-principles・design-discovery-light・design-synthesis・design-review-gate）。
- 実験の扱い: 設計の段で、設計どおりの変更を手元で当てて clippy とテストを走らせようとしたが、操作が許可されず、ファイルは書き換えていない。根拠は 3 節の実験（同じ基点）。使われなくなる関数・`use` は読んで洗った。

### 9.1 まとめ方（design-synthesis）

- 一般化: 「標準ランタイムをテストが自分で置く」手順が 2 種類（`scripts/` への写し・直下の `pasta_scripts/` への本物のコピー）あるが、根は同じで、どちらも自己展開に任せれば要らない。新しい部品は作らず、両方を消す。
- 作るか使うか: 自己展開（`sync_pasta_scripts`）という既にある仕組みを使う。写しを本物から作る案（案 C）は取らない（5 節）。
- 削れるもの: 案 B（準備の部品をまとめる）・見張りのテストは足さない（8 節の確定 4・5）。

### 9.2 7 節の持ち越しの決め

- (a) `test_path_helpers_test.rs`: 50〜53 行（`scriptlibs` の確かめ）だけを消す。`scripts` の確かめ（46〜49 行）は、フィクスチャ `shiori_lifecycle` 自身の `scripts/pasta/shiori/entry.lua` で成り立つので残す。`pasta.toml` が置かれて読めることの確かめ（38〜45 行）と合わせて、「深いパス・ANSI で表せないパスの下に一時ゴーストを置けて読める」という意図を確かめられる。
- (b) `tests/common/mod.rs`: `copy_fixture_to_temp` の説明（36〜48 行）を、「コピーするのはフィクスチャ自身のファイルだけ。標準ランタイムは `pasta.dll` が `profile/pasta/pasta_scripts/` へ自己展開した本物を使う。フィクスチャには差し替えるファイルだけ置く（例: `scripts/pasta/shiori/entry.lua`）。`scripts/` は標準ランタイムより先に読まれる。`lua_search_paths` を書き換える必要はない」という文にする（文面は design.md）。62〜67 行の写しのコピーとコメント（`main.lua` の例を含む）と、154〜172 行の `copy_support_dirs` を消す。
- (c) `copy_dir_recursive` をファイルごとに洗った結果: 直下へのコピーを消すと呼び出しが無くなるのは `tests/actor_kick_test.rs` だけ（フィクスチャを書き出すだけで、コピーしない）。関数（31〜50 行）、`use` の `Path`（13 行）、`manifest_dir`（59 行）を消す。ほかの 9 ファイルと `src/shiori_request_tests.rs`、`tests/common/mod.rs` は、フィクスチャ（またはサンプルゴースト）のコピーに使い続けるので残す。`src/shiori_request_tests.rs` の `crate_root` も `scriptlibs` のコピーに使い続けるので残す。
- (d) 申し送り: `.kiro/specs/scene-attribute-store/brief.md` 176 行を、完了のときに直す（design.md の申し送り）。

### 9.3 新しく分かったこと

- `src/shiori_request_tests.rs` は、`crates/pasta_lua/scriptlibs/`（本物の `lua_test`）も一時ゴーストの `scriptlibs/` へコピーしている。写しではなく本物で、`scriptlibs` は検索パスに入っているので、効いていない手順ではない。範囲外として残す。
- 同じファイルの `copy_shiori_lifecycle_fixture` は、フィクスチャを丸ごとコピーした（464 行）後に、フィクスチャの `scripts/` をもう一度コピーしている（486〜491 行）。二度目は要らないが、動きに影響せず要件の範囲外なので残し、コメントの読み込み先だけ直す。
- 説明の直しが要るのは、`async_callback_support.rs`（18〜20・42〜43 行）、`actor_thread_vm_test.rs`（39 行）、`byte_invariant_test.rs`（130 行）、`ffi_extern_session_e2e_test.rs`（203 行）、`debug_preservation_test.rs`（59 行）、`actor_kick_test.rs`（52〜53・78 行）、`src/shiori_request_tests.rs`（38・466・486 行）。ほかのファイルの説明は直下の `pasta_scripts/` に触れていない。
- `pasta_shiori` の `Cargo.toml`・steering・スキルの文書は `tests/support` を指していない。今も写しを指しているのは、過去の記録（触らない）と `scene-attribute-store/brief.md` 176 行（申し送り）と `roadmap.md` 346 行（完了の手順で直す）だけ。

## 10. 設計のディスカッションで決めたこと（2026-10-16、全部推す答えのとおりに確定）

- **`test_path_helpers_test.rs`**: `scriptlibs` の確かめだけを消す。`pasta.toml` を読む確かめと `scripts` の確かめで、「深いパス・ANSI で表せないパスの下に置けて読める」という意図は保てる。差分がいちばん小さい。
- **共通部品の説明の言葉**: 日本語。すぐ下の `copy_fixture_into` と、その下の節の説明が日本語なので、隣り合う関数で言葉を混ぜない。
- **`src/shiori_request_tests.rs` の二度目のコピー**: 消す。直下の `pasta_scripts/` を置いた後に差し替えを重ねるための手順で、根が同じ。コメントを書き直して残すより差分が小さく、読む人が「なぜ二度コピーするのか」と迷わない。
