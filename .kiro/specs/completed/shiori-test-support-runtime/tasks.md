# Implementation Plan

行の番号は design.md と同じく基点 `0ecc524a` のもの。ずれたときはコードの形で場所を探す。

- [x] 1. 写しを消して、共通部品がフィクスチャだけを置くようにする
- [x] 1.1 写しを消し、共通部品の写しのコピーと説明を直す
  - `crates/pasta_shiori/tests/support/` の 13 ファイル（`scripts/pasta/` の 10、`scriptlibs/lua_test/` の 3）を消す。ディレクトリも残さない
  - 共通部品から、写しをコピーする呼び出しとその前後のコメント（「Copy support files first」など、`main.lua` の例を含む）と、使われなくなる写しのコピー関数を消す。フィクスチャのコピーと、ディレクトリを再帰コピーする関数は残す
  - `copy_fixture_to_temp` の説明を design.md の日本語の文に書き換える（自己展開を使う・差し替えだけ置く・例は `entry.lua`・検索パスを書き換えなくてよい）
  - 終わった状態: `crates/pasta_shiori/tests/support/` が無く、共通部品に写しへの参照が無い。`cargo test -p pasta_shiori --test shiori_lifecycle_test` など `shiori_lifecycle` を使うテストが、フィクスチャの `entry.lua` の差し替えだけで通る
  - `test_path_helpers_test` は 1.2 で直すまで `scriptlibs` の確かめで落ちる。`pasta_shiori` のテストを通しで確かめるのは 1.2 の後
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.4, 3.1, 3.2, 6.1, 6.2, 6.4_
- [x] 1.2 パスの自己検証テストを、写しの有無ではなく置けて読めることの確かめに直す
  - `test_path_helpers_test.rs` の `scriptlibs` が置かれることの確かめ（4 行）だけを消す。`pasta.toml` と `scripts/` の確かめは残す
  - この tasks.md の実装メモに、直したテストと理由（写しの `scriptlibs` に頼っていた）を書く
  - 終わった状態: `cargo test -p pasta_shiori --test test_path_helpers_test` が通り、実装メモに記録がある
  - _Depends: 1.1_
  - _Requirements: 4.1, 4.3_

- [x] 2. 4 つのフィクスチャの `pasta.toml` から、写しを避ける設定とコメントを外す
  - `codegen_runtime_safety`・`actor_proxy_act_delegation`・`call_execution_correctness`・`scene_alias_ontalk` の `lua_search_paths` と回避のコメントを、design.md の表のとおりに消す
  - 回避と関係ない行（見出し、`call_execution_correctness` のウェイトの理由、`scene_alias_ontalk` の別名表の説明、`[loader]` の `pasta_patterns`・`transpiled_output_dir`、`[talk]`・`[ghost]`・`[actor.*]`）は変えない
  - 写しが残ったままだと回避が要るので、1.1 の後に行う
  - 終わった状態: 4 つの `pasta.toml` に `lua_search_paths`・「古い runtime の写し」・「理由は同じ」が無く、4 つの E2E（`codegen_runtime_safety_e2e_test`・`actor_proxy_act_delegation_e2e_test`・`call_execution_correctness_e2e_test`・`scene_alias_ontalk_e2e_test`）が同じ確かめのまま通る
  - _Depends: 1.1_
  - _Requirements: 2.1, 2.2, 2.3, 2.5_

- [x] 3. ゴーストの直下の `pasta_scripts/` への効いていないコピーを消す
- [x] 3.1 (P) `tests/` の 9 ファイルから直下へのコピーを消し、説明を直す
  - 対象: `common/async_callback_support.rs`・`actor_thread_vm_test.rs`・`actor_marshaling_test.rs`・`actor_reload_leak_test.rs`・`actor_teardown_test.rs`・`byte_invariant_test.rs`・`ffi_actor_lifecycle_test.rs`・`ffi_extern_session_e2e_test.rs`・`debug_preservation_test.rs`
  - `.join("pasta_scripts")` のかたまり（コピー元の組み立て・`create_dir_all`・コピーの呼び出し、すぐ上のコメントと後ろの空行）を消す。フィクスチャのコピーに使う再帰コピー関数は残す
  - 説明を design.md の「説明の直し」の表の文に直す（読み込み先は自己展開した `profile/pasta/pasta_scripts/`）。`async_callback_support.rs` 18〜20 行の書き換えで、呼んでいない手順（「`copy_fixture_to_temp` でサポート＋フィクスチャをコピーした後」）の記述も消える
  - 終わった状態: 9 ファイルに `.join("pasta_scripts")` が無く、9 ファイルのテストが同じ確かめのまま通る
  - _Boundary: 別のコピー処理（tests の 9 ファイル）_
  - _Requirements: 5.1, 5.2, 6.3, 6.4_
- [x] 3.2 (P) `actor_kick_test.rs` から直下へのコピーと、使われなくなる部品を消す
  - 直下へのコピーのかたまりと `manifest_dir` の行を消す
  - 使われなくなる再帰コピー関数と後ろの空行を消し、`use std::path::{Path, PathBuf};` を `use std::path::PathBuf;` にする
  - 説明（52〜53 行・78 行）を design.md の表の文に直す
  - 終わった状態: `cargo test -p pasta_shiori --test actor_kick_test` が通り、このファイルについて clippy の未使用の警告が出ない
  - _Boundary: 別のコピー処理（actor_kick_test）_
  - _Requirements: 5.1, 5.2, 6.3, 7.3_
- [x] 3.3 (P) `src/shiori_request_tests.rs` から直下へのコピーとフィクスチャの二度目のコピーを消す
  - 直下へのコピー 2 か所（43〜48 行・471〜476 行）を消し、すぐ上のコメント（38 行・466 行）を `// pasta_lua の scriptlibs をコピーする（標準ランタイムは自己展開されるのでコピーしない）` に書き換え、`// Copy scriptlibs directory`（50 行・478 行）を消す
  - フィクスチャの `scripts/` の二度目のコピー（486〜491 行）とその前の空行を消す
  - `crate_root` と本物の `scriptlibs` のコピーは残す
  - 終わった状態: このファイルに `.join("pasta_scripts")` が無く、`cargo test -p pasta_shiori --lib` が通る
  - _Boundary: 別のコピー処理（src/shiori_request_tests.rs）_
  - _Requirements: 5.2, 5.3_

- [x] 4. 全体を確かめる
- [x] 4.1 grep と、ワークスペースのテスト・clippy で全体を確かめる
  - 走らせる前に `NoDefaultCurrentDirectoryInExePath` と `PASTA_DEBUG` を外す
  - grep で確かめる: `crates/pasta_shiori/tests/support/` が無い、4 つの `pasta.toml` に回避の設定と文言が無い、`crates/pasta_shiori/` の `.rs` に `.join("pasta_scripts")` が無い、`crates/pasta_lua/` と `pasta_shiori` の本番のコードに差分が無い
  - `cargo test --all` と `cargo clippy --all-targets --workspace -- -D warnings` を走らせる
  - 写しにしか無い振る舞いに頼って落ちるテストが出たら、写しを戻さず、そのフィクスチャの `scripts/` に要る差し替えだけを置くか、期待値を本物の結果に直し、実装メモに記録する
  - `pasta_lua` のテストで `sample.generated.lua` に改行だけの差分が出たら `git checkout --` で戻す
  - 終わった状態: テストが全部通り、clippy が警告を出さず、grep の確かめがすべて空
  - _Depends: 1.2, 2, 3.1, 3.2, 3.3_
  - _Requirements: 1.5, 2.3, 3.3, 3.4, 4.2, 5.2, 7.1, 7.2, 7.3_

## 実装メモ

- 1.2: `tests/test_path_helpers_test.rs` の `deep_non_ansi_fixture_is_buildable_and_readable` から `scriptlibs` が置かれることの確かめ（4 行）を消した。理由: `scriptlibs/` は写し（`tests/support/scriptlibs/`）がコピーされることで置かれていただけで、フィクスチャ `shiori_lifecycle` 自身は持たない。テストの意図（深いパス・ANSI で表せないパスの下に一時ゴーストを置けて読める）は `pasta.toml` と `scripts/` の確かめで足りる。
