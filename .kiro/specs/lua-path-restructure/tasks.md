# Implementation Plan

## Overview
8 major tasks, 22 sub-tasks。Tasks 1–2 は並列実行可能。Tasks 3–7 は Tasks 1+2 完了後に並列実行可能。Task 8 は最終検証で逐次実行。

---

- [ ] 1. (P) デフォルト検索パス定義を更新する
  - `config.rs` の `default_lua_search_paths()` 戻り値を新パス順序 `profile/pasta/save/lua` → `scripts` → `pasta_scripts` → `profile/pasta/cache/lua` → `scriptlibs` に変更する
  - `user_scripts` エントリを削除する
  - 関数シグネチャ・`#[serde(default)]` による注入メカニズムは変更しない
  - _Requirements: 1.1, 1.2, 2.3_

- [ ] 2. (P) 標準ランタイムディレクトリをリネームする

- [ ] 2.1 ディレクトリを git mv でリネームする
  - `crates/pasta_lua/scripts/` を `crates/pasta_lua/pasta_scripts/` に `git mv` でリネームする
  - 配下全ファイル（`main.lua`, `ct.lua`, `pasta/` サブツリー）の内容はそのまま保持
  - _Requirements: 2.1, 2.2_

- [ ] 2.2 main.lua 内のコメントを更新する
  - `pasta_scripts/main.lua` 内のコメント中にある `user_scripts` パス参照を `scripts` に更新する
  - ロジック自体は変更しない（コメント文字列のみの修正）
  - _Requirements: 2.2_

- [ ] 3. (P) ディレクトリの役割 README.md を配置する

- [ ] 3.1 (P) pasta_scripts/README.md を配置する
  - `pasta_scripts/README.md` を作成し、pasta 標準ランタイムスクリプトである旨を明記する
  - ゴースト開発者がこのフォルダーを編集すべきでないこと、変更が必要な場合は `scripts/` に同名ファイルを置くことで上書きできる旨を案内する
  - 旧 `scripts/README.md`（リネームにより `pasta_scripts/README.md` となったもの）を上記内容で置き換える
  - _Requirements: 2.4_

- [ ] 3.2 (P) scripts/README.md を配置する
  - `scripts/` ディレクトリを新規作成し、`scripts/README.md` を配置する
  - ゴースト開発者が自由に配置できるカスタム Lua スクリプト用フォルダーであること、および `pasta_scripts/` より優先して読み込まれる旨を明記する
  - 使用例（`scripts/main.lua` で `pasta_scripts/main.lua` を上書きできる）を記載する
  - _Requirements: 2.5_

- [ ] 4. (P) hello.lua 関連ファイルをすべて削除する

- [ ] 4.1 (P) hello.lua を削除する
  - `crates/pasta_lua/pasta_scripts/hello.lua` を `git rm` で削除する
  - _Requirements: 2.6_

- [ ] 4.2 (P) transpiler_test.lua および init.lua エントリを削除する
  - `crates/pasta_lua/tests/lua_specs/transpiler_test.lua` を `git rm` で削除する
  - `crates/pasta_lua/tests/lua_specs/init.lua` の `"transpiler_test"` エントリを削除する
  - _Requirements: 2.6_

- [ ] 4.3 (P) launch.json のデバッグ設定エントリを削除する
  - `.vscode/launch.json` の `"Lua (pasta_lua scripts)"` エントリ（`hello.lua` を `program` とする設定）を削除する
  - 残りの `"Lua (lua_specs tests)"` エントリは変更しない
  - _Requirements: 2.6_

- [ ] 5. (P) hello-pasta サンプルゴーストの設定と配布物を更新する

- [ ] 5.1 (P) 設定ファイル pasta.toml を更新する
  - `crates/pasta_sample_ghost/dist-src/ghost/master/pasta.toml` と `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml` の `lua_search_paths` を新パス構成（`scripts`, `pasta_scripts`）に更新する
  - `user_scripts` エントリを `scripts` に、`scripts` エントリを `pasta_scripts` に変更する
  - _Requirements: 3.1, 3.4_

- [ ] 5.2 (P) リリーススクリプト release.ps1 を更新する
  - `$ScriptsDest` 変数の値を `"pasta_scripts"` に変更する
  - コピー元パス（`scripts/`）を `pasta_scripts/` に更新する
  - コメント内の `scripts/` 参照を `pasta_scripts/` に更新する
  - _Requirements: 3.2, 3.4_

- [ ] 5.3 (P) ソースコードのコメントを更新する
  - `crates/pasta_sample_ghost/src/main.rs` のコメント内 `scripts/` 参照を `pasta_scripts/` に更新する
  - `crates/pasta_sample_ghost/src/lib.rs` のコメント内 `scripts/` 参照を `pasta_scripts/` に更新する
  - _Requirements: 3.4_

- [ ] 5.4 配布物を再生成する
  - `crates/pasta_sample_ghost/` ディレクトリで `release.ps1` を実行する
  - `ghosts/hello-pasta/` 配下の Lua スクリプト群、updates2.dau、updates.txt が新パス構成で再生成されることを確認する（5.1, 5.2 完了後に実行すること）
  - hello.lua が配布物から除外されていることを確認する
  - _Requirements: 3.2, 3.3, 3.4_

- [ ] 6. (P) テストコードのパス参照を新ディレクトリ構成に更新する

- [ ] 6.1 (P) pasta_lua テストヘルパーのパス参照を更新する
  - `crates/pasta_lua/tests/common/mod.rs` の `.join("scripts")` を `.join("pasta_scripts")` に更新する
  - `crates/pasta_lua/tests/common/e2e_helpers.rs` の `.join("scripts")` を `.join("pasta_scripts")` に更新する
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 6.2 (P) ローダーテストのアサーションと一時ディレクトリ設定を更新する
  - `crates/pasta_lua/tests/loader/config_test.rs` のデフォルトパスアサーションを更新する（`user_scripts` 削除、`pasta_scripts` 追加）
  - `crates/pasta_lua/tests/loader/lifecycle_test.rs` の一時ディレクトリ構成を更新する（`user_scripts`→`scripts`、`scripts`→`pasta_scripts`）
  - `crates/pasta_lua/tests/loader/startup_test.rs` の `scripts` 参照を更新する
  - `crates/pasta_lua/tests/loader/lua_passthrough_test.rs` の `"scripts"` 参照を確認・更新する
  - `src/loader/context.rs` 内テストは変更不要（任意パスを渡す汎用テストのため）
  - _Requirements: 4.1, 4.3, 4.4_

- [ ] 6.3 (P) ランタイムテストのパス参照を更新する
  - `crates/pasta_lua/tests/runtime/finalize_scene_test.rs` の `.join("scripts")` を更新する
  - `crates/pasta_lua/tests/runtime/encoding_test.rs` の `"scripts"` 参照を更新する
  - `crates/pasta_lua/tests/transpiler/fallback_search_integration_test.rs` の `.join("scripts")` を更新する
  - _Requirements: 4.1, 4.2_

- [ ] 6.4 (P) pasta_sample_ghost テストのパス参照を更新する
  - `crates/pasta_sample_ghost/tests/common/mod.rs` の `.join("scripts")` を `.join("pasta_scripts")` に更新し、コメントも更新する
  - `crates/pasta_sample_ghost/tests/integration_test.rs` の `user_scripts` アサーションを削除し、`pasta_scripts` アサーションを追加する
  - _Requirements: 4.1, 4.2, 4.4_

- [ ] 7. (P) ドキュメントとステアリングを新構成に更新する

- [ ] 7.1 (P) ステアリング構造ファイルを更新する
  - `.kiro/steering/structure.md` のディレクトリツリー表記で `scripts/` を `pasta_scripts/` に更新する
  - `scripts/`（ユーザー用）と `pasta_scripts/`（標準ランタイム）の役割の違いを記述する
  - _Requirements: 5.1, 5.4_

- [ ] 7.2 (P) pasta_lua README.md を更新する
  - `crates/pasta_lua/README.md` の検索パス説明・コード例を新デフォルト値（`scripts`, `pasta_scripts`）に更新する
  - `scripts`（ユーザー用）・`pasta_scripts`（標準ランタイム）の役割の違いを明記する
  - _Requirements: 5.2, 5.4_

- [ ] 7.3 (P) pasta_sample_ghost ドキュメントを更新する
  - `crates/pasta_sample_ghost/README.md` の `scripts/` 参照を `pasta_scripts/` に更新する
  - `crates/pasta_sample_ghost/RELEASE.md` のビルド手順を新ディレクトリ構成に対応させる
  - _Requirements: 5.3, 5.4_

- [ ] 7.4 (P) TEST_COVERAGE.md を更新する
  - `TEST_COVERAGE.md` の `user_scripts` テスト名を新パス名に更新する
  - _Requirements: 5.4_

- [ ] 7.5 (P) AIスキルファイルのパス参照を更新する
  - `.agents/skills/pasta-lua-coding/SKILL.md` の `scripts/` 参照を `pasta_scripts/` に更新する
  - `.agents/skills/pasta-lua-coding/references/runtime-api.md` のパス参照を更新する
  - `.agents/skills/pasta-lua-coding/references/testing-lint.md` の luacheck パスを更新する
  - _Requirements: 5.4_

- [ ] 8. 全テストを実行して整合性を確認する
  - `cargo test --all` を実行し、全テストがパスすることを確認する
  - テスト失敗時は該当箇所を特定し修正する（Tasks 6.1–6.4 の漏れを確認する）
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
