# pasta_lsp

Pasta DSL の Language Server Protocol (LSP) 実装クレート。

`*.pasta` ファイルに対するセマンティックハイライト、診断情報、ドキュメント同期を提供します。VSCode 拡張への WASM モジュール組み込みを前提に設計されています。

## アーキテクチャ

```
┌─────────────────────────────────────────────────┐
│  PastaLangServer (server.rs)                    │
│  tower-lsp LanguageServer trait 実装            │
│  - initialize / didOpen / didChange / didClose  │
│  - semanticTokens/full                          │
│  - std::panic::catch_unwind クラッシュ保護      │
├─────────────────────────────────────────────────┤
│  DocumentManager (document.rs)                  │
│  - ドキュメント状態管理 (open/close/change)     │
│  - 増分・全体テキスト更新                       │
│  - UNICODE バイトオフセット計算                 │
├─────────────────────────────────────────────────┤
│  AnalysisEngine (analysis.rs)                   │
│  - AST → セマンティックトークン変換             │
│  - 14 トークンタイプ / 3 モディファイア          │
│  - UTF-8 → UTF-16 位置変換                      │
│  - 部分パースフォールバック (parse_str_partial)  │
│  - ParseError → LSP Diagnostics 変換            │
├─────────────────────────────────────────────────┤
│  TransportBridge (transport.rs)                  │
│  - WASM: wasm-bindgen エントリポイント           │
│  - Native: スタブ（将来拡張用）                  │
└─────────────────────────────────────────────────┘
         ↓ 依存
  pasta_dsl (PEG パーサー、AST、部分パース API)
```

## セマンティックトークンタイプ

| トークンタイプ | 対応する Pasta 構文要素     |
| -------------- | --------------------------- |
| comment        | コメント行 (`＃` / `#`)     |
| namespace      | グローバルシーン (`＊` / `*`) |
| scene          | ローカルシーン (`・` / `-`)  |
| decorator      | 属性定義 (`＆` / `&`)       |
| word           | 単語定義 (`＠` / `@`)       |
| variable       | 変数参照 (`＄` / `$`)       |
| call           | Call文 (`＞` / `>`)          |
| actor          | アクター定義 (`％` / `%`)   |
| actorName      | アクション行のアクター名    |
| codeBlock      | Lua コードブロック          |
| string         | 文字列リテラル / Talk テキスト |
| sakuraScript   | さくらスクリプトタグ         |
| escape         | エスケープシーケンス         |
| operator       | コロン区切り (`：` / `:`)    |

## 公開 API

```rust
// サーバー
pub struct PastaLangServer { /* ... */ }

// エラー型
pub enum LangServerError {
    Parse(String),
    DocumentNotFound(String),
    Internal(String),
}
```

## ビルド

### ネイティブ

```sh
cargo build -p pasta_lsp
cargo test -p pasta_lsp
```

### WASM

```sh
rustup target add wasm32-unknown-unknown
cargo build -p pasta_lsp --target wasm32-unknown-unknown --release
```

## テスト

72 テスト（インラインテスト 12 + 統合テスト 60）:

| テストファイル                 | テスト数 | 内容                              |
| ------------------------------ | -------- | --------------------------------- |
| インラインテスト               | 12       | analysis / document モジュール    |
| semantic_token_test.rs         | 9        | 14 トークンタイプ識別             |
| fullwidth_halfwidth_test.rs    | 5        | 全角/半角マーカー同等認識         |
| japanese_identifier_test.rs    | 5        | 日本語識別子のトークン化          |
| utf16_conversion_test.rs       | 12       | UTF-8→UTF-16 位置変換             |
| lsp_lifecycle_test.rs          | 4        | LSP ライフサイクル統合            |
| document_sync_test.rs          | 4        | ドキュメント同期                  |
| diagnostics_test.rs            | 6        | パースエラー診断                  |
| crash_recovery_test.rs         | 4        | パニック回復・サーバー継続        |
| partial_token_test.rs          | 5        | 部分パース→トークン提供           |

## 依存関係

- `tower-lsp 0.20` - LSP フレームワーク（runtime-agnostic、lsp-types 0.94.1 再エクスポート）
- `pasta_dsl` - Pasta DSL パーサー（PEG ベース、部分パース API）
- `thiserror 2` - エラー型定義
- `serde` / `serde_json` - シリアライズ
- `wasm-bindgen` / `wasm-bindgen-futures` (WASM ターゲット条件コンパイル)

## ライセンス

MIT OR Apache-2.0（ワークスペース共通）
