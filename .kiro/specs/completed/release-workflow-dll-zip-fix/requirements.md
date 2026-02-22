# Requirements Document

## Project Description (Input)
リリースのうち、「pasta.dll」がzip圧縮されずにAssetsに含まれている。リリース作業に「pasta.dll.zip」への圧縮タスクを追加して、次回リリースからzipにするように仕様を変更してほしい。合わせて現在の「pasta v0.1.5」リリースの「pasta.dll」を、「pasta.dll.zip」に変更する。

---

## Introduction

本ドキュメントは、既存のリリースワークフロー仕様（`release-workflow`）に対する修正要件を定義する。現行のリリースでは `pasta.dll` が zip 圧縮されずに GitHub Release の Assets に直接添付されているが、これを `pasta.dll.zip` に変更し、ダウンロード時の利便性とファイル整合性を向上させる。

本仕様は以下の2つの作業スコープを含む：

1. **仕様変更（恒久対応）**: `release-workflow` の要件・タスクに zip 圧縮工程を追加し、今後のリリースで `pasta.dll.zip` が Assets に含まれるようにする
2. **既存リリース修正（一回限り）**: 現行の「pasta v0.1.5」リリースの Assets から `pasta.dll` を削除し、`pasta.dll.zip` に差し替える

---

## Requirements

### Requirement 1: release-workflow 仕様への zip 圧縮工程の追加

**Objective:** As a 開発者, I want リリース成果物の `pasta.dll` を zip 圧縮してから GitHub Release に添付したい, so that ダウンロード時のファイルサイズ削減とブラウザによるDLL直接ダウンロードブロックを回避できる

#### Acceptance Criteria

1. When ゴーストビルド（Phase 4）が成功する, the Release Workflow shall `target/i686-pc-windows-msvc/release/pasta.dll` を `target/i686-pc-windows-msvc/release/pasta.dll.zip` に圧縮する
2. When `pasta.dll` の zip 圧縮が完了する, the Release Workflow shall 圧縮後の `pasta.dll.zip` ファイルの存在を確認する
3. If zip 圧縮が失敗する, the Release Workflow shall エラーを報告しリリース作業を中断する
4. When GitHub Release を作成する, the Release Workflow shall アセットとして `pasta.dll` の代わりに `pasta.dll.zip` を添付する
5. The Release Workflow shall zip 圧縮に PowerShell の `Compress-Archive -Force` コマンドレットを使用し、既存ファイルがあっても上書きする

### Requirement 2: release-workflow 仕様ドキュメントの更新

**Objective:** As a 開発者, I want release-workflow 仕様の要件・設計・タスクドキュメントを更新して zip 圧縮工程を反映したい, so that 今後のリリース実行時に自動的に zip 圧縮が行われる

#### Acceptance Criteria

1. When 本仕様が実装される, the Release Workflow shall `release-workflow/requirements.md` の Requirement 4（サンプルゴーストビルド）に zip 圧縮の受入基準を追加する
2. When 本仕様が実装される, the Release Workflow shall `release-workflow/requirements.md` の Requirement 6（GitHub Release 作成）のアセット記述を `pasta.dll` から `pasta.dll.zip` に変更する
3. When 本仕様が実装される, the Release Workflow shall `release-workflow/tasks.md` の Phase 4（タスク7）に zip 圧縮ステップを追加する
4. When 本仕様が実装される, the Release Workflow shall `release-workflow/tasks.md` の Phase 6（タスク10）のアセットリストを `pasta.dll.zip` に変更する
5. When 本仕様が実装される, the Release Workflow shall `release-workflow/design.md` の該当箇所を zip 圧縮工程に合わせて更新する

### Requirement 3: 既存リリース「pasta v0.1.5」のアセット差し替え

**Objective:** As a 開発者, I want 現行の「pasta v0.1.5」リリースの `pasta.dll` を `pasta.dll.zip` に差し替えたい, so that 既存リリースのアセットも統一された形式になる

#### Acceptance Criteria

1. When 本仕様が実装される, the Release Workflow shall `gh release download v0.1.5 -p pasta.dll -D .` コマンドで v0.1.5 リリースから `pasta.dll` をダウンロードする
2. When `pasta.dll` のダウンロードが完了する, the Release Workflow shall ダウンロードした `pasta.dll` を `Compress-Archive -Force` で `pasta.dll.zip` に圧縮する
3. When `pasta.dll.zip` の生成が完了する, the Release Workflow shall `gh release delete-asset v0.1.5 pasta.dll -y` コマンドでリリースから `pasta.dll` アセットを削除する
4. When `pasta.dll` アセットの削除が完了する, the Release Workflow shall `gh release upload v0.1.5 pasta.dll.zip` コマンドで v0.1.5 リリースに `pasta.dll.zip` をアップロードする
5. When アセット差し替えが完了する, the Release Workflow shall ダウンロードした一時ファイル（`pasta.dll`, `pasta.dll.zip`）を削除する
6. If `gh release download`, `gh release delete-asset`, または `gh release upload` が失敗する, the Release Workflow shall エラーを報告し手動での対応手順を案内する
7. When アセット差し替えが完了する, the Release Workflow shall v0.1.5 リリースページに少なくとも `pasta.dll.zip` と `hello-pasta.nar` が存在することを確認する

---

## 既存仕様との関係

### 変更対象

| ファイル | 変更内容 |
|----------|----------|
| `release-workflow/requirements.md` | Req 4 に zip 圧縮基準追加、Req 6 のアセット記述を `pasta.dll.zip` に変更 |
| `release-workflow/design.md` | Phase 4 以降のフローに zip 圧縮ステップを追加 |
| `release-workflow/tasks.md` | タスク7 に zip 圧縮追加、タスク10 のアセットリストを変更 |

### 変更しない部分

- Phase 0〜3（前提条件確認、事前検証、バージョン更新、crates.io 公開）は変更なし
- `release.ps1` スクリプトは変更不要（DLL ビルド自体は従来通り）
- コミットメッセージ規約やタグ管理は変更なし

