# Pasta VSCode Extension

Pasta DSL（`*.pasta` ファイル）のシンタックスハイライトと診断情報を提供する VSCode 拡張です。

## 機能

- **TextMate 文法ハイライト**: 全角/半角マーカー両対応の構文ハイライト
- **セマンティックトークン**: pasta_lsp (WASM) による14種のセマンティックトークン提供
- **診断情報**: パースエラーの問題パネル表示
- **フォールバック**: WASM ロード失敗時は TextMate 文法のみで動作

## アーキテクチャ

```
┌──────────────────────────────────────────┐
│  VSCode Extension (TypeScript)           │
│  ├── extension.ts    - アクティベーション│
│  ├── wasmBridge.ts   - WASM ブリッジ     │
│  ├── semanticTokensProvider.ts           │
│  ├── diagnosticsManager.ts              │
│  └── documentSync.ts - 200ms デバウンス  │
├──────────────────────────────────────────┤
│  TextMate Grammar (tmLanguage.json)      │
│  全角/半角マーカー対応の正規表現         │
├──────────────────────────────────────────┤
│  pasta_lsp.wasm (Rust → WebAssembly)     │
│  AnalysisEngine: 14 token types          │
│  catch_unwind パニック保護               │
└──────────────────────────────────────────┘
```

## ビルド手順

### 前提条件

- Node.js 18+
- npm 9+
- Rust ツールチェイン (stable)
- `wasm-pack` (`cargo install wasm-pack`)

### 1. WASM ビルド

```powershell
# PowerShell (Windows)
npm run build:wasm

# または手動:
wasm-pack build ../../crates/pasta_lsp --target web --out-dir ../../editors/vscode/wasm
```

### 2. TypeScript コンパイル

```bash
cd editors/vscode
npm install
npm run compile
```

### 3. VSIX パッケージング

```bash
npm run package
```

`pasta-vscode-0.1.0.vsix` が生成されます。

## インストール

### ローカルインストール

```bash
code --install-extension pasta-vscode-0.1.0.vsix
```

### 開発モード

1. VSCode で `editors/vscode/` フォルダを開く
2. `F5` で Extension Development Host を起動
3. `*.pasta` ファイルを開いてハイライトを確認

## テスト

```bash
# 全テスト実行
npm test

# 個別実行
npm run test:grammar   # TextMate 文法テスト (20 tests)
npm run test:unit      # ユニットテスト (29 tests)
npm run test:e2e       # E2E/ビルド検証テスト (30 tests)
```

## トラブルシューティング

### WASM ロードに失敗する

- エラーメッセージ: `Pasta WASM initialization failed: ...`
- 原因: `wasm/` ディレクトリに WASM バイナリが配置されていない
- 対処: `npm run build:wasm` を実行して WASM をビルド
- フォールバック: TextMate 文法による基本ハイライトは WASM なしでも動作します

### ハイライトが表示されない

1. ファイル拡張子が `.pasta` であることを確認
2. VSCode 右下のステータスバーで言語が「Pasta DSL」になっていることを確認
3. Output パネル → 「Pasta Language」チャンネルでログを確認

### セマンティックハイライトが動作しない

- WASM が正常にロードされているか Output パネルで確認
- `WASM bridge initialized successfully.` と表示されていれば正常
- 表示されない場合はフォールバックモード（TextMate のみ）で動作中

## セマンティックトークンタイプ

| # | タイプ       | Pasta 構文要素                  |
|---|-------------|--------------------------------|
| 0 | comment     | コメント行 (`＃` / `#`)        |
| 1 | namespace   | グローバルシーン (`＊` / `*`)  |
| 2 | scene       | ローカルシーン (`・` / `-`)    |
| 3 | decorator   | 属性定義 (`＆` / `&`)         |
| 4 | word        | 単語定義 (`＠` / `@`)         |
| 5 | variable    | 変数参照 (`＄` / `$`)         |
| 6 | call        | Call文 (`＞` / `>`)           |
| 7 | actor       | アクター定義 (`％` / `%`)     |
| 8 | actorName   | アクション行のアクター名       |
| 9 | codeBlock   | Lua コードブロック             |
| 10| string      | Talk テキスト                  |
| 11| sakuraScript| さくらスクリプトタグ           |
| 12| escape      | エスケープシーケンス           |
| 13| operator    | コロン区切り (`：` / `:`)     |

## ライセンス

MIT
