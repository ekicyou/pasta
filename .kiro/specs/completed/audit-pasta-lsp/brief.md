# Brief: audit-pasta-lsp

## Problem
pasta_lspはtower-lsp ベースのLSPサーバーラッパーで比較的小規模（~400行）だが、WASM対応コード（wasm-bindgen, js-sys）を含む。ネットワーク経由のJSON-RPCリクエスト処理があるため、入力検証の確認が必要。

## Current State
- ~400行のソースコード（src/ 6ファイル）
- tower-lsp 0.20 ベース
- WASM対応（cfg(wasm32)条件コンパイル）
- 16テストファイル
- 外部依存: `tower-lsp`, `serde`, `serde_json`, `thiserror`, `wasm-bindgen`, `js-sys`

## Desired Outcome
- JSON-RPC入力処理の安全性検証
- WASM境界の安全性確認
- デッドコード除去、冗長表現削減
- 既存テスト全パス、外部振る舞い不変

## Approach
クレート内完結型監査。LSPプロトコルハンドラの入力検証→WASM境界→デッドコードの順に調査する。

## Scope
- **In**: pasta_lsp/src/ 全ファイルの脆弱性調査、入力検証確認、デッドコード除去、冗長表現削減
- **Out**: LSPプロトコル仕様の変更、tower-lspの内部実装、新しいLSP機能追加

## Boundary Candidates
- LSPリクエストハンドラ
- WASM対応コード（cfg(wasm32)）
- エラー型定義

## Out of Boundary
- tower-lsp クレートの内部実装
- VS Code拡張（editors/vscode/）の変更
- pasta_dsl パーサーの変更

## Upstream / Downstream
- **Upstream**: pasta_dsl（パーサー）
- **Downstream**: なし（エンドユーザー向けLSP）

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: audit-pasta-dsl（上流パーサー）

## Constraints
- 外部振る舞い（LSPレスポンス）不変
- WASM互換性維持
- 既存テスト全パス必須
