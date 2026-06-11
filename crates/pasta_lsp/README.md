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
│  AnalysisEngine (analysis/)                     │
│  - mod.rs: AST → セマンティックトークン変換     │
│  - token_types.rs: 17 トークンタイプ / 3 モディファイア │
│  - visitors.rs: AST ビジター                    │
│  - text_utils.rs: UTF-8 → UTF-16 位置変換       │
│  - 部分パースフォールバック (parse_str_partial)  │
│  - ParseError → LSP Diagnostics 変換            │
├─────────────────────────────────────────────────┤
│  Transport (transport.rs)                        │
│  - WASM 型変換 (WasmAnalysisResult ほか)         │
│  - WASM: wasm_analyze エントリポイント           │
│  - Native: スタブ（将来拡張用）                  │
└─────────────────────────────────────────────────┘
         ↓ 依存
  pasta_dsl (PEG パーサー、AST、部分パース API)
```

## セマンティックトークンタイプ

| トークンタイプ | 対応する Pasta 構文要素             |
| -------------- | ----------------------------------- |
| comment        | コメント行 (`＃` / `#`)             |
| namespace      | グローバルシーン (`＊` / `*`)       |
| scene          | ローカルシーン (`・` / `-`)         |
| decorator      | 属性定義 (`＆` / `&`)               |
| word           | 単語定義 (`＠` / `@`)               |
| variable       | 変数参照 (`＄` / `$`)               |
| call           | Call文 (`＞` / `>`)                 |
| actor          | アクター定義 (`％` / `%`)           |
| actorName      | アクション行のアクター名            |
| codeBlock      | Lua コードブロック                  |
| talk           | 文字列リテラル / Talk テキスト      |
| sakuraScript   | さくらスクリプトタグ                |
| escape         | エスケープシーケンス                |
| operator       | コロン区切り (`：` / `:`)・代入 (`＝` / `=`)・括弧・二項演算子 |
| number         | 数値リテラル                        |
| cueMarker      | キューコマンドマーカー (`！` / `!`) |
| cueCommand     | キューコマンド名                    |

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

112 テスト（インラインテスト 22 + 統合テスト 90）:

| テストファイル              | テスト数 | 内容                                   |
| --------------------------- | -------- | -------------------------------------- |
| document インラインテスト   | 11       | document モジュール・位置変換境界      |
| transport インラインテスト  | 9        | WASM型変換・severity・シリアライズ     |
| error インラインテスト      | 2        | エラー型 Display 契約                  |
| semantic_token_test.rs      | 9        | 17 トークンタイプ識別                  |
| fullwidth_halfwidth_test.rs | 5        | 全角/半角マーカー同等認識              |
| japanese_identifier_test.rs | 5        | 日本語識別子のトークン化               |
| utf16_conversion_test.rs    | 12       | UTF-8→UTF-16 位置変換                  |
| lsp_lifecycle_test.rs       | 4        | LSP ライフサイクル統合                 |
| document_sync_test.rs       | 4        | ドキュメント同期                       |
| diagnostics_test.rs         | 6        | パースエラー診断                       |
| crash_recovery_test.rs      | 4        | パニック回復・サーバー継続             |
| partial_token_test.rs       | 5        | 部分パース→トークン提供                |
| cue_command_token_test.rs   | 10       | キューコマンドトークン生成             |
| var_set_token_test.rs       | 10       | 変数代入行のトークン生成               |
| analysis_test.rs            | 15       | コメント走査・改行正規化・解析結果     |
| analyze_robustness_test.rs  | 1        | 敵対的入力コーパスの no-panic 境界     |

## 依存関係

- `tower-lsp 0.20` - LSP フレームワーク（runtime-agnostic、lsp-types 0.94.1 再エクスポート）
- `pasta_dsl` - Pasta DSL パーサー（PEG ベース、部分パース API）
- `thiserror 2` - エラー型定義
- `serde 1` - WASM 型のシリアライズ定義
- `wasm-bindgen 0.2` (WASM ターゲット条件コンパイル)
- `serde-wasm-bindgen 0.6` - Rust→JS型変換（WASMエントリポイント）
- dev: `serde_json 1` - JSON 形状検証（テスト専用）

バージョンは workspace で一元管理。

## ライセンス

MIT OR Apache-2.0（ワークスペース共通）
