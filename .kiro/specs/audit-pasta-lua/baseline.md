# pasta_lua 監査ベースライン

> 取得日: 2026-05-29
> 対象: `crates/pasta_lua/src/`

## 1. ソースコード行数

**ファイル数**: 37
**総行数**: 8,538

### ファイル別行数

| ファイル | 行数 |
|---|---:|
| code_gen/element_gen.rs | 528 |
| code_gen/mod.rs | 77 |
| code_gen/scope_gen.rs | 298 |
| config.rs | 93 |
| context.rs | 304 |
| encoding/mod.rs | 129 |
| encoding/unix.rs | 48 |
| encoding/windows.rs | 248 |
| error.rs | 166 |
| lib.rs | 63 |
| loader/cache.rs | 334 |
| loader/config.rs | 377 |
| loader/context.rs | 225 |
| loader/discovery.rs | 155 |
| loader/error.rs | 227 |
| loader/mod.rs | 477 |
| logging/logger.rs | 210 |
| logging/mod.rs | 18 |
| logging/registry.rs | 182 |
| logging/tracing_init.rs | 67 |
| normalize.rs | 168 |
| runtime/enc.rs | 221 |
| runtime/finalize.rs | 256 |
| runtime/log.rs | 241 |
| runtime/mod.rs | 389 |
| runtime/module_registry.rs | 161 |
| runtime/persistence.rs | 433 |
| runtime/runtime_config.rs | 242 |
| sakura_script/line_breaker.rs | 328 |
| sakura_script/mod.rs | 207 |
| sakura_script/tokenizer.rs | 338 |
| sakura_script/wait_inserter.rs | 320 |
| search/context.rs | 236 |
| search/error.rs | 27 |
| search/mod.rs | 72 |
| string_literalizer.rs | 177 |
| transpiler.rs | 496 |

## 2. テスト結果

| テストスイート | ステータス |
|---|---|
| `cargo test -p pasta_lua` | ✅ **643 passed, 0 failed, 9 ignored** |
| `cargo test --workspace` | ✅ **全パス（0 failed）** |

### `cargo test -p pasta_lua` 詳細 (test_results.txt より)

| テストバイナリ | passed | ignored |
|---|---:|---:|
| unittests src/lib.rs | 169 | 0 |
| japanese_identifier_test | 2 | 0 |
| loader (integration) | 96 | 0 |
| log (integration) | 33 | 0 |
| lua_unittest_runner | 1 | 0 |
| runtime (integration) | 119 | 0 |
| sakura_script (integration) | 38 | 0 |
| search (integration) | 29 | 0 |
| shiori_act (integration) | 70 | 0 |
| transpiler (integration) | 80 | 0 |
| word_table (integration) | 3 | 0 |
| doc tests | 3 | 9 |


## 3. unsafe ブロック

**合計: 4箇所**（文字列リテラル中の "unsafe" 言及を除く）

| ファイル | 行 | コード |
|---|---:|---|
| encoding/windows.rs | 112 | `unsafe {` — Windows API `MultiByteToWideChar` 呼び出し |
| encoding/windows.rs | 168 | `unsafe {` — Windows API `WideCharToMultiByte` 呼び出し |
| runtime/enc.rs | 146 | `unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) }` |
| runtime/mod.rs | 101 | `let lua = unsafe { Lua::unsafe_new_with(std_lib, mlua::LuaOptions::default()) };` |

## 4. unwrap() 呼び出し

**合計: 159箇所**

### ファイル別内訳

| ファイル | 件数 | 備考 |
|---|---:|---|
| runtime/persistence.rs | 40 | テストコード含む |
| loader/discovery.rs | 27 | 大部分がテストコード |
| runtime/enc.rs | 23 | テストコード含む |
| sakura_script/tokenizer.rs | 18 | テストコード |
| logging/logger.rs | 13 | テストコード含む |
| sakura_script/mod.rs | 8 | テストコード含む |
| encoding/windows.rs | 8 | テストコード |
| encoding/mod.rs | 6 | テストコード含む |
| encoding/unix.rs | 6 | テストコード |
| logging/registry.rs | 4 | 本番コード（`Mutex::lock().unwrap()`） |
| loader/context.rs | 3 | テストコード |
| runtime/runtime_config.rs | 1 | 本番コード |
| sakura_script/line_breaker.rs | 1 | 本番コード（`Regex::new().unwrap()`） |

## 5. unreachable!() マクロ

**合計: 2箇所**

| ファイル | 行 | コード |
|---|---:|---|
| code_gen/element_gen.rs | 246 | `VarScope::Property => unreachable!()` |
| sakura_script/wait_inserter.rs | 110 | `_ => unreachable!()` |

## 6. 監査チェックリスト

- [x] ファイル別行数ベースライン取得
- [x] `cargo test -p pasta_lua` 全テストパス確認（643 passed, 0 failed）
- [ ] `cargo test --workspace` 全テストパス確認
- [x] `unsafe` ブロック全箇所リスト化
- [x] `unwrap()` 全箇所リスト化
- [x] `unreachable!()` 全箇所リスト化
