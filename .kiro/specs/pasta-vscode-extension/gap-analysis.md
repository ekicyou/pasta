# Gap Analysis Report: pasta-vscode-extension

## 概要

本レポートは、`*.pasta` 構文のシンタックスハイライトを提供するVSCode拡張の実装に向けて、既存コードベースと要件とのギャップを分析した結果をまとめたものである。

---

## 1. 現状調査

### 1.1 既存アセットの棚卸し

| 資産 | 状態 | 詳細 |
|------|------|------|
| **pasta_lsp コアロジック** | ✅ 完成 | PastaLangServer (tower-lsp trait実装)、14セマンティックトークン、3モディファイア、部分パース対応、クラッシュ保護 |
| **analysis.rs** | ✅ 完成 | AST→セマンティックトークン変換、UTF-8→UTF-16位置変換、ParseError→Diagnostics変換（594行） |
| **document.rs** | ✅ 完成 | DocumentManager（open/change/close）、増分更新対応、UTF-16列→バイト変換（142行） |
| **transport.rs (WASM)** | ⚠️ スタブ | `WasmLspServer` 構造体あり。`send()`/`on_message()` がTODO状態 |
| **transport.rs (Native)** | ❌ 空 | スタブのみ |
| **WASMビルド** | ✅ ビルド済 | `pasta_lsp.wasm` (0.19 MB)、wasm32-unknown-unknown/release に存在 |
| **Cargo.toml WASM依存** | ✅ 設定済 | wasm-bindgen 0.2, wasm-bindgen-futures 0.4, js-sys 0.3（条件コンパイル） |
| **tower-lsp** | ✅ WASM互換 | runtime-agnostic フィーチャーで構成済み |
| **editors/ ディレクトリ** | ❌ 未作成 | 新規作成が必要 |
| **TextMate文法** | ❌ 未作成 | pasta.pest (PEG) から派生作成が必要 |
| **テスト** | ✅ 72件パス | 統合テスト60 + インラインテスト12 |

### 1.2 既存パターン・規約

- **ワークスペース構成**: Pure Virtual Workspace（ルートに[package]なし）
- **依存方向**: pasta_dsl ← pasta_lsp（パーサー依存のみ、pasta_core不要）
- **テスト配置**: `crates/<crate>/tests/<feature>_test.rs`
- **エラー型**: thiserror 2ベース
- **ドキュメント**: 各クレートに README.md

### 1.3 参考リポジトリ分析（tower-lsp-boilerplate）

**重要な差異**:

| 項目 | tower-lsp-boilerplate | pasta-vscode |
|------|----------------------|--------------|
| LSP実行方式 | **ネイティブ実行ファイル (stdio)** | **WASM (ブラウザ内実行)** |
| クライアント接続 | `Executable` → `ServerOptions` | WASM ロード → JS バインディング |
| パッケージング | cargo build → PATH に配置 | wasm-bindgen → 拡張にバンドル |
| ビルドツール | esbuild | esbuild（同等構成可能） |
| 拡張構造 | `client/src/extension.ts` | `src/extension.ts`（類似構造） |

**活用可能な要素**:
- esbuild による TypeScript バンドル構成
- `vscode-languageclient` の LanguageClient パターン
- package.json マニフェスト構造（contributes, activationEvents）
- デバッグ構成（launch.json / F5 起動）

**活用不可能な要素**:
- `Executable` ベースの ServerOptions（WASM では使用不可）
- stdio ベースのトランスポート
- `DashMap` / `Rope` の使用（pasta_lsp は独自の DocumentManager を使用）

---

## 2. 要件実現性分析

### 要件→アセット マッピング

| 要件 | 既存アセット | ギャップ | 状態 |
|------|-------------|---------|------|
| **Req 1: 拡張基盤** | なし | package.json, tsconfig, esbuild 全体 | **Missing** |
| **Req 2: WASM LSP統合** | transport.rs (スタブ), pasta_lsp.wasm | WASMトランスポート実装、wasm-bindgen JS生成、LSPクライアント接続 | **Missing (部分的)** |
| **Req 3: セマンティックハイライト** | analysis.rs (完成) | VSCode側のセマンティックトークンマッピング設定 | **Missing (軽微)** |
| **Req 4: TextMate文法** | pasta.pest (PEG文法) | .tmLanguage.json への変換、全角/半角の正規表現対応 | **Missing** |
| **Req 5: 診断情報** | server.rs (publish_diagnostics 完成) | VSCode側での自動表示はLSPクライアントが処理 | **既存で対応可** |
| **Req 6: ドキュメント同期** | document.rs (完成) | LSPクライアントが自動処理 | **既存で対応可** |
| **Req 7: プロジェクト配置** | なし | editors/vscode/ 作成、ビルドスクリプト、steering更新 | **Missing** |

### 技術的未知数 (Research Needed)

1. **WASM LSPトランスポート**: tower-lspのWASMランタイムでの動作確認。`LspService::build()` + カスタムトランスポート（非stdio）でのメッセージルーティングパターン
2. **wasm-bindgen 出力形式**: `--target web` vs `--target bundler` vs `--target nodejs` の選択
3. **VSCode Language Client WASM対応**: `vscode-languageclient` はネイティブプロセス前提。WASM LSPへの接続にはカスタムトランスポートが必要（`TransportKind` の制約）
4. **WASMバイナリサイズ最適化**: 現在0.19MB。wasm-opt、LTO設定の最適化余地

---

## 3. 実装アプローチの選択肢

### Option A: ネイティブ実行ファイル方式（tower-lsp-boilerplate 準拠）

**方針**: pasta_lsp にネイティブ stdio トランスポートを実装し、実行ファイルとして起動

- LSPサーバーを `cargo build --release` で実行ファイルとしてビルド
- VSCode拡張から `Executable` として起動（boilerplateと同じパターン）
- WASMビルドは不要

**Trade-offs**:
- ✅ 実装が最も簡単（参考リポジトリをほぼそのまま適用可能）
- ✅ stdio 標準プロトコルで信頼性が高い
- ✅ デバッグが容易
- ❌ ユーザーに Rust ツールチェーンまたはプリビルドバイナリが必要
- ❌ クロスプラットフォーム配布が複雑（Windows/macOS/Linux バイナリ）
- ❌ WASM 前提の既存設計との不整合

### Option B: WASM インプロセス方式（要件書の方針）

**方針**: pasta_lsp.wasm を拡張内で直接ロードし、JS/TSレイヤーでLSPプロトコルをブリッジ

- `wasm-bindgen` でJSバインディング生成
- TypeScript側でLSPメッセージのシリアライズ/デシリアライズを処理
- `vscode-languageclient` のカスタムトランスポートまたは独自実装

**Trade-offs**:
- ✅ 依存なし（WASM は拡張にバンドル）
- ✅ クロスプラットフォーム問題なし
- ✅ pasta_lsp の設計意図と合致
- ❌ WASMトランスポート層の実装が複雑（Rust側 + JS側）
- ❌ tower-lsp の WASM ランタイムでの実績が限られている
- ❌ デバッグが困難

### Option C: ハイブリッド方式（推奨）

**方針**: 段階的に実装。まず TextMate 文法のみで最低限のハイライトを提供し、次にネイティブ LSP、最終的に WASM へ移行

**Phase 1**: TextMate文法 + 言語登録のみ（LSPなし）
- `.tmLanguage.json` による基本ハイライト
- `language-configuration.json` によるブラケットマッチ等
- セマンティックハイライトなしで即座に動作確認可能

**Phase 2**: ネイティブ LSP 統合
- pasta_lsp に stdio トランスポートを追加
- `vscode-languageclient/node` で接続
- セマンティックハイライト + 診断が有効化

**Phase 3**: WASM 移行（将来）
- ネイティブLSPの動作確認後、WASMトランスポートに置換
- ユーザー依存をゼロにする最終形態

**Trade-offs**:
- ✅ 段階的リスク軽減（各段階で動作確認可能）
- ✅ TextMate文法は WASM 移行後もフォールバックとして残る
- ✅ ネイティブ LSP は参考リポジトリのパターンを直接活用可能
- ❌ 最終的に3段階の実装が必要
- ❌ WASMトランスポート実装は Phase 3 まで先送り

---

## 4. 複雑性とリスク評価

### 工数見積り

| アプローチ | 工数 | 理由 |
|-----------|------|------|
| Option A (ネイティブ) | **M (3-7日)** | 参考リポジトリのパターン活用可。stdio追加 + 拡張スキャフォールド |
| Option B (WASM) | **L (1-2週)** | WASMトランスポート実装に技術的不確実性あり |
| Option C (ハイブリッド) | Phase1: **S (1-3日)**, Phase2: **M (3-7日)** | 段階的に達成、各フェーズで成果物あり |

### リスク評価

| リスク項目 | レベル | 説明 |
|-----------|--------|------|
| TextMate文法の全角/半角対応 | **Low** | 正規表現で対応可能、pasta.pest の定義が明確 |
| ネイティブ LSP 統合 | **Low** | tower-lsp + stdio は確立されたパターン |
| WASM トランスポート実装 | **High** | tower-lsp のWASM動作実績が限定的、カスタムトランスポート設計が必要 |
| wasm-bindgen JS バインディング | **Medium** | 出力形式の選択（web/bundler/nodejs）に依存 |
| VSCode Language Client WASM対応 | **High** | 標準LCはプロセス起動前提、WASM接続にはカスタム実装が必要 |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option C（ハイブリッド）**

**理由**:
1. Phase 1 の TextMate 文法のみで「実際にVSCodeで確認できる」要件を最速で達成
2. Phase 2 のネイティブ LSP で セマンティックハイライト + 診断を実現
3. WASM 移行は技術的不確実性が高いため、動作実績を積んでから取り組むのが安全
4. 要件書の全要件（Req 1-7）を段階的にカバー可能

### 設計フェーズでの要調査事項

1. **TextMate文法設計**: pasta.pest から `.tmLanguage.json` への変換ルール策定
2. **ネイティブトランスポート設計**: pasta_lsp に `main()` + stdio を追加する方法
3. **WASM実現性調査**: tower-lsp + wasm-bindgen + カスタムトランスポートの PoC
4. **esbuild構成**: TypeScript バンドル + WASM バイナリの拡張パッケージング
5. **配置構造**: `editors/vscode/` のディレクトリレイアウト決定

### 要件書への修正提案

- **Req 2 (WASM LSP統合)** の受入基準は Phase 2/3 で段階的に達成することを明記
- **Req 4 (TextMate文法)** を Phase 1 の主要成果物として優先度を引き上げ
- Phase 1 完了時点で「実際にVSCodeで確認できる」を達成可能であることを要件に反映
