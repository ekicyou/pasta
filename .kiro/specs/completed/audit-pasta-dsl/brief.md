# Brief: audit-pasta-dsl

## Problem
pasta_dslクレートはPest PEGパーサーを用いたDSL解析層で~2500行に成長している。パーサーは外部入力（.pastaファイル）を直接処理するため、入力検証・エラーハンドリングの堅牢性が重要。大きなparser/mod.rs（800+行）は複雑度が高く、簡素化の余地がある。

## Current State
- ~2500行のソースコード（src/ 20+ファイル）
- parser/mod.rs が800+行（最大ファイルの一つ）
- Pest 2.8.6 による PEG パーサー生成
- 外部依存: `pest`, `pest_derive`, `thiserror`
- `read_to_string()` によるファイルI/O（エラーハンドリング済み）

## Desired Outcome
- 外部入力（.pastaファイル）の解析パスにおける堅牢性検証完了
- parser/mod.rs の複雑度削減・分割検討
- デッドコード除去、冗長表現削減
- 既存テスト全パス、外部振る舞い不変

## Approach
クレート内完結型監査。パーサーの入力検証パス、エラーリカバリ、ASTビルド処理を重点的に調査し、コード簡素化を行う。

## Scope
- **In**: pasta_dsl/src/ 全ファイルの脆弱性調査、parser/mod.rsの複雑度削減、デッドコード除去、冗長表現削減
- **Out**: Pest文法定義（pasta.pest）の変更、AST型の公開インターフェース変更、新しい構文の追加

## Boundary Candidates
- parser/mod.rs のAST構築ロジック
- 属性マージ処理
- partial.rs のパーシャルパース
- error.rs のエラー型定義

## Out of Boundary
- Pasta DSL文法仕様の変更
- 新しいDSL構文の追加
- pasta_core への変更

## Upstream / Downstream
- **Upstream**: pasta_core（レジストリ型を参照）
- **Downstream**: pasta_lua, pasta_lsp が依存

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: audit-pasta-core（上流）、audit-pasta-lua（下流）、choice-definition-dsl（将来の文法拡張spec）

## Constraints
- 外部振る舞い（公開API・AST型）不変
- Pest文法定義は変更しない
- 既存テスト全パス必須
- 性能劣化禁止
