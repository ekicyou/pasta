# Requirements Document

## Introduction

本ドキュメントは、Pasta DSL（`*.pasta`ファイル）のシンタックスハイライトを提供するVSCode拡張「pasta-vscode」の要件を定義する。既存の`pasta_lsp`クレート（14セマンティックトークンタイプ、部分パースフォールバック、クラッシュ保護実装済み、WASMビルド対応）を活用し、TypeScriptベースのVSCode拡張として実装する。

**実装スコープ**：本仕様は **Phase 3（WASM インプロセスLSP統合）** を最終ゴールとする。Gap分析で推奨された段階的実装アプローチ（Option C）に従い、Phase 1（TextMate文法）で基本ハイライトを実現し、Phase 3（WASM統合）でLSPセマンティックハイライトを完成させる。

> **Note (Phase 2の位置づけ)**: Phase 2（ネイティブLSP統合）は任意実装とし、WASM transport 実装前のLSP機能検証、またはWASM実装が困難だった場合のフォールバックとして位置づける。設計フェーズで実装の必要性を判断する。

配置場所については、Rustクレートではなくnpmベースのプロジェクトであるため、`crates/`配下ではなくワークスペースルート直下の`editors/vscode/`に配置する（エディタ拡張は`editors/`配下に配置する慣例が広く採用されており、将来的に他エディタ対応も見据えた構造とする）。

## Requirements

### Requirement 1: 拡張の基盤構造

**Objective:** 開発者として、pasta-vscode拡張が標準的なVSCode拡張の構造に従い、インストール・起動できること。

#### Acceptance Criteria

1. The pasta-vscode extension shall `editors/vscode/`配下にTypeScript + npmベースのプロジェクトとして構成される
2. The pasta-vscode extension shall `package.json`にVSCode拡張マニフェスト（`contributes`、`engines`、`activationEvents`）を含む
3. When `*.pasta`ファイルを開いた時、the pasta-vscode extension shall 自動的にアクティベートされる
4. The pasta-vscode extension shall VSIXパッケージとしてビルド可能である
5. The pasta-vscode extension shall `pasta`言語IDをVSCodeに登録し、`*.pasta`ファイル拡張子と関連付ける

### Requirement 2: LSPサーバー統合

**Objective:** 開発者として、pasta_lspが提供するLSP機能（セマンティックトークン、診断情報、ドキュメント同期）がVSCode拡張内で利用できること。

> **Note (Phase 3 - WASM統合)**: 本仕様のゴールは **WASM インプロセスでのLSPサーバー統合** である。pasta_lsp.wasm をVSCode拡張内でロードし、wasm-bindgen 経由でLSPプロトコルを処理する。Gap分析で指摘されたWASM transport実装の技術的リスク（High）については、設計フェーズで具体的な実装アプローチを決定する。

> **Note (Phase 2 - 任意実装)**: ネイティブ実行ファイルとしてのLSPサーバー起動（stdio経由）は、WASM実装前のLSP機能検証、またはWASM実装が困難な場合のフォールバックとして任意実装とする。設計フェーズでPhase 2実装の必要性を判断する。

#### Acceptance Criteria

1. The pasta-vscode extension shall pasta_lspのビルド成果物を拡張パッケージに含める、またはパスで参照できる構成とする
2. When 拡張がアクティベートされた時、the pasta-vscode extension shall LSPサーバーを起動する
3. The pasta-vscode extension shall LSPクライアント（`vscode-languageclient`）を使用してLSPプロトコルを処理する
4. The pasta-vscode extension shall LSP標準のドキュメント同期（`textDocument/didOpen`, `didChange`, `didClose`）をサポートする
5. When pasta_lspが診断情報を送信した時、the pasta-vscode extension shall エラーをVSCodeの問題パネルに表示する
6. The pasta-vscode extension shall UTF-16ベースの位置情報を正しく処理する（日本語・全角文字対応）
7. If LSPサーバーの起動に失敗した場合、the pasta-vscode extension shall ユーザーにエラー通知を表示し、TextMate文法のみのフォールバックモードで動作する

> **Note (LSP標準機能)**: ドキュメント同期、診断情報表示、UTF-16位置変換はLSPクライアント（`vscode-languageclient`）が自動処理する標準機能であり、拡張側での特別な実装は不要である。

### Requirement 3: セマンティックハイライト

**Objective:** 開発者として、`*.pasta`ファイルの構文要素が意味的に色分け表示されること。

#### Acceptance Criteria

1. The pasta-vscode extension shall pasta_lspが提供する14セマンティックトークンタイプを全てサポートする:
   - comment, namespace, scene, decorator, word, variable, call, actor, actorName, codeBlock, string, sakuraScript, escape, operator
2. The pasta-vscode extension shall pasta_lspが提供する3モディファイアをサポートする
3. When `*.pasta`ファイルを開いた時、the pasta-vscode extension shall セマンティックトークンをリクエストしハイライト表示する
4. When `*.pasta`ファイルを編集した時、the pasta-vscode extension shall セマンティックトークンを再取得しハイライトを更新する

> **Note**: 全角/半角マーカーの同等認識はpasta_lsp（LSPサーバー側）の責務として既に実装済み。TextMate文法でのフォールバック対応はRequirement 4に含む。

### Requirement 4: TextMate文法による基本ハイライト（フォールバック）

**Objective:** 開発者として、LSPが利用できない場合やセマンティックトークンが返される前でも、最低限のシンタックスカラーリングが提供されること。

#### Acceptance Criteria

1. The pasta-vscode extension shall TextMate文法定義（`.tmLanguage.json`）をバンドルする
2. The pasta-vscode extension shall TextMate文法で以下の基本要素をハイライトする:
   - コメント行（`＃` / `#`）
   - グローバルシーン（`＊` / `*`）
   - ローカルシーン（`・` / `-`）
   - 属性定義（`＆` / `&`）
   - 単語定義（`＠` / `@`）
   - 変数参照（`＄` / `$`）
   - Call/Jump文（`＞` / `>`）
   - アクター定義（`％` / `%`）
   - Luaコードブロック
3. The pasta-vscode extension shall TextMate文法で全角マーカー（`＊`、`＃`、`・`、`＆`、`＠`、`＄`、`＞`、`％`）と半角マーカー（`*`、`#`、`-`、`&`、`@`、`$`、`>`、`%`）を同等に認識してハイライトする
4. While セマンティックトークンが利用可能である場合、the pasta-vscode extension shall セマンティックハイライトでTextMate文法のハイライトを上書きする
5. If LSPサーバーが起動に失敗した場合、the pasta-vscode extension shall TextMate文法のみでの基本ハイライトにフォールバックする

### Requirement 5: プロジェクト配置と構成

**Objective:** メンテナーとして、pasta-vscode拡張がpastaワークスペース内の適切な場所に配置され、ビルド・パッケージングが可能であること。

#### Acceptance Criteria

1. The pasta-vscode extension shall ワークスペースルートの`editors/vscode/`ディレクトリに配置される
2. The pasta-vscode extension shall WASMビルド成果物（`pasta_lsp.wasm`+JSバインディング）を拡張パッケージに取り込むビルドスクリプトを持つ
3. The pasta-vscode extension shall `vsce package`コマンドでVSIXファイルを生成できる
4. The pasta-vscode extension shall READMEにビルド・インストール手順を記載する
5. The pasta-vscode extension shall `.kiro/steering/structure.md`にて新規ディレクトリとして登録される
