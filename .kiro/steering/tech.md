# Technical Steering

## 技術スタック

### 言語・ランタイム
- **Rust 2024 edition**: メインコンパイラ言語
- **Rune 0.14**: バックエンドスクリプトVM（generator機能を使用）
- **Pest 2.8**: PEGパーサー生成器（`pasta.pest`文法定義）

### 主要依存関係
- **thiserror 2**: エラー型定義
- **glob 0.3**: ファイルパターンマッチング（スクリプト読み込み）
- **tracing 0.1**: ロギング・診断
- **rand 0.9**: ランダム選択（重複シーン、前方一致候補）
- **futures 0.3**: 非同期処理サポート
- **toml 0.9.8**: 設定ファイル管理
- **fast_radix_trie 1.1.0**: 前方一致シーン検索

### 開発環境
- **tempfile 3**: テスト用一時ファイル生成

## アーキテクチャ原則

### レイヤー構成
```
Engine (上位API) → Cache/Loader
    ↓
Transpiler (2pass) ← Parser (Pest)
    ↓
Runtime (Rune VM) → IR Output (ScriptEvent)
```

| レイヤー | 責務 |
|---------|------|
| Parser | DSL→AST変換 |
| Transpiler | AST→Runeコード、シーン管理 |
| Runtime | Rune VM実行、yield出力 |
| Engine | 統合API、キャッシュ |
| IR | ScriptEventイベント出力 |

### 設計哲学

| 原則 | 内容 |
|------|------|
| UI独立性 | Wait/Syncはマーカーのみ、areka側で制御 |
| 宣言的フロー | Call/Jumpで制御、if/while/forなし |
| Yield型 | 全出力はyield、Generator継続 |
| 2パス変換 | Pass1: シーン登録、Pass2: コード生成 |

**結果**: 完全なユニットテスト可能性を実現

## コーディング規約

| 項目 | 規約 |
|------|------|
| テストファイル | `<feature>_test.rs` |
| Rust識別子 | スネークケース |
| DSL識別子 | 日本語/UNICODE可 |
| エラー型 | `Result<T, PastaError>` |
| ドキュメント | `///`で公開API |

### テスト戦略
- ユニット: レイヤー独立
- 統合: `tests/`配下
- Fixture: `tests/fixtures/*.pasta`
- Doctest: API例をドキュメント内に

## 品質基準

| 項目 | 基準 |
|------|------|
| テスト | 新機能必須、リグレッション防止 |
| キャッシュ | パース結果をメモリ保持 |
| 検索性能 | シーンO(1)、前方一致Radix Trie |
| セキュリティ | Rune VMサンドボックス依存 |

## 依存関係管理

### バージョン戦略
- セマンティックバージョニング準拠
- Rune 0.14に固定（破壊的変更対応まで）
- 依存ライブラリ: メジャーバージョン指定

### ライセンス
- **MIT OR Apache-2.0**: デュアルライセンス
- 依存関係ライセンス: 互換性確認済み

### 公開ポリシー
- `publish = true` in Cargo.toml
- crates.io公開予定
- API安定化後にリリース

## デプロイメント

```bash
cargo test --all-features  # 全テスト
cargo test --release       # リリースビルド
```

### 将来計画
- CI/CD: GitHub Actions
- SHIORI.DLL: C FFIラッパー
- areka統合: 動的リンク、MCP Server
