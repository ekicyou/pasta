# Requirements Document

## Introduction

本ドキュメントは、Pasta DSL（`*.pasta`ファイル）のシンタックスハイライトを提供するVSCode拡張「pasta-vscode」の要件を定義する。既存の`pasta_lsp`クレート（14セマンティックトークンタイプ、部分パースフォールバック、クラッシュ保護実装済み、WASMビルド対応）を活用し、TypeScriptベースのVSCode拡張として実装する。LSPサーバーとの接続方式（ネイティブ実行ファイル / WASM インプロセス）は設計フェーズで決定する。

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

> **Note**: LSPサーバーとの接続方式（ネイティブ実行ファイル / WASM インプロセス）は設計フェーズで決定する。以下の受入基準は方式に依存しない形で記述する。

#### Acceptance Criteria

1. The pasta-vscode extension shall pasta_lspのビルド成果物を拡張パッケージに含める、またはパスで参照できる構成とする
2. When 拡張がアクティベートされた時、the pasta-vscode extension shall LSPサーバーを起動する
3. The pasta-vscode extension shall LSPクライアント（`vscode-languageclient`）を使用してLSPプロトコルを処理する
4. If LSPサーバーの起動に失敗した場合、the pasta-vscode extension shall ユーザーにエラー通知を表示し、TextMate文法のみのフォールバックモードで動作する

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

### Requirement 5: 診断情報の表示

**Objective:** 開発者として、`*.pasta`ファイルのパースエラーがVSCodeの問題パネルに表示されること。

#### Acceptance Criteria

1. When `*.pasta`ファイルにパースエラーが存在する時、the pasta-vscode extension shall エラーを診断情報としてVSCodeの問題パネルに表示する
2. The pasta-vscode extension shall エラー位置（行・列）を正確にハイライトする
3. When パースエラーが修正された時、the pasta-vscode extension shall 対応する診断情報をクリアする
4. While 部分パースが有効である場合、the pasta-vscode extension shall エラー箇所以外の部分について正常にハイライトを提供する

### Requirement 6: ドキュメント同期

**Objective:** 開発者として、ファイルの開閉・編集がLSPサーバーと正しく同期されること。

#### Acceptance Criteria

1. When `*.pasta`ファイルを開いた時、the pasta-vscode extension shall `textDocument/didOpen`通知をLSPサーバーに送信する
2. When `*.pasta`ファイルを編集した時、the pasta-vscode extension shall `textDocument/didChange`通知をLSPサーバーに送信する
3. When `*.pasta`ファイルを閉じた時、the pasta-vscode extension shall `textDocument/didClose`通知をLSPサーバーに送信する
4. The pasta-vscode extension shall UTF-16ベースの位置情報を正しく処理する（日本語・全角文字対応）

### Requirement 7: プロジェクト配置と構成

**Objective:** メンテナーとして、pasta-vscode拡張がpastaワークスペース内の適切な場所に配置され、ビルド・パッケージングが可能であること。

#### Acceptance Criteria

1. The pasta-vscode extension shall ワークスペースルートの`editors/vscode/`ディレクトリに配置される
2. The pasta-vscode extension shall LSPサーバーのビルド成果物を取り込む、または参照するビルドスクリプトを持つ
3. The pasta-vscode extension shall `vsce package`コマンドでVSIXファイルを生成できる
4. The pasta-vscode extension shall READMEにビルド・インストール手順を記載する
5. The pasta-vscode extension shall `.kiro/steering/structure.md`にて新規ディレクトリとして登録される
