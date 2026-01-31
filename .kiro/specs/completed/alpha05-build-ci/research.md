# Research & Design Decisions

---
**Purpose**: alpha05-build-ci の設計調査結果と判断記録
---

## Summary

- **Feature**: `alpha05-build-ci`
- **Discovery Scope**: Simple Addition（単一YAMLファイル作成）
- **Key Findings**:
  - GitHub Actions の Rust ワークフローは成熟したパターンがある
  - `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2` が推奨構成
  - Windows ランナーでの MSVC ターゲットビルドは標準サポート

## Research Log

### GitHub Actions Rust ワークフローパターン

- **Context**: 最新の推奨アクションとバージョンを確認
- **Sources Consulted**:
  - https://github.com/dtolnay/rust-toolchain
  - https://github.com/Swatinem/rust-cache
- **Findings**:
  - `dtolnay/rust-toolchain`: @stable/@nightly 等の rev 指定方式が推奨
  - `Swatinem/rust-cache`: v2.8.2（2025-11-26）が最新、自動キャッシュキー生成
  - `actions/checkout`: v6 が最新
  - `actions/upload-artifact`: v4 が現行バージョン
- **Implications**: 
  - ツールチェーンとキャッシュの順序が重要（キャッシュは rustc バージョンをキーに使用）
  - マトリックスビルド時も `shared-key` でキャッシュ共有可能

### Windows MSVC ターゲットビルド

- **Context**: `windows-latest` で x86/x64 両方ビルド可能か
- **Sources Consulted**: GitHub Actions ドキュメント、Rust ターゲット仕様
- **Findings**:
  - `windows-latest` には MSVC が標準搭載
  - `i686-pc-windows-msvc` ターゲットは `rustup target add` で追加可能
  - クロスコンパイルではなく、同一 Windows 上でのマルチターゲットビルド
- **Implications**: 追加設定なしで両アーキテクチャビルド可能

### rust-cache のマトリックス対応

- **Context**: 複数ターゲット（x86/x64）でキャッシュが分離されるか
- **Sources Consulted**: Swatinem/rust-cache README
- **Findings**:
  - デフォルトで `job_id` がキャッシュキーに含まれる
  - `add-rust-environment-hash-key: true`（デフォルト）で rustc バージョンも含まれる
  - マトリックスジョブは自動的に分離キャッシュを使用
  - `shared-key` で意図的に共有も可能
- **Implications**: 特別な設定なしでマトリックスビルドに対応

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 単一ワークフロー + マトリックス | 1つの build.yml でマトリックスビルド | シンプル、保守容易 | なし | **採用** |
| 分離ワークフロー | x86/x64 で別ファイル | 独立した制御 | 重複コード、同期困難 | 不採用 |

## Design Decisions

### Decision: アクションバージョン固定

- **Context**: 再現性と安定性のバランス
- **Alternatives Considered**:
  1. メジャーバージョンタグ（@v2）- 自動更新、破壊的変更リスク
  2. 完全 SHA 固定 - 最大安定性、更新手動
  3. メジャーバージョンタグ + Dependabot - 自動更新提案
- **Selected Approach**: メジャーバージョンタグ（@v2, @v4, @v6）
- **Rationale**: pasta は小規模プロジェクト、シンプルさ優先
- **Trade-offs**: 稀に破壊的変更の影響を受ける可能性あり
- **Follow-up**: 問題発生時にピンポイント SHA 固定を検討

### Decision: テスト実行ターゲット

- **Context**: x86/x64 両方でテストするか、x64 のみか
- **Alternatives Considered**:
  1. x64 のみ - ビルド時間短縮
  2. 両方 - 完全なカバレッジ
  3. x64 必須 + x86 オプション - バランス
- **Selected Approach**: x64 のみでテスト実行
- **Rationale**: 
  - 現行テストはアーキテクチャ依存がほぼない
  - x86 ビルド成功が品質指標として十分
  - 将来的に x86 テスト追加は容易
- **Trade-offs**: x86 固有バグを見逃す可能性（低リスク）

### Decision: workflow_dispatch トリガー

- **Context**: 手動実行機能の要否
- **Alternatives Considered**:
  1. 含めない - シンプル
  2. 含める - 手動トリガー可能
- **Selected Approach**: 含める（オプショナルとして）
- **Rationale**: 追加コスト最小限、デバッグ時に有用
- **Trade-offs**: なし（1行追加のみ）

## Risks & Mitigations

- **GitHub Actions 障害** - ビルド失敗はすぐに通知される、ローカルビルドでカバー
- **キャッシュ破損** - `prefix-key` 変更で手動リセット可能
- **アクション非互換** - メジャーバージョン更新時に確認、問題なら SHA 固定

## References

- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) - Rust ツールチェーンインストール
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) - Cargo キャッシュ（v2.8.2）
- [actions/checkout](https://github.com/actions/checkout) - リポジトリチェックアウト（v6）
- [actions/upload-artifact](https://github.com/actions/upload-artifact) - アーティファクトアップロード（v4）
