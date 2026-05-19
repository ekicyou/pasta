# Brief: luajit-migration

## Problem
pasta_luaクレートはLua 5.5（mlua 0.11 + lua-src vendored）を使用しているが、パフォーマンス（JITコンパイル）、FFI機能、ネイティブUTF-8識別子サポートの観点からLuaJIT 2.1への移行が求められている。

## Current State
- mlua 0.11 の `lua55` + `vendored` + `serialize` featureでLua 5.5をvendored build
- `lua-src` >= 550.0.0 を `ucid` feature付きでbuild-dependencyに追加（UTF-8識別子サポートのため）
- `.luacheckrc` は `std = "lua51"` 設定（Lua 5.1互換の静的解析）
- コードベースにLua 5.3+固有の機能は一切使用されていない
- 950+テストが全パス

## Desired Outcome
- pasta_luaのLuaランタイムがLuaJIT 2.1に変更されている
- UTF-8識別子がLuaJIT 2.1ネイティブサポートで動作している
- `lua-src` 依存が除去されている
- 全テストスイートがパスしている
- ステアリングドキュメント（tech.md）が更新されている

## Approach
**ダイレクト切り替え**: mlua featuresを`lua55`→`luajit52`に変更し、`lua-src`を除去する最小変更アプローチ。LuaJIT 2.1はネイティブでUTF-8識別子をサポートしているため、lua-src/luajit-srcの明示依存は不要。mlua vendored featureが内部でluajit-srcを処理する。

## Scope
- **In**: mlua feature切り替え、lua-src依存除去、テスト確認、ステアリング更新
- **Out**: Lua APIの変更、新規FFI機能の実装、LuaJIT固有最適化の実装

## Boundary Candidates
- Cargo.toml依存設定の変更（ワークスペース + pasta_lua）
- テスト互換性確認（ucid_test.rs含む）
- ドキュメント更新（tech.md）

## Out of Boundary
- LuaJIT FFIを活用した新機能の開発（将来の別spec）
- LuaJIT固有のパフォーマンスチューニング
- luacheckrc設定の変更（既にlua51互換）

## Upstream / Downstream
- **Upstream**: mlua 0.11クレート（LuaJIT公式サポート済み）
- **Downstream**: pasta_shiori、pasta_check、pasta_sample_ghost（pasta_luaに依存）

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: release-workflow（リリースビルドがpasta_luaに依存）

## Constraints
- mlua 0.11の`luajit52` featureを使用（Lua 5.2互換モード）
- vendored build方式を維持（システムLuaJIT不要）
- LuaJIT 2.1はLua 5.1ベース + 一部5.2拡張（luajit52モード）
- mlua-stdlibとの互換性（json, regex, yaml）
