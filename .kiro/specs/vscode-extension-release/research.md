# Research & Design Decisions: vscode-extension-release

## Summary
- **Feature**: `vscode-extension-release`
- **Discovery Scope**: Extension（既存 `release-workflow` への拡張 + ドキュメント整備）
- **Key Findings**:
  - release-workflow の Phase 3（crates.io 公開）と Phase 4（ゴーストビルド）の間に VSCode 拡張ステップを挿入する設計が最適
  - SVG アイコンは Marketplace が対応しているため変換不要（ただし vsce が警告を出す可能性あり）
  - VSIX ファイル名は vsce が自動生成する（`<name>-<version>.vsix`）

## Research Log

### VSCode Marketplace 公開要件
- **Context**: Marketplace に公開するために package.json に必要なメタデータを調査
- **Sources Consulted**: VSCode Publishing Extensions ドキュメント、vsce CLI ヘルプ
- **Findings**:
  - 必須: `name`, `displayName`, `description`, `version`, `publisher`, `engines.vscode`
  - 推奨: `icon`, `categories`, `keywords`, `repository`, `homepage`, `bugs`
  - `icon` は 128x128px 以上推奨。SVG 対応済み
  - `galleryBanner` は不要（要件協議で除外済み）
  - `badges` は不要（要件協議で除外済み）
- **Implications**: 現在の package.json に `icon`, `keywords`, `homepage`, `bugs` を追加すれば公開可能

### release-workflow との統合ポイント
- **Context**: 既存 release-workflow のどのフェーズに VSCode 拡張公開を挿入するか
- **Sources Consulted**: `.kiro/specs/release-workflow/design.md`, `tasks.md`
- **Findings**:
  - 既存パイプライン: Phase 0→1→2→3(crates.io)→4(ゴーストビルド)→5(タグ)→6(GitHub Release)
  - VSCode 拡張は crates.io 公開後に WASM ビルドが必要（pasta_lsp が crates には含まれないが、同じソースコードを使用）
  - Phase 3 と Phase 4 の間に「Phase 3.5: VSCode Extension」を挿入するのが自然
  - Phase 2 に package.json バージョン更新を追加
  - Phase 6 に VSIX アセット添付を追加
- **Implications**:
  - release-workflow の Phase 2 タスクに package.json 更新を1行追加
  - 新規 Phase を Phase 3 と Phase 4 の間に挿入
  - Phase 6 の gh release create コマンドに VSIX アセットを追加

### WASM ビルドとパッケージング
- **Context**: VSIX パッケージング時の WASM ビルド依存関係
- **Sources Consulted**: `editors/vscode/package.json` のスクリプト定義
- **Findings**:
  - `prepackage` スクリプト: `npm run build:wasm && npm run compile`
  - `package` スクリプト: `vsce package`
  - `build:wasm`: `powershell -File scripts/build-wasm.ps1`
  - WASM ビルドには `wasm-pack` と Rust toolchain が必要
  - 生成される VSIX ファイル名: `pasta-vscode-<version>.vsix`
- **Implications**: `npm run package` の実行で WASM ビルド→コンパイル→VSIX 生成の全工程が完了する

### README 構成設計
- **Context**: Marketplace ページに表示される README のユーザー向け構成を決定
- **Sources Consulted**: 人気 VSCode 拡張（Python, ESLint, Rust Analyzer）の README パターン
- **Findings**:
  - 効果的な構成: 概要 → スクリーンショット → 機能一覧 → 要件 → 使い方 → ライセンス
  - ビルド手順やアーキテクチャは開発者向け情報として省略またはリンクのみ
  - 言語情報（Pasta DSL の紹介）は必要（ニッチ言語のため）
- **Implications**: 現在の README を大幅にリストラクチャする必要がある

## Design Decisions

### Decision: release-workflow への挿入位置
- **Context**: VSCode 拡張公開ステップをどのフェーズに配置するか
- **Alternatives Considered**:
  1. Phase 3 と Phase 4 の間に新規 Phase を挿入
  2. Phase 4（ゴーストビルド）に統合
  3. Phase 6（GitHub Release）の直前に挿入
- **Selected Approach**: Phase 3 と Phase 4 の間に挿入
- **Rationale**: crates.io 公開完了後が最初の安定ポイント。ゴーストビルドとは独立した成果物であり、分離が望ましい。vsce publish が失敗しても後続フェーズに影響しない設計
- **Trade-offs**: Phase 番号の再採番は行わず、概念的に Phase 3.5 として扱う
- **Follow-up**: release-workflow の design.md / tasks.md への反映はタスクフェーズで実施

### Decision: エラー時の継続戦略
- **Context**: `vsce publish` 失敗時のリリース継続可否
- **Alternatives Considered**:
  1. 失敗時に全リリースを中断
  2. 警告のみで後続フェーズを継続
- **Selected Approach**: 警告のみで後続フェーズを継続（要件 6.3 に準拠）
- **Rationale**: crates.io 公開はロールバック不可。VSCode Marketplace 公開はリリース後に手動でも実行可能
- **Trade-offs**: Marketplace への公開が遅延する可能性あるが、crates.io + GitHub Release の整合性は保たれる

### Decision: package.json バージョン更新の手段
- **Context**: release-workflow 実行時に package.json のバージョンをどう更新するか
- **Alternatives Considered**:
  1. npm version コマンド
  2. LLM の replace_string_in_file（Cargo.toml と同様）
  3. jq による JSON 編集
- **Selected Approach**: LLM の replace_string_in_file（Cargo.toml と同様）
- **Rationale**: 既存の release-workflow が Cargo.toml に対して同じ手法を使用しており、一貫性が高い。npm version は git tag を自動作成するため不要な副作用がある
- **Trade-offs**: JSON の構造的編集ではなく文字列置換のため、フォーマット変更に弱いが、package.json の version フィールドは安定

## Risks & Mitigations
- **SVG アイコン警告**: vsce が PNG を推奨する警告を出す可能性 → 初回パッケージング時に確認し、必要なら PNG 変換を検討
- **WASM ビルド時間**: release-workflow の実行時間が増加 → ビルド時間の計測を初回実施時に記録
- **vsce PAT 期限切れ**: PAT の有効期限（2027-02-10）が切れるとリリース不可 → リリース前チェックに PAT 有効性確認を含める（ただし vsce login で確認可能）
- **release-workflow への影響**: 既存リリースフローへの破壊的変更リスク → すべて additive 変更（Phase 2 は拡張、Phase 3.5 は新規、Phase 6 は拡張）とし、環境変数 `$env:SKIP_VSCODE_RELEASE` でスキップ可能な設計により後方互換性を保証

## References
- [VSCode Publishing Extensions](https://code.visualstudio.com/api/working-with-extensions/publishing-extension) — 公開手順公式ドキュメント
- [Keep a Changelog](https://keepachangelog.com/) — CHANGELOG 形式標準
- [vsce CLI Reference](https://github.com/microsoft/vscode-vsce) — vsce コマンドリファレンス
- `.kiro/specs/release-workflow/design.md` — 既存リリースワークフロー設計
