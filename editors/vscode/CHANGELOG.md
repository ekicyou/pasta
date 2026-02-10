# Changelog

All notable changes to the "Pasta DSL" extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.4] - 2026-02-10

### Fixed

- Marketplace ページでスクリーンショット画像が表示されない問題を修正（相対パス → GitHub raw URL）

## [0.1.3] - 2026-02-10

### Added

- TextMate 文法によるシンタックスハイライト（全角/半角マーカー両対応）
- WASM ベース LSP によるセマンティックトークン（14種類）
- パースエラーの診断情報表示（Problems パネル）
- WASM ロード失敗時の TextMate フォールバック動作
- ドキュメント変更時の 200ms デバウンス同期
- Pasta DSL ファイル（*.pasta）の言語登録
