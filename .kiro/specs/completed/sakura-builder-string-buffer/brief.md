# Brief: sakura-builder-string-buffer

## Problem
ゴーストのトーク生成のたびに `pasta.shiori.sakura_builder` の `BUILDER.build` が呼ばれる。
現状は `local buffer = {}` に `table.insert` でトークンを一つずつ蓄積し、最後に
`table.concat(buffer)` で結合している。トークン数が多いトークでは中間テーブルの成長と
GC 負荷が無視できない。LuaJIT には専用の String Buffer Library（`string.buffer`）が
あり、追記が定数時間・GC 負荷も低く、文字列組み立てに最適なのに未活用である。

## Current State
- [sakura_builder.lua](crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua) の `build()` は
  `local buffer = {}` + 多数の `table.insert(buffer, ...)` + `return table.concat(buffer) .. "\\e"`。
- ランタイムは mlua 0.11 / `features = ["luajit52", "vendored"]`（LuaJIT 2.1）。
  `string.buffer` は vendored LuaJIT に同梱されているはずだが、Lua コードからは未参照。
- `pasta/buf.lua` は存在しない。
- 既存テスト [sakura_builder_test.lua](crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua) が
  振る舞いを保護している。

## Desired Outcome
- `pasta/buf.lua` が `new()` を公開する。
  - LuaJIT `string.buffer` が利用可能なら `new = luajit_buf.new`。
  - 利用不可なら、`put` / `tostring`（取り出し）だけの最小実装フォールバックを返す。
- `sakura_builder.build` がバッファ経由で組み立て、**さくらスクリプト出力はバイト一致を維持**する。
- 既存テストが全パスし、回帰がないことを検証する。

## Approach
- **buf.lua**: `local ok, luajit_buf = pcall(require, "string.buffer")` で安全に存在確認する。
  `require("string.buffer")` は失敗時に nil を返さず**例外を投げる**ため pcall が必須。
  成功時は `luajit_buf.new` を `new` として公開。失敗時は table ベースの最小実装
  （`:put(...)` で追記、`:tostring()` で結合取り出し）を返すクロージャを `new` とする。
- **sakura_builder**: `local buffer = {}` を `local buffer = buf.new()` に置換。
  `table.insert(buffer, x)` → `buffer:put(x)`、
  `return table.concat(buffer) .. "\\e"` → `buffer:put("\\e"); return buffer:tostring()`。
- LuaJIT buffer API メモ: `buf:put(...)` 追記、`buf:tostring()` 非破壊取り出し、`buf:get()` 破壊的消費。
  `build()` は一度しか取り出さないため `tostring` を採用（最小実装と API を揃えやすい）。

## Scope
- **In**:
  - `pasta/buf.lua` 新規作成（`new()` + 最小実装フォールバック）
  - `sakura_builder.build` のバッファ化（外部振る舞い不変）
  - 既存テストでの回帰検証、必要なら最小実装フォールバック経路のテスト追加
- **Out**:
  - 他モジュール（`act.lua`, `scene.lua` 等）の文字列組み立てバッファ化
  - `buf.lua` の高度 API（`get`/`set`/`reserve`/`skip`/`encode`/`decode`/FFI 連携）
  - ベンチマーク基盤の整備・性能数値の保証（高速化は期待であり受け入れ条件は振る舞い不変）

## Boundary Candidates
- バッファ抽象（`pasta/buf.lua`）= 再利用可能なユーティリティ層
- さくらスクリプト組み立て（`sakura_builder.build`）= バッファ消費側

## Out of Boundary
- LuaJIT `string.buffer` の直列化（encode/decode）や FFI 連携機能
- `sakura_builder` 以外のホットパス最適化

## Upstream / Downstream
- **Upstream**: LuaJIT `string.buffer`（mlua vendored）、`pasta.shiori.act`（`build` の呼び出し元）
- **Downstream**: 将来 `buf.lua` を他の文字列組み立て箇所（`act.lua` 等）でも再利用可能

## Existing Spec Touchpoints
- **Extends**: なし（既存 spec の境界外の新規最適化）
- **Adjacent**: `audit-pasta-lua`（簡素化監査済み領域）。振る舞い不変のため衝突しない

## Constraints
- **外部振る舞い不変**: さくらスクリプト出力はバイト一致を維持すること
- LuaJIT 2.1 / mlua `luajit52` vendored の範囲内で実装
- `require("string.buffer")` は失敗時に例外を投げる → `pcall` で保護必須
- 最小実装フォールバックは `sakura_builder` が使う API（`put` / `tostring`）を最低限満たすこと
