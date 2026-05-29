# Brief: audit-pasta-shiori

## Problem
pasta_shioriはSHIORI DLLインターフェース層で、プロジェクト内のunsafeコード集中箇所（10ブロック）。Windows HGLOBAL メモリ管理、extern "C" FFI境界（3関数）、SHIORIリクエストパーサー（req_parser.rs 800+行）を含む。FFI境界はメモリ安全性の最重要ポイント。また11箇所の`#[allow(dead_code)]`がある。

## Current State
- ~1500行のソースコード（src/ 8モジュール）
- 10個の `unsafe` ブロック（windows.rs, hglobal/mod.rs に集中）
- 3個の `extern "C"` 関数（windows.rs L50-76）
- req_parser.rs: 800+行（SHIORIプロトコルパーサー、27箇所の`unwrap_or_else(panic!)`)
- hglobal/mod.rs: 11個の `#[allow(dead_code)]`
- 外部依存: `pest`, `time`, `tracing`, `thiserror`, `windows-sys`

## Desired Outcome
- 全unsafeブロックの安全性検証・ドキュメント化（SAFETY コメント）
- FFI境界の入力検証強化
- req_parser.rs のパニック除去（Result伝搬への変換）
- デッドコード除去（#[allow(dead_code)] 11箇所の精査）
- 冗長表現削減
- 既存テスト全パス、外部振る舞い不変

## Approach
クレート内完結型監査。FFI境界（unsafe + extern "C"）を最優先で調査し、次にパーサー堅牢性、最後にデッドコード・冗長表現を処理する。

## Scope
- **In**: pasta_shiori/src/ 全ファイルの脆弱性調査、unsafe安全性検証、FFI境界強化、パーサー堅牢化、デッドコード除去
- **Out**: SHIORIプロトコル仕様の変更、Windows API呼び出しパターンの設計変更、新しいSHIORI機能追加

## Boundary Candidates
- windows.rs: DLLエクスポート・extern "C" 関数
- hglobal/mod.rs: HGLOBALメモリ管理ユーティリティ
- util/parsers/req_parser.rs: SHIORIリクエストパーサー
- エラーハンドリング・ログ出力

## Out of Boundary
- Windows API (windows-sys) クレートの内部
- SHIORIプロトコル仕様自体の変更
- pasta_lua ランタイムの変更

## Upstream / Downstream
- **Upstream**: pasta_core, pasta_lua
- **Downstream**: なし（最終出力層・DLL）

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: audit-pasta-lua（上流）、release-workflow（リリース手順で参照）

## Constraints
- 外部振る舞い（SHIORI API応答）不変
- Windows DLLエクスポートシグネチャ不変
- unsafe ブロックは必要最小限に留める（ゼロにはできない可能性あり）
- 既存テスト全パス必須
- 性能劣化禁止
