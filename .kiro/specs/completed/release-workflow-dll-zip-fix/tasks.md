# Implementation Plan

## Task Breakdown

### 概要
本仕様は既存の `release-workflow` 仕様を修正し、`pasta.dll` を zip 圧縮してから GitHub Release に添付する工程を追加する。加えて、既存リリース v0.1.5 のアセットを差し替える一回限りの操作を実行する。

**作業スコープ**:
- release-workflow 仕様ドキュメントの修正（`.kiro/specs/release-workflow/`）
- v0.1.5 リリースのアセット差し替え（gh CLI 操作）
- 次回リリース時の動作確認（Phase 4, 6 の zip 圧縮フロー）

---

## Tasks

- [x] 1. release-workflow 仕様ドキュメント更新 (P)
- [x] 1.1 (P) requirements.md への zip 圧縮工程追加
  - Req 4（サンプルゴーストビルド）に以下の AC を追加:
    - AC 4.6: `pasta.dll` 存在確認後、`Compress-Archive -Force` で `pasta.dll.zip` に圧縮する
    - AC 4.7: zip 圧縮完了後、`pasta.dll.zip` の存在を確認する
    - AC 4.8: zip 圧縮失敗時はエラー報告しリリース作業を中断する
  - Req 6（GitHub Release 作成）のアセット記述を `pasta.dll` から `pasta.dll.zip` に変更
  - _Requirements: 2.1, 2.2_

- [x] 1.2 (P) design.md への zip 圧縮フロー追加
  - Phase 4（GhostBuild）セクションに zip 圧縮ステップを追加
  - Phase 4 フローチャートに `pasta.dll` → `pasta.dll.zip` 圧縮ステップを挿入
  - Phase 6（Release）セクションのアセットリストを `pasta.dll.zip` に変更
  - _Requirements: 2.5_

- [x] 1.3 (P) tasks.md への zip 圧縮タスク追加
  - タスク7（Phase 4: ゴーストビルド）に以下を追加:
    - DLL 存在確認直後に `Compress-Archive -Path "target/i686-pc-windows-msvc/release/pasta.dll" -DestinationPath "target/i686-pc-windows-msvc/release/pasta.dll.zip" -Force` を実行
    - `Test-Path "target/i686-pc-windows-msvc/release/pasta.dll.zip"` で zip 確認
    - 失敗時はエラー報告し中断
  - タスク10（Phase 6: GitHub Release 作成）の `$assets` 配列を `"target/i686-pc-windows-msvc/release/pasta.dll.zip"` に変更
  - _Requirements: 2.3, 2.4_

- [x] 2. 既存リリース v0.1.5 のアセット差し替え
- [x] 2.1 DLL ダウンロードと zip 圧縮
  - `gh release download v0.1.5 -p "pasta.dll" -D .` で v0.1.5 から `pasta.dll` をダウンロード
  - `Compress-Archive -Path "pasta.dll" -DestinationPath "pasta.dll.zip" -Force` で zip 圧縮
  - `Test-Path "pasta.dll.zip"` で圧縮成功を確認
  - _Requirements: 3.1, 3.2_

- [x] 2.2 アセット差し替え実行
  - `gh release upload v0.1.5 pasta.dll.zip` で zip をアップロード（元の `pasta.dll` はまだ残存）
  - アップロード成功後、`gh release delete-asset v0.1.5 pasta.dll -y` で旧 DLL を削除
  - `Remove-Item pasta.dll, pasta.dll.zip` で一時ファイルを削除
  - エラー発生時はエラー内容を報告し手動対応手順を案内
  - _Requirements: 3.3, 3.4, 3.5, 3.6_

- [x] 2.3 v0.1.5 アセット確認
  - `gh release view v0.1.5 --json assets -q ".assets[].name"` でアセットリストを取得
  - `pasta.dll.zip` と `hello-pasta.nar` の存在を確認
  - `pasta.dll` が削除されていることを確認
  - _Requirements: 3.7_

- [x] 3. 動作確認と検証
  - release-workflow 仕様の3ファイル（requirements.md, design.md, tasks.md）の diff を確認し、zip 圧縮工程が正しく追加されていることを検証
  - v0.1.5 リリースページで `pasta.dll.zip` がダウンロード可能であることを確認
  - v0.1.5 の `pasta.dll.zip` を展開し、元の `pasta.dll` とファイルサイズが一致することを確認（同一性検証）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

---

## Requirements Coverage

| Requirement | Covered by Tasks |
|-------------|------------------|
| 1.1 - Phase 4 で DLL を zip 圧縮 | 1.1, 1.2, 1.3, 3 |
| 1.2 - zip ファイルの存在確認 | 1.1, 1.2, 1.3, 3 |
| 1.3 - zip 圧縮失敗時の中断 | 1.1, 1.2, 1.3, 3 |
| 1.4 - アセットを `pasta.dll.zip` に変更 | 1.1, 1.2, 1.3, 3 |
| 1.5 - `Compress-Archive -Force` 使用 | 1.1, 1.2, 1.3, 3 |
| 2.1 - requirements.md Req 4 修正 | 1.1, 3 |
| 2.2 - requirements.md Req 6 修正 | 1.1, 3 |
| 2.3 - tasks.md タスク7 修正 | 1.3, 3 |
| 2.4 - tasks.md タスク10 修正 | 1.3, 3 |
| 2.5 - design.md 修正 | 1.2, 3 |
| 3.1 - v0.1.5 DLL ダウンロード | 2.1, 2.3, 3 |
| 3.2 - ダウンロード DLL の zip 圧縮 | 2.1, 2.3, 3 |
| 3.3 - v0.1.5 アセット削除 | 2.2, 2.3, 3 |
| 3.4 - v0.1.5 に zip アップロード | 2.2, 2.3, 3 |
| 3.5 - 一時ファイル削除 | 2.2, 3 |
| 3.6 - エラー時の手動対応案内 | 2.2, 3 |
| 3.7 - アセット存在確認 | 2.3, 3 |

**全要件カバー済み**: ✅ 17/17 AC
