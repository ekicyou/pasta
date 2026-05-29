# Brief: audit-dependency-supply-chain

## Problem
プロジェクトは20+の外部クレートに依存しており、サプライチェーンセキュリティの体系的な調査が未実施。バージョン固定戦略、ライセンス互換性、既知の脆弱性（RustSec Advisory DB）、不要な依存の精査が必要。

## Current State
- ワークスペース全体で20+の外部依存
- 主要依存: pest 2.8.6, mlua 0.11 (vendored), windows-sys 0.61, thiserror 2, serde 1, flate2 1.x, zip 8.4, md5 0.8
- Cargo.lock による間接依存の固定あり
- `cargo audit` の定期実行は未確認

## Desired Outcome
- 全外部依存の既知脆弱性チェック完了（RustSec Advisory DB）
- ライセンス互換性の確認（GPL汚染等の問題がないこと）
- 不要な依存の特定と除去
- バージョン固定戦略の確認・改善
- 依存更新ポリシーの文書化（オプション）

## Approach
`cargo audit`, `cargo deny`, `cargo tree` 等のツールを活用し、体系的に依存関係を調査する。Wave 1の各クレート監査で発見された依存関連の知見も統合する。

## Scope
- **In**: 全Cargo.toml の依存クレート調査、RustSec Advisory DBチェック、ライセンス監査、不要依存除去、バージョン更新検討
- **Out**: 依存クレートの内部コード修正、メジャーバージョンアップグレード（破壊的変更を伴うもの）

## Boundary Candidates
- 直接依存（Cargo.toml記載）の調査
- 間接依存（Cargo.lock経由）の調査
- ライセンス互換性チェック
- MD5クレートの用途適切性評価

## Out of Boundary
- 依存クレートのフォーク・パッチ
- Rustツールチェイン自体の更新
- CI/CDパイプラインの構築（別spec範囲）

## Upstream / Downstream
- **Upstream**: Wave 1全spec（各クレートの依存情報）
- **Downstream**: なし（調査結果は各クレートの改善に反映）

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: Wave 1全audit spec

## Constraints
- 外部振る舞い不変
- 破壊的な依存バージョン変更は行わない
- 既存テスト全パス必須
