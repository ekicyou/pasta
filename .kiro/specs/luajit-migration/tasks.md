# Implementation Plan

- [x] 1. 依存設定とStdLibマッピングの一括切り替え
- [x] 1.1 mlua feature切り替え・lua-src除去・StdLibマッピング一括更新
  - ワークスペースCargo.toml: mlua の features を `["lua55", "vendored", "serialize"]` から `["luajit52", "vendored", "serialize"]` に変更する
  - ワークスペースCargo.toml: `lua-src` エントリを完全に削除する
  - pasta_lua/Cargo.toml: `[build-dependencies]` セクションから `lua-src.workspace = true` を削除する（セクションが空になった場合はセクションごと削除）
  - `runtime_config.rs` の `parse_std_lib()` から `"std_utf8" => Ok(StdLib::UTF8)` マッピングを削除する
  - LuaJIT固有マッピングを追加する: `"std_jit" => Ok(StdLib::JIT)`, `"std_ffi" => Ok(StdLib::FFI)`, `"std_bit" => Ok(StdLib::BIT)`
  - `RuntimeConfig` の `pub libs` フィールドのドキュメントコメントと `loader/config.rs` の `LuaConfig.libs` フィールドのドキュメントコメントを更新する: `std_utf8` を削除し `std_jit`, `std_ffi`, `std_bit` を追加
  - `scriptlibs/lua_test/toDebugString.lua` の `table.move` 呼び出し（2箇所）をLua 5.1互換のforループに置換する（GAP-2）
  - `cargo check -p pasta_lua` がコンパイルエラーなく完了すること
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2_
  - _Boundary: Cargo.toml (workspace), crates/pasta_lua/Cargo.toml, crates/pasta_lua/src/runtime/runtime_config.rs, crates/pasta_lua/src/loader/config.rs, crates/pasta_lua/src/error.rs, crates/pasta_lua/tests/runtime/unit_test.rs, crates/pasta_lua/scriptlibs/lua_test/toDebugString.lua_

- [ ] 2. テスト検証
- [ ] 2.1 テストスイート全パス確認
  - `cargo test --workspace` を実行し全テストがパスすることを確認する
  - 特に以下を重点確認: ucid_test.rs（UTF-8識別子）、runtime/unit_test.rs（StdLib設定）、lua_specs/（Luaスクリプトテスト）
  - mlua-stdlib互換性確認: json/regex/yaml機能を使用するテスト（lua_specs/内のdkjson使用テスト等）がLuaJITバックエンドで動作すること
  - `StdLib::UTF8` を使用していた既存テストがあればLuaJIT互換に修正する（`"std_utf8"` → `UnknownLibrary` エラーになることを確認するテスト追加を検討）
  - 下流クレート（pasta_shiori, pasta_check, pasta_sample_ghost）のテストもパスすること
  - テスト全パスのログ出力が確認できること
  - _Requirements: 3.1, 3.2, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3, 6.1, 6.2, 6.3_

- [ ] 3. ステアリングドキュメント更新
- [ ] 3.1 (P) tech.mdのLuaランタイム記載更新
  - `.kiro/steering/tech.md` のLuaランタイム記載を「Lua 5.5 (mlua 0.11)」から「LuaJIT 2.1 (mlua 0.11)」に変更する
  - `lua-src` への言及を除去または「luajit-src（mlua vendored内部使用）」に更新する
  - `lua55` feature への言及を `luajit52` に更新する
  - tech.mdの記載が実際のCargo.toml設定と一致していること
  - _Requirements: 7.1, 7.2_
  - _Boundary: .kiro/steering/tech.md_

## Implementation Notes

- Task 1.1 debug: `UnknownLibrary` 診断文と StdLib 回帰テストは `parse_std_lib()` と同じ設定API面のため、境界に `crates/pasta_lua/src/error.rs` と `crates/pasta_lua/tests/runtime/unit_test.rs` を追加。
