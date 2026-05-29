# Brief: audit-pasta-check

## Problem
pasta_checkはリリースCLIツールで、ファイルI/O（NAR作成、更新ファイル生成）やMD5ハッシュ計算を行う。ファイルパス操作やアーカイブ処理における入力検証、パストラバーサル対策の確認が必要。

## Current State
- ~500行のソースコード（src/ 5ファイル）
- update_files.rs: ファイル走査・MD5ハッシュ・updates.txt生成
- nar.rs: ZIPアーカイブ（NAR）作成
- copy.rs: ファイルコピー操作
- 外部依存: `lexopt`, `md5`, `zip`, `thiserror`
- Result伝搬によるエラーハンドリング（良好）

## Desired Outcome
- ファイルパス操作のパストラバーサル安全性検証
- アーカイブ処理（zip）の安全性検証
- MD5使用箇所のセキュリティ評価（用途が適切か）
- デッドコード除去、冗長表現削減
- 既存テスト全パス、外部振る舞い不変

## Approach
クレート内完結型監査。ファイルI/Oパス→アーカイブ処理→ハッシュ計算→CLI入力検証の順に調査する。

## Scope
- **In**: pasta_check/src/ 全ファイルの脆弱性調査、パストラバーサル検証、デッドコード除去、冗長表現削減
- **Out**: CLIインターフェースの変更、NARフォーマット仕様の変更、新しいサブコマンド追加

## Boundary Candidates
- nar.rs: ZIPアーカイブ作成処理
- update_files.rs: ファイル走査・ハッシュ生成
- copy.rs: ファイルコピー操作
- main.rs: CLI引数解析

## Out of Boundary
- NARフォーマット仕様
- リリースワークフロー全体（release-workflow specの範囲）
- pasta_lua との統合

## Upstream / Downstream
- **Upstream**: pasta_lua（将来のLuaテスト基盤として依存）
- **Downstream**: なし（エンドユーザー向けCLI）

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: release-workflow（リリース手順specで利用される）、pasta-check skill

## Constraints
- 外部振る舞い（CLI出力・生成ファイル）不変
- NAR互換性維持
- 既存テスト全パス必須
