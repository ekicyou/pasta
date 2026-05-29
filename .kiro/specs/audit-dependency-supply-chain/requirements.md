# Requirements Document

## Introduction

pastaプロジェクトは20以上の外部クレートに依存しているが、サプライチェーンセキュリティの体系的な調査が未実施である。本仕様では、全外部依存クレートに対して既知脆弱性チェック（RustSec Advisory DB）、ライセンス互換性監査、不要依存の特定・除去、バージョン固定戦略の確認・改善を実施する。Wave 1の各クレート監査で発見された依存関連の知見も統合する。

## Boundary Context
- **In scope**: ワークスペース全体のCargo.toml/Cargo.lockに記載される全外部依存クレートの調査、RustSec Advisory DBチェック、ライセンス監査、不要依存除去、バージョン固定戦略確認
- **Out of scope**: 依存クレートの内部コード修正・フォーク・パッチ、メジャーバージョンアップグレード（破壊的変更を伴うもの）、Rustツールチェイン自体の更新、CI/CDパイプラインの構築
- **Adjacent expectations**: Wave 1全audit specの依存関連知見（特にMD5用途適切性評価、unsafe箇所の外部依存関連）を本specに統合。audit-workspace-patternsは横断パターン抽出を担当し、依存監査自体は本specの責務

## Requirements

### Requirement 1: 既知脆弱性チェック
**Objective:** As a プロジェクトメンテナ, I want 全外部依存クレートの既知脆弱性を体系的にチェックしたい, so that セキュリティリスクを早期に特定・対処できる

#### Acceptance Criteria
1. When `cargo audit` を実行した場合, the 監査システム shall RustSec Advisory DBに基づき全直接・間接依存の脆弱性レポートを生成する
2. If 既知の脆弱性（advisory）が検出された場合, the 監査レポート shall 各advisoryのID・深刻度・影響範囲・推奨対処を記録する
3. If 脆弱性が検出されなかった場合, the 監査レポート shall クリーンステータスを明示する
4. The 監査システム shall 直接依存（Cargo.toml記載）と間接依存（Cargo.lock経由）の両方を検査対象とする

### Requirement 2: ライセンス互換性監査
**Objective:** As a プロジェクトメンテナ, I want 全依存クレートのライセンスがプロジェクトのライセンス（MIT OR Apache-2.0）と互換であることを確認したい, so that ライセンス違反のリスクを排除できる

#### Acceptance Criteria
1. When ライセンス監査を実行した場合, the 監査システム shall 全依存クレートのライセンス情報を収集・一覧化する
2. If GPL系ライセンスなどプロジェクトライセンスと非互換なライセンスが検出された場合, the 監査レポート shall 該当クレート名とライセンス種別を警告として記録する
3. If 全依存のライセンスが互換である場合, the 監査レポート shall ライセンス互換性の確認完了を明示する
4. The 監査システム shall vendoredソースを含む依存（mlua vendored等）のライセンスも検査対象とする

### Requirement 3: 不要依存の特定と除去
**Objective:** As a プロジェクトメンテナ, I want 実際には使用されていない依存クレートを特定・除去したい, so that ビルド時間短縮とアタックサーフェス削減ができる

#### Acceptance Criteria
1. When 依存分析を実行した場合, the 監査システム shall 各クレートの依存ツリーを解析し未使用の直接依存を特定する
2. If 未使用の依存が特定された場合, the 監査システム shall 該当依存をCargo.tomlから除去する
3. When 依存を除去した後, the ビルドシステム shall 全クレートのコンパイルが成功する
4. When 依存を除去した後, the テストスイート shall 全テスト（950+件）がパスする

### Requirement 4: バージョン固定戦略の確認・改善
**Objective:** As a プロジェクトメンテナ, I want 依存バージョンの固定戦略を確認・改善したい, so that 再現性のあるビルドと安全なアップデートが可能になる

#### Acceptance Criteria
1. When バージョン戦略を調査した場合, the 監査システム shall workspace.dependenciesでのバージョン管理の網羅性を確認する
2. If workspace.dependenciesで管理されていない直接依存が存在する場合, the 監査レポート shall 該当依存を指摘し、workspace統合を推奨する
3. When マイナー/パッチバージョンの更新が利用可能な場合, the 監査レポート shall 更新可能な依存の一覧と現行バージョン・最新バージョンを記録する
4. The 監査システム shall Cargo.lockの存在と適切性を確認する

### Requirement 5: MD5クレート用途適切性評価
**Objective:** As a プロジェクトメンテナ, I want MD5クレートの使用が適切であることを文書化したい, so that セキュリティレビューで暗号学的用途でないことを明示できる

#### Acceptance Criteria
1. When MD5クレートの用途を調査した場合, the 監査レポート shall 使用箇所と用途（ファイル変更検出 vs 暗号学的用途）を記録する
2. The 監査レポート shall MD5がSSP仕様要件に基づく非暗号学的ハッシュとして使用されていることを確認・文書化する
3. If MD5が暗号学的用途で使用されている箇所がある場合, the 監査レポート shall 代替アルゴリズムへの移行を推奨する

### Requirement 6: 監査結果の文書化
**Objective:** As a プロジェクトメンテナ, I want 監査結果を再現可能な形で文書化したい, so that 将来の監査で比較・追跡ができる

#### Acceptance Criteria
1. The 監査レポート shall 監査実行日時・使用ツールバージョン・監査対象範囲を記録する
2. The 監査レポート shall 各調査カテゴリ（脆弱性・ライセンス・不要依存・バージョン戦略）の結果を構造化して記録する
3. When 是正アクションが必要な場合, the 監査レポート shall 具体的な対処手順を記載する
4. The 監査レポート shall Wave 1各クレート監査からの依存関連知見の統合結果を含む

### Requirement 7: 回帰安全性
**Objective:** As a プロジェクトメンテナ, I want 監査に基づく変更が既存機能を壊さないことを保証したい, so that 安全にサプライチェーンを改善できる

#### Acceptance Criteria
1. When 依存の除去・更新を実施した場合, the ビルドシステム shall `cargo build` が全クレートで成功する
2. When 依存の除去・更新を実施した場合, the テストスイート shall `cargo test` で全テストがパスする
3. When 依存の除去・更新を実施した場合, the ビルドシステム shall i686-pc-windows-msvcターゲットでのクロスコンパイルが成功する
4. The 監査プロセス shall 外部振る舞い（CLI出力・生成ファイル・NAR互換性・DLL API）を不変に保つ
