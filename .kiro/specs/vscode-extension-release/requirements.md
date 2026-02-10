# Requirements Document

## Project Description (Input)
editors/vscode にpasta DSLの言語拡張を作成した。実際にvscodeの拡張機能としてリリースするためにREADME文面などを整理したうえでリリースを実施したい。リリースバージョンは本クレートのバージョンと同一とする。最終的に、仕様「release-workflow」と合流し、release-workflowの実施により拡張機能も併せてリリースできるようにするのが最終目標。まずはドキュメント作成からですね。

---

## Introduction

本ドキュメントは Pasta DSL VSCode 拡張機能（`editors/vscode/`）のリリースプロセスに関する要件を定義する。現在、拡張機能の実装（TextMate 文法、セマンティックトークン、WASM ベース LSP 統合）は完了しているが、Visual Studio Code Marketplace への公開に向けたドキュメント整備・バージョン管理・パッケージング・公開手順が未整備である。

本仕様では以下の3つの領域を対象とする：
1. **ドキュメント整備**: Marketplace 公開用 README、CHANGELOG、アイコン等の準備
2. **バージョン同期**: ワークスペースクレートバージョン（`workspace.package.version`）と `package.json` の同期
3. **release-workflow 統合**: 既存仕様 `release-workflow` のリリースフローに VSCode 拡張公開ステップを組み込み

---

## Requirements

### Requirement 1: Marketplace 公開用 README の整備

**Objective:** As a 拡張機能利用者, I want Marketplace ページで拡張機能の概要・機能・使い方が明確に分かるようにしたい, so that インストール判断と初期利用が円滑になる

#### Acceptance Criteria

1. The Pasta VSCode Extension shall Marketplace 公開用 README を `editors/vscode/README.md` に配置する
2. The README shall 拡張機能の概要（1〜2文の説明）をドキュメント冒頭に配置する
3. The README shall 主要機能一覧（TextMate 文法ハイライト、セマンティックトークン、診断情報、フォールバック動作）を箇条書きで列挙する
4. The README shall スクリーンショットまたは GIF による視覚的なデモを少なくとも1つ含む
5. The README shall 対応する VSCode バージョン要件（`engines.vscode` フィールドと整合）を記載する
6. The README shall Pasta DSL の簡潔な紹介と公式リポジトリへのリンクを含む
7. The README shall ライセンス情報（MIT）を記載する
8. The README shall ビルド手順やソースコードに関する詳細は開発者向け情報として分離または省略する（Marketplace README はエンドユーザー向け）
9. When README が更新される, the README shall `package.json` の `description` フィールドと矛盾しない記述を維持する

### Requirement 2: CHANGELOG の作成と管理

**Objective:** As a 拡張機能利用者, I want 各バージョンの変更点を確認したい, so that アップデートの影響範囲を把握できる

#### Acceptance Criteria

1. The Pasta VSCode Extension shall `editors/vscode/CHANGELOG.md` を作成する
2. The CHANGELOG shall [Keep a Changelog](https://keepachangelog.com/) 形式に準拠する
3. The CHANGELOG shall 各リリースバージョンの変更を `Added` / `Changed` / `Fixed` / `Removed` のカテゴリで分類する
4. When 新しいバージョンがリリースされる, the CHANGELOG shall そのバージョンのエントリを先頭に追加する
5. The CHANGELOG shall 初回リリースバージョンのエントリとして、現在実装済みの全機能を `Added` カテゴリに列挙する

### Requirement 3: package.json のメタデータ完備

**Objective:** As a Marketplace 管理者, I want package.json に必要なメタデータが完備されている状態にしたい, so that Marketplace 掲載が正常に行われる

#### Acceptance Criteria

1. The package.json shall `publisher` フィールドに有効なパブリッシャーID（`ekicyou`）を設定する
2. The package.json shall `icon` フィールドにアイコン画像ファイルへのパスを設定する
3. When アイコンが未作成の場合, the package.json shall 最小限のプレースホルダーアイコン（128x128px 以上の PNG）を `editors/vscode/` 配下に配置する
4. The package.json shall `categories` フィールドに `"Programming Languages"` を含める
5. The package.json shall `keywords` フィールドに検索性を高めるキーワード（例: `"pasta"`, `"dsl"`, `"ukagaka"`, `"ghost"`, `"scripting"`）を設定する
6. The package.json shall `badges` フィールドにリポジトリの状態を示すバッジ（任意）を設定できるようにする
7. The package.json shall `repository` フィールドに有効なリポジトリ URL を設定する（現在設定済み）
8. The package.json shall `homepage` フィールドにプロジェクトのホームページ URL を設定する
9. The package.json shall `bugs` フィールドに Issue トラッカー URL を設定する
10. The package.json shall `galleryBanner` フィールドに Marketplace ページの配色を設定する（任意だが推奨）

### Requirement 4: バージョン番号の同期

**Objective:** As a 開発者, I want VSCode 拡張のバージョンをワークスペースクレートバージョンと自動的に同期させたい, so that バージョン不整合が発生しない

#### Acceptance Criteria

1. The Pasta VSCode Extension shall `package.json` の `version` フィールドをワークスペースルート `Cargo.toml` の `[workspace.package].version` と同一の値に保つ
2. When `release-workflow` でバージョン更新が実行される, the Release Workflow shall `editors/vscode/package.json` の `version` フィールドも同時に更新する
3. When バージョン同期が行われる, the Release Workflow shall `editors/vscode/CHANGELOG.md` に新バージョンのエントリ追加を促す（自動生成または手動追加）
4. If `package.json` と `Cargo.toml` のバージョンが不一致の場合, the Release Workflow shall 不一致を検知し警告を表示する
5. The package.json shall semver 形式（`MAJOR.MINOR.PATCH`）のバージョンを使用する

### Requirement 5: VSIX パッケージングと公開手順

**Objective:** As a 開発者, I want VSIX パッケージの作成と Marketplace 公開手順を標準化したい, so that リリースごとに一貫した手順で公開できる

#### Acceptance Criteria

1. The Pasta VSCode Extension shall `npm run package` コマンドで VSIX ファイルを生成できる（現在 `prepackage` スクリプトにより WASM ビルド＋コンパイルも実行される）
2. When VSIX が生成される, the Extension shall ファイル名に `pasta-vscode-<version>.vsix` 形式を使用する
3. The Pasta VSCode Extension shall `vsce publish` コマンドによる Marketplace 公開をサポートする
4. When `vsce publish` を実行する前に, the Extension shall Personal Access Token（PAT）が設定されていることを検証する
5. If PAT が未設定の場合, the Extension shall PAT の取得手順を案内する
6. When Marketplace 公開が成功する, the Extension shall 公開された拡張機能の URL を報告する
7. The Pasta VSCode Extension shall `.vscodeignore` ファイルにより、パッケージに含めないファイル（ソースコード、テスト、ビルドスクリプト等）を指定する

### Requirement 6: release-workflow との統合

**Objective:** As a 開発者, I want `release-workflow` のリリースフローに VSCode 拡張の公開を組み込みたい, so that 単一のリリース操作で全成果物が公開される

#### Acceptance Criteria

1. When `release-workflow` が実行される, the Release Workflow shall Requirement 4 に基づき `package.json` のバージョンを更新する
2. When `release-workflow` の crates.io 公開ステップが完了した後, the Release Workflow shall WASM ビルド → VSIX パッケージング → `vsce publish` を順次実行する
3. If `vsce publish` が失敗する, the Release Workflow shall エラーを報告するが、GitHub Release 作成は続行する（crates.io 公開はロールバックしない）
4. When GitHub Release を作成する, the Release Workflow shall リリースアセットに VSIX ファイルも追加する
5. When VSIX パッケージングを実行する前に, the Release Workflow shall Node.js 依存パッケージ（`npm install`）が最新であることを確認する
6. The Release Workflow shall VSCode 拡張公開の成否をリリースサマリーに含める

### Requirement 7: .vscodeignore の整備

**Objective:** As a 開発者, I want VSIX パッケージに不要なファイルを含めないようにしたい, so that パッケージサイズが最小化される

#### Acceptance Criteria

1. The Pasta VSCode Extension shall `.vscodeignore` ファイルを `editors/vscode/` に配置する
2. The .vscodeignore shall ソースファイル（`src/`）をパッケージから除外する
3. The .vscodeignore shall テストファイル（`src/test/`）をパッケージから除外する
4. The .vscodeignore shall ビルドスクリプト（`scripts/`）をパッケージから除外する
5. The .vscodeignore shall TypeScript 設定ファイル（`tsconfig.json`）をパッケージから除外する
6. The .vscodeignore shall Node.js 開発依存（`node_modules/` の不要部分）をパッケージから除外する
7. The .vscodeignore shall 以下のファイルをパッケージに**含める**：`out/extension.js`、`syntaxes/`、`wasm/`、`language-configuration.json`、`README.md`、`CHANGELOG.md`、`LICENSE`、`package.json`

---

## 備考

### 既存の release-workflow との関係

本仕様の Requirement 4 および Requirement 6 は、既存仕様 `release-workflow` に対する拡張要件として位置づけられる。具体的には：

- **Requirement 2（Cargo.toml バージョン更新）** のステップに `package.json` 更新を追加
- **Requirement 3（crates.io 公開）** と **Requirement 4（サンプルゴーストビルド）** の間に VSCode 拡張パッケージング・公開ステップを挿入
- **Requirement 6（GitHub Release）** のアセットに VSIX ファイルを追加

### 段階的アプローチ

本仕様はまずドキュメント整備（Req 1〜3, 7）を先行し、その後バージョン同期（Req 4）と release-workflow 統合（Req 5〜6）を実施する。これにより、release-workflow 側の設計変更を最小限に抑えつつ、段階的に統合を進めることができる。

### 現状の package.json バージョン不一致

現在、`package.json` の version は `0.1.0` だが、`Cargo.toml` の `workspace.package.version` は `0.1.3` である。Requirement 4 の実施時にこの不一致を解消する必要がある。
