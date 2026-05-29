# Brief: audit-pasta-lua

## Problem
pasta_luaはプロジェクト最大のクレート（~4000行）で、コード生成・トランスパイラ・ランタイムの3大モジュールを含む。element_gen.rs（600+行）、finalize.rs（500+行）、transpiler.rs（500+行）が複雑度ホットスポット。`lua.load().eval()` によるLua実行、`unsafe_new_with` によるLua VM初期化、ファイルI/O操作など、セキュリティ上の注意点が集中している。

## Current State
- ~4000行のソースコード（src/ 15+モジュール）
- code_gen/element_gen.rs: 600+行（コード生成ロジック、`unreachable!()` 使用あり）
- runtime/finalize.rs: 500+行（Luaレジストリ収集、ネストテーブル走査）
- transpiler.rs: 500+行（マルチフェーズトランスパイル）
- 2箇所の `unsafe` ブロック（Lua VM初期化）
- `lua.load().eval()` の使用（ハードコードされたrequire呼び出し、注入リスク低）
- Lua側スクリプト群: scripts/, scriptlibs/, pasta_scripts/

## Desired Outcome
- 3大モジュールの複雑度削減（可能なら各400行以下）
- unsafe使用箇所の安全性ドキュメント化または代替手段検討
- Lua実行パスの安全性検証完了
- デッドコード除去、冗長表現削減、アルゴリズム改善
- 既存テスト全パス、外部振る舞い不変

## Approach
クレート内完結型監査。module単位（codegen → runtime → transpiler → scripts）でタスクを分割し、段階的に調査・簡素化する。APIは変更せず内部実装のみ対象。

## Scope
- **In**: pasta_lua/src/ 全モジュールの脆弱性調査・コード簡素化。Lua側スクリプト（scripts/, scriptlibs/, pasta_scripts/）の安全性調査
- **Out**: mlua/LuaJITの内部実装、公開APIシグネチャ変更、新機能追加

## Boundary Candidates
- code_gen/ モジュール（Lua コード生成）
- runtime/ モジュール（Lua VM管理・レジストリ操作）
- transpiler モジュール（DSL→Lua変換パイプライン）
- loader/ モジュール（ファイル読み込み・キャッシュ）
- Lua側スクリプト群

## Out of Boundary
- mlua クレートの内部実装
- LuaJIT ランタイムの修正
- Pasta DSL文法の変更
- SHIORI プロトコル処理

## Upstream / Downstream
- **Upstream**: pasta_core（レジストリ）、pasta_dsl（AST）
- **Downstream**: pasta_shiori が依存

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: audit-pasta-core, audit-pasta-dsl（上流）、audit-pasta-shiori（下流）

## Constraints
- 外部振る舞い（公開API）不変
- Lua側スクリプトの動作互換性維持
- 既存テスト全パス必須
- 性能劣化禁止（特にトランスパイル・コード生成のホットパス）
