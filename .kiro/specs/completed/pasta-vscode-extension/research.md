# Research & Design Decisions

## Summary
- **Feature**: `pasta-vscode-extension`
- **Discovery Scope**: Complex Integration（WASM + VSCode拡張 + 既存Rust LSP統合）
- **Key Findings**:
  1. `AnalysisEngine::analyze()` は純粋関数であり、tower-lspを介さずwasm-bindgenで直接公開可能
  2. Microsoft公式の `@vscode/wasm-wasi-lsp` パターンが存在するが、WASI Preview1 + threads ターゲットが必要
  3. tower-lspをWASMで動作させた公開事例はゼロ。`runtime-agnostic`フラグがあるが非同期ランタイム問題は未解決

## Research Log

### tower-lsp のWASM互換性調査

- **Context**: 要件書 Req 2 でWASMインプロセスLSP統合がゴール。既存のpasta_lspはtower-lsp 0.20を使用
- **Sources Consulted**:
  - https://docs.rs/tower-lsp/0.20.0/tower_lsp/
  - https://github.com/ebkalderon/tower-lsp/blob/master/src/service.rs
- **Findings**:
  - `LspService` は `tower::Service<Request>` を実装しており、`Server`を介さず直接呼び出し可能
  - リクエスト型: `tower_lsp::jsonrpc::Request`、レスポンス型: `Option<tower_lsp::jsonrpc::Response>`
  - テストコードで `service.ready().await.unwrap().call(request).await` の直接呼び出しパターンが確認済み
  - `Server` ドキュメントにWASMへの言及あり（"exotic targets such as WASM"）
  - **しかし**: tower-lspの内部依存が非同期ランタイム（tokio features）を要求。WASMシングルスレッド環境では `tokio::spawn` 等が動作しない
  - **公開実績**: tower-lspをWASM上で動作させた公開事例は**ゼロ**
- **Implications**: tower-lspをWASMで直接使う方式は技術的リスクが極めて高い

### vscode-languageclient カスタムトランスポート

- **Context**: WASMモジュールとLSPクライアント間の通信方式を調査
- **Sources Consulted**:
  - https://github.com/microsoft/vscode-languageserver-node/blob/main/client/src/common/client.ts
- **Findings**:
  - `ServerOptions` は `() => Promise<MessageTransports>` 関数型をサポート
  - `MessageTransports` = `{ reader: MessageReader, writer: MessageWriter }`
  - プロセススポーン**不要**。任意のReader/Writerペアでカスタムトランスポートが構築可能
  - `LanguageClient` のコンストラクタに関数を渡せば、完全にインプロセスで動作可能
- **Implications**: LSPプロトコルレベルでの接続は柔軟。問題はサーバー側のWASM互換性

### Microsoft公式 WASM LSPサンプル（@vscode/wasm-wasi）

- **Context**: 公式のWASM LSP統合パターンを調査
- **Sources Consulted**:
  - https://github.com/microsoft/vscode-wasm (`testbeds/lsp-rust`)
  - https://www.npmjs.com/package/@vscode/wasm-wasi
  - https://www.npmjs.com/package/@vscode/wasm-wasi-lsp
- **Findings**:
  - Microsoft公式サンプルが存在。Rustで書いたLSPサーバーをWASMとしてVSCode内で実行
  - `wasm32-wasi-preview1-threads` ターゲットを使用（`wasm32-unknown-unknown`ではない）
  - `lsp-server` クレート（同期型）を使用。tower-lsp（非同期型）は**使用していない**
  - `@vscode/wasm-wasi` が WASI ランタイムを提供し、stdio パイプで LSP 通信
  - `@vscode/wasm-wasi-lsp` が `MessageTransports` ラッパーを提供
  - `ms-vscode.wasm-wasi-core` 拡張への依存が必要
- **Implications**: 公式パターンに従うなら `lsp-server`クレートへの移行が必要。tower-lspからの乖離が大きい

### AnalysisEngine直接公開アプローチ

- **Context**: tower-lspのWASM問題を回避する代替案を調査
- **Sources Consulted**:
  - `crates/pasta_lsp/src/analysis.rs` （既存コード分析）
  - https://rustwasm.github.io/docs/wasm-bindgen/
- **Findings**:
  - `AnalysisEngine::analyze(source: &str) -> AnalysisResult` は**純粋関数**
  - 入力: テキスト文字列。出力: `AnalysisResult { tokens: Vec<SemanticToken>, diagnostics: Vec<Diagnostic> }`
  - tower-lspの型（`SemanticToken`, `Diagnostic`）に依存しているが、serde_json経由でシリアライズ可能
  - `catch_unwind` によるクラッシュ保護は `server.rs` 側で実装済み（`analyze_and_publish` メソッド内）
  - `wasm-bindgen` + `serde-wasm-bindgen` で JS側に直接公開可能
  - TypeScript側で `vscode.languages.registerDocumentSemanticTokensProvider` / `vscode.languages.createDiagnosticCollection` を使って VSCode API に直接接続
- **Implications**: tower-lsp依存を完全に排除でき、バイナリサイズも小さくなる。最も低リスクなアプローチ

### TextMate文法のPasta DSL適用可能性

- **Context**: Req 4 のTextMate文法フォールバック実現性を調査
- **Sources Consulted**:
  - `crates/pasta_dsl/src/parser/grammar.pest`（PEG文法定義）
  - `.kiro/steering/grammar.md`（マーカー一覧）
- **Findings**:
  - Pasta DSLは**行指向文法**であり、各行はマーカー文字で始まる
  - TextMate文法の正規表現ベースのパターンマッチングと相性が良い
  - 全角/半角マーカーは `[＊*]`、`[＃#]` のような文字クラスで対応可能
  - Luaコードブロックは ` ```lua` ... ` ``` ` の `begin`/`end` パターンで対応可能
  - 制限: TextMateでは文脈依存の解析（例: アクション行内のインライン要素）は困難
- **Implications**: Phase 1（TextMate文法のみ）で基本ハイライトは十分実現可能

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: tower-lsp on WASM | tower-lsp LspServiceをWASM上で直接実行 | LSPプロトコル完全準拠、既存コードの再利用最大 | 非同期ランタイム問題、公開事例ゼロ、バイナリサイズ大 | **非推奨** |
| B: @vscode/wasm-wasi | WASI ランタイム上でLSPサーバーを実行 | 公式サンプルあり、stdio標準パイプ | lsp-serverクレートへの移行必要、wasm-wasi-core依存 | 要追加調査 |
| C: AnalysisEngine直接公開 | wasm-bindgenでAnalysisEngineを直接JS公開 | 最低リスク、最小バイナリ、既存コード最大活用 | LSPプロトコル非準拠（VSCode API直接使用） | **推奨** |

## Design Decisions

### Decision: WASM統合方式としてAnalysisEngine直接公開を採用

- **Context**: 要件書 Req 2 はWASMインプロセスLSP統合をゴールとしている。Gap分析でWASMトランスポート実装のリスクがHighと評価された
- **Alternatives Considered**:
  1. Option A: tower-lsp LspServiceをWASM上で直接実行 — 非同期ランタイム問題で非現実的
  2. Option B: @vscode/wasm-wasi パターン — lsp-serverクレートへの書き換え必要、wasm-wasi-core拡張依存
  3. Option C: AnalysisEngine::analyze()をwasm-bindgenで直接公開 — 最低リスク、既存コード最大活用
- **Selected Approach**: Option C — AnalysisEngine直接公開
- **Rationale**:
  - `AnalysisEngine::analyze()` は純粋関数であり、WASMに最適
  - tower-lspの非同期ランタイム依存を完全に回避
  - TypeScript側でVSCode APIに直接接続するため、LSPプロトコルのオーバーヘッドなし
  - バイナリサイズが最小（tower-lsp + tokio依存を排除可能）
  - 既存の14セマンティックトークンタイプと診断情報がそのまま利用可能
- **Trade-offs**:
  - ✅ 実装リスク最小、デバッグ容易、バイナリサイズ小
  - ❌ LSPプロトコル準拠ではない（ただし機能的には同等）
  - ❌ 将来的にLSP機能追加（補完、ホバー等）時にTypeScript側の実装が増える
- **Follow-up**: WASM公開用のCargo.tomlフィーチャーフラグ設計、tower-lsp型からの依存分離方法

### Decision: Phase 2（ネイティブLSP）は実装しない

- **Context**: Phase 2は任意実装とされている。AnalysisEngine直接公開方式ではネイティブLSPを経由しないため不要
- **Selected Approach**: Phase 2をスキップし、Phase 1（TextMate） → Phase 3（WASM直接公開）の2段階で実装
- **Rationale**: AnalysisEngine直接公開方式により、tower-lspのstdioトランスポートは不要。直接WASM統合でLSP機能（セマンティックトークン、診断）を実現
- **Trade-offs**: ネイティブバイナリでのデバッグ手段がなくなるが、TypeScript側でのデバッグが容易なため問題なし

### Decision: tower-lsp型への依存を分離する

- **Context**: AnalysisEngine直接公開では、tower-lspの`SemanticToken`/`Diagnostic`型をWASMバイナリに含める必要がある
- **Selected Approach**: 
  - Phase 1ではtower-lsp依存を含んだままビルド（既存コード変更最小化）
  - 将来的にanalysis.rs内の型を独自型に置き換え、tower-lsp依存をオプショナルにする
- **Rationale**: 現段階ではtower-lsp型をJSON経由でシリアライズして橋渡しする方が実装が速い
- **Trade-offs**: WASMバイナリサイズが若干大きくなる（tower-lsp型の定義を含むため）

## Risks & Mitigations
- **tower-lsp型のserde互換性**: SemanticToken/Diagnosticがserdeでシリアライズ可能か確認必要 → serde_jsonで確認済み（lsp-types crateはSerialize/Deserialize derive済み）
- **WASMバイナリサイズ**: tower-lsp依存を含むと膨張する可能性 → wasm-opt + LTO + strip で最適化
- **catch_unwindのWASM動作**: std::panic::catch_unwindがwasm32ターゲットで動作するか → Rustのpanicをabortに設定した場合は動作しない。panicをunwindに設定する必要あり
- **全角/半角正規表現のTextMate互換性**: oniguruma（TextMateの正規表現エンジン）でUnicode文字クラスが使えるか → 使用可能（`\p{Fullwidth_Forms}` または直接文字指定 `[＊*]`）

## References
- [tower-lsp docs.rs](https://docs.rs/tower-lsp/0.20.0/tower_lsp/) — LspService API
- [tower-lsp GitHub](https://github.com/ebkalderon/tower-lsp) — ソースコード、テストパターン
- [vscode-wasm GitHub](https://github.com/microsoft/vscode-wasm) — 公式WASM LSPサンプル
- [@vscode/wasm-wasi npm](https://www.npmjs.com/package/@vscode/wasm-wasi) — WASIランタイム
- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/) — wasm-bindgen参照
- [vscode-languageserver-node](https://github.com/microsoft/vscode-languageserver-node) — カスタムMessageTransports
- [lsp-types crate](https://docs.rs/lsp-types/0.94.1/lsp_types/) — SemanticToken/Diagnostic型定義
