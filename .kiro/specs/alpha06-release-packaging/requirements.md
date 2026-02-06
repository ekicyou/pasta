# Requirements Document

## Introduction

hello-pasta ゴーストを `.nar` 形式で GitHub Releases に公開するための手動リリースワークフローを定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: alpha01〜05（ビルド基盤・ゴースト生成基盤が前提）
- **目的**: テスターが SSP に直接インストール可能な `.nar` ファイルを GitHub Releases で入手できる状態を実現する

### 既存基盤

| 基盤 | 状態 | 備考 |
|------|------|------|
| `setup.bat` | ✅ 実装済み | DLL ビルド → ゴースト生成 → コピー → finalize の 4 ステップ |
| `pasta_sample_ghost` クレート | ✅ 実装済み | ゴーストファイル・画像・設定の自動生成 |
| GitHub CLI (`gh`) | ✅ 認証済み | `gh release create` でリリース公開可能 |
| CI (`build.yml`) | ✅ 実装済み | x86/x64 マトリックスビルド |

### リリース成果物

- **hello-pasta.nar**: ゴースト配布ファイル（ZIP を `.nar` に拡張子変更したもの）
- **GitHub Releases ページ**: `.nar` アセットの添付 + リリースノート + インストール手順

---

## Requirements

### Requirement 1: .nar ファイル生成

**Objective:** As a 配布担当者, I want `setup.bat` で生成されたゴーストを `.nar` 形式にパッケージングしたい, so that SSP 標準の配布形式でテスターに提供できる

#### Acceptance Criteria

1. The パッケージングプロセス shall `ghosts/hello-pasta/` ディレクトリ配下を ZIP 圧縮し、拡張子を `.nar` に変更して `hello-pasta.nar` を生成する
2. The `.nar` ファイル shall 以下のディレクトリ構成を含む:
   - `ghost/master/pasta.dll` — SHIORI DLL (x86)
   - `ghost/master/pasta.toml` — 設定ファイル
   - `ghost/master/descript.txt` — ゴースト設定
   - `ghost/master/dic/*.pasta` — Pasta DSL スクリプト
   - `ghost/master/scripts/` — Lua ランタイム
   - `shell/master/descript.txt` — シェル設定
   - `shell/master/surfaces.txt` — サーフェス定義
   - `shell/master/surface*.png` — ピクトグラム画像
   - `install.txt` — インストール情報
   - `updates.txt`, `updates2.dau` — 更新情報
3. The `.nar` 生成 shall `setup.bat` 実行完了後の手動操作として実施される

---

### Requirement 2: GitHub Releases への公開

**Objective:** As a テスター, I want GitHub Releases から `.nar` ファイルをダウンロードしたい, so that 最新のゴーストを入手できる

#### Acceptance Criteria

1. The リリース公開 shall `gh release create` コマンドで実行される
2. The リリースタグ shall `v<version>` 形式とする（例: `v0.1.1`）
3. The タグのバージョン shall ワークスペース `Cargo.toml` の `workspace.package.version` と一致すること
4. The GitHub Releases ページ shall `hello-pasta.nar` をアセットとして添付する
5. The リリース本文 shall リリースノートおよびインストール手順を含む

---

### Requirement 3: リリースノートとインストール手順

**Objective:** As a テスター, I want リリース内容とインストール方法を理解したい, so that 迷わずゴーストを使い始められる

#### Acceptance Criteria

1. The リリース本文 shall 以下の情報を含む:
   - バージョン番号
   - リリース概要（変更点・新機能）
   - 含まれるコンポーネント（pasta.dll バージョン、hello-pasta ゴースト）
   - 必要環境（SSP 2.x 以上、Windows x86）
2. The リリース本文 shall 以下のインストール手順を含む:
   - `.nar` ファイルを SSP にドラッグ＆ドロップ（推奨）
   - または SSP のゴーストフォルダに手動展開
3. The リリース本文 shall 動作確認方法を含む:
   - SSP のゴースト切り替え手順
   - 正常動作の確認ポイント
4. The リリース本文 shall 問題報告先（GitHub Issues）を記載する

---

### Requirement 4: リリース前検証

**Objective:** As a 配布担当者, I want リリース前にゴーストの完全性を確認したい, so that 不完全な配布物の公開を防げる

#### Acceptance Criteria

1. When リリース前検証を実施する場合, the 配布担当者 shall `ghosts/hello-pasta/` に対して以下を確認する:
   - `ghost/master/pasta.dll` が存在し、ファイルサイズが 0 でないこと
   - `ghost/master/pasta.toml` が存在すること
   - `ghost/master/dic/` 配下に `.pasta` ファイルが存在すること
   - `ghost/master/scripts/` 配下に Lua ランタイムファイルが存在すること
   - `shell/master/` 配下に画像ファイルが存在すること
   - `install.txt`, `updates.txt`, `updates2.dau` が存在すること
2. If いずれかの確認が失敗した場合, the 配布担当者 shall リリースを中断し、`setup.bat` を再実行する

---

## Out of Scope

- GitHub Actions による自動リリース（手動実行が基本）
- x64 版リリース
- 自動更新機能（SSP ネットワーク更新）
- インストーラー（.msi, .exe）
- 多言語ドキュメント
- 署名付きバイナリ
- crates.io への公開（別途仕様）

---

## Glossary

| 用語 | 説明 |
|------|------|
| `.nar` | ZIP を `.nar` 拡張子に変更したもの。伺か ゴースト配布形式 |
| SSP | 伺か標準ベースウェア。デスクトップマスコット実行環境 |
| SHIORI DLL | SSP が読み込む対話エンジン DLL。`pasta.dll` がこれに該当 |
| hello-pasta | `pasta_sample_ghost` クレートで生成されるサンプルゴースト |
| `setup.bat` | DLL ビルド → ゴースト生成 → ファイルコピー → finalize を実行するバッチスクリプト |
| GitHub CLI (`gh`) | GitHub をコマンドラインから操作するツール |
