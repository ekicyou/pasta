# Implementation Tasks: pasta-vscode-extension

## Phase 1: プロジェクト基盤とTextMate文法

- [x] 1. プロジェクト構成の初期化
- [x] 1.1 (P) TypeScript + npm プロジェクトのスキャフォールド
  - `editors/vscode/` ディレクトリ配下にプロジェクト作成
  - `package.json` でVSCode拡張マニフェスト定義（`engines`, `activationEvents`, `contributes`）
  - TypeScript 設定ファイル（`tsconfig.json`）でESM出力設定
  - esbuild 設定でバンドル構成
  - _Requirements: 1.1, 1.2, 5.1_

- [x] 1.2 (P) VSCode 拡張マニフェストの設定
  - `pasta` 言語ID登録（`*.pasta` ファイル拡張子関連付け）
  - アクティベーションイベント設定（`onLanguage:pasta`）
  - カスタムセマンティックトークンタイプ・モディファイア宣言（14 types + 3 modifiers）
  - _Requirements: 1.2, 1.5, 3.1, 3.2_

- [x] 1.3 (P) TextMate文法の実装
  - `syntaxes/pasta.tmLanguage.json` ファイル作成
  - 全角/半角マーカー対応の正規表現パターン実装（文字クラス方式: `[＃#]`, `[＊*]` 等）
  - 基本構文要素のスコープマッピング（コメント、シーン、属性、単語、変数、Call/Jump、アクター）
  - Lua コードブロック認識（`` ```lua `` ... `` ``` ``、`source.lua` インクルード）
  - `package.json` で TextMate 文法ファイルを `grammars` に登録
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 1.4 拡張のアクティベーション・ライフサイクル実装
  - ExtensionActivator 実装（`activate()`, `deactivate()` エクスポート）
  - `*.pasta` ファイルオープン時の自動アクティベーション確認
  - エラー通知機能実装（`vscode.window.showErrorMessage`）
  - アクティベーション状態管理（`wasmReady`, `fallbackMode` フラグ）
  - _Requirements: 1.3, 2.2, 2.7, 4.5_

- [x] 1.5 ビルド・パッケージング環境の構築
  - npm scripts 定義（`compile`, `watch`, `package`）
  - esbuild でTypeScript バンドル処理
  - `@vscode/vsce` でVSIXパッケージ生成スクリプト
  - _Requirements: 1.4, 5.3_

- [x] 1.6* (P) Phase 1 ベースライン検証
  - TextMate文法のみでのハイライト表示テスト（全角/半角マーカー両方）
  - 各構文要素のスコープマッチング検証（`vscode-textmate` ライブラリ使用）
  - VSCode Extension Test Runner でのヘッドレステスト実行
  - _Requirements: 4.2, 4.3, 4.4, 4.5_

## Phase 2: Rust WASM エントリポイント実装

- [x] 2. WASM 公開エントリポイントの実装
- [x] 2.1 (P) `wasm_analyze` 関数の実装
  - `crates/pasta_lsp/src/transport.rs` の WASM モジュール内に `wasm_analyze(source: &str) -> JsValue` 実装
  - `AnalysisEngine::analyze()` 呼び出しをラップ
  - `catch_unwind` でパーサーパニック保護
  - `serde_wasm_bindgen::to_value()` で結果をJSオブジェクトに変換
  - _Requirements: 2.1, 2.3_

- [x] 2.2 (P) WASM 用型定義の追加
  - `WasmAnalysisResult`, `WasmSemanticToken`, `WasmDiagnostic` の Serialize 実装
  - tower-lsp 型から WASM 型への変換ロジック（フィールドコピー）
  - UTF-16 位置情報の正確性検証
  - _Requirements: 2.6_

- [x] 2.3 (P) WASM ビルドスクリプトの作成
  - `wasm-pack build --target web crates/pasta_lsp` コマンドスクリプト
  - WASM バイナリとJSバインディングを `editors/vscode/wasm/` にコピー
  - npm scripts に統合（`build:wasm`）
  - _Requirements: 5.2_

- [x] 2.4* Rust 側ユニットテストの追加
  - `wasm_analyze` の各マーカータイプ解析結果検証（`#[cfg(test)]`）
  - パニック時の空結果返却テスト
  - 不正UTF-8入力のハンドリングテスト
  - _Requirements: 2.3_

## Phase 3: TypeScript WASM ブリッジ実装

- [x] 3. WASM Bridge モジュールの実装
- [x] 3.1 WasmBridge クラスの実装
  - WASM モジュールロード機能（`WebAssembly.compile()` → `init()`）
  - `initialize(wasmUri: vscode.Uri): Promise<void>` 実装
  - `analyzeDocument(text: string): WasmAnalysisResult` 実装（例外スロー型）
  - `isReady()`, `dispose()` メソッド実装
  - _Requirements: 2.1, 2.3_

- [x] 3.2 エラーハンドリングの統合
  - WASM ロード失敗時の例外ハンドリング（ExtensionActivator での catch）
  - `analyzeDocument()` 実行エラーの例外ハンドリング（DocumentSync での catch）
  - フォールバックモード移行ロジック（`fallbackMode: true` 設定）
  - エラーログ出力（Output Channel「Pasta Language」）
  - _Requirements: 2.7, 4.5_

- [x] 3.3* WasmBridge ユニットテストの追加
  - WASM 初期化のモックテスト
  - `analyzeDocument()` 呼び出しと結果デシリアライズ検証
  - 例外スロー時の動作テスト
  - _Requirements: 2.3_

## Phase 4: セマンティックトークンとドキュメント同期

- [x] 4. ドキュメント同期とトークン提供の実装
- [x] 4.1 DocumentSync モジュールの実装
  - `onDidOpenTextDocument`, `onDidChangeTextDocument`, `onDidCloseTextDocument` イベント監視
  - デバウンス処理実装（固定 200ms、`setTimeout()` 使用）
  - `*.pasta` ファイルのフィルタリング
  - ドキュメントごとの解析直列化（同一ファイルは前回完了後に実行）
  - WasmBridge 呼び出しと例外ハンドリング（try/catch）
  - _Requirements: 2.4, 3.3, 3.4_

- [x] 4.2 SemanticTokensProvider の実装
  - `vscode.DocumentSemanticTokensProvider` インターフェース実装
  - SemanticTokensLegend 定義（14 トークンタイプ + 3 モディファイア、pasta_lsp と同一順序）
  - `provideDocumentSemanticTokens()` 実装（WasmBridge の結果から SemanticTokensBuilder でトークン構築）
  - VSCode API 登録（`vscode.languages.registerDocumentSemanticTokensProvider`）
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 4.4_

- [x] 4.3 DiagnosticsManager の実装
  - `vscode.DiagnosticCollection` 管理
  - WASM 解析結果から Diagnostic 変換
  - 問題パネルへの診断情報表示（`diagnosticCollection.set()`）
  - ドキュメントクローズ時の診断クリア
  - _Requirements: 2.5_

- [x] 4.4* セマンティックトークン統合テスト
  - 14 トークンタイプすべてのハイライト表示検証
  - 編集時のトークン再取得とハイライト更新確認
  - TextMate 文法のセマンティックハイライトによる上書き確認
  - _Requirements: 3.1, 3.2, 3.4, 4.4_

## Phase 5: 統合テストとドキュメント

- [x] 5. エンドツーエンドテストの実装
- [x] 5.1 Phase 3 (WASM統合) E2E テスト
  - WASM ロード成功時の完全ハイライト表示テスト
  - 診断情報の問題パネル表示テスト
  - 全角/半角マーカー同等認識テスト
  - UTF-16 位置情報正確性テスト（日本語・全角文字）
  - _Requirements: 2.5, 2.6, 3.1, 4.3_

- [x] 5.2 フォールバック動作テスト
  - WASM ロード失敗時の TextMate フォールバックテスト
  - エラー通知表示確認
  - 手動リロード後の再試行テスト
  - _Requirements: 2.7, 4.5_

- [x] 5.3 ビルド・パッケージング検証
  - `npm run compile` でのバンドル成功確認
  - `vsce package` での VSIX 生成テスト
  - ローカルインストールと動作確認
  - _Requirements: 1.4, 5.3_

- [x] 6. ドキュメント整備
- [x] 6.1 (P) README の作成
  - ビルド手順（WASM ビルド、TypeScript バンドル、VSIX パッケージング）
  - インストール手順（VSIX ローカルインストール）
  - 動作確認手順（`*.pasta` ファイルでのハイライト確認）
  - トラブルシューティング（WASM ロード失敗時の対処等）
  - _Requirements: 5.4_

- [x] 6.2 (P) ステアリング登録
  - `.kiro/steering/structure.md` に `editors/vscode/` ディレクトリ追加
  - プロジェクト構造の説明（npm ベース、TypeScript 拡張、WASM 統合）
  - _Requirements: 5.5_

- [x] 6.3 ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認（日本語フレンドリー、UNICODE識別子、宣言的フロー）
  - doc/spec/ - 言語仕様への影響確認（該当なし）
  - GRAMMAR.md - 文法リファレンスの同期確認（該当なし）
  - TEST_COVERAGE.md - 新規テストのマッピング追加（Phase 1/3 E2E テスト、TextMate 文法テスト）
  - クレート README - pasta_lsp の WASM エントリポイント追加を反映
  - steering/* - 該当領域のステアリング更新（structure.md 更新済み）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5_

## タスク概要

- **Phase 1**: プロジェクト基盤とTextMate文法（6タスク）
- **Phase 2**: Rust WASM エントリポイント（4タスク）
- **Phase 3**: TypeScript WASM ブリッジ（3タスク）
- **Phase 4**: セマンティックトークンとドキュメント同期（4タスク）
- **Phase 5**: 統合テストとドキュメント（6タスク）

**合計**: 23タスク（メジャータスク: 6、サブタスク: 17）
**全要件カバー**: 28 Acceptance Criteria すべてマッピング済み

### 並列実行可能タスク

`(P)` マーカーがついたタスクは依存関係がなく、並列実行可能：
- 1.1, 1.2, 1.3 (Phase 1 基盤)
- 1.6, 2.1, 2.2, 2.3 (Phase 1/2 検証と WASM)
- 6.1, 6.2 (ドキュメント)

### オプショナルタスク

`*` マーカーがついたタスクは、MVP 後に延期可能なテストカバレッジ強化タスク：
- 1.6, 2.4, 3.3, 4.4 (ユニット/統合テスト)
