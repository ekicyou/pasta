# Implementation Plan: alpha05-build-ci

## Tasks

### ワークフロー作成と基本設定

- [x] 1. GitHub Actions ワークフローファイル作成 (P)
  - `.github/workflows/build.yml` を新規作成
  - ワークフロー名を "Build" に設定
  - push/pull_request/workflow_dispatch トリガーを構成（main ブランチ対象）
  - windows-latest ランナーを指定
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 8.4_

- [x] 2. マトリックスビルド構成 (P)
  - matrix.include 構造で target と arch の組み合わせを定義
  - i686-pc-windows-msvc (arch: x86) と x86_64-pc-windows-msvc (arch: x64) を追加
  - fail-fast を false に設定（一方の失敗が他方に影響しない）
  - ジョブ名を `build-${{ matrix.arch }}` に設定
  - _Requirements: 1.5, 7.1, 7.2, 7.3, 7.4, 7.5_

### ビルド環境セットアップ

- [x] 3. リポジトリチェックアウトステップ追加 (P)
  - actions/checkout@v6 を使用
  - ステップ名を "Checkout repository" に設定
  - _Requirements: 8.1_

- [x] 4. Rust ツールチェーンインストール (P)
  - dtolnay/rust-toolchain@stable を使用
  - matrix.target をターゲットとして指定
  - ステップ名を "Install Rust toolchain" に設定
  - _Requirements: 2.1, 2.2, 2.3, 8.1_

- [x] 5. ビルドキャッシュ設定 (P)
  - Swatinem/rust-cache@v2 を使用
  - キャッシュキーに matrix.target を指定（ターゲットごとに分離）
  - ステップ名を "Setup Rust cache" に設定
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 8.1_

### ビルドとテスト

- [x] 6. pasta_shiori DLL ビルド
  - `cargo build --release --target ${{ matrix.target }} -p pasta_shiori` を実行
  - ステップ名を "Build pasta_shiori" に設定
  - ビルドエラー時は非ゼロ終了コードでジョブ失敗
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 8.1_

- [x] 7. テスト実行（x64 のみ）
  - x64 ターゲットの場合のみ `cargo test --all` を実行
  - if 条件: `matrix.arch == 'x64'`
  - ステップ名を "Run tests" に設定
  - テスト失敗時は非ゼロ終了コードでジョブ失敗
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 8.1_

### アーティファクト保存

- [x] 8. ビルド成果物アップロード
  - actions/upload-artifact@v4 を使用
  - アーティファクト名: `pasta-dll-${{ matrix.arch }}`
  - パス: `target/${{ matrix.target }}/release/pasta.dll`
  - 保持期間を 7 日間に設定
  - ステップ名を "Upload artifact" に設定
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 8.1, 8.2_

### 検証と完了

- [x] 9. ワークフロー検証
  - YAML 構文エラーがないことを確認
  - 実際の push で両ターゲット（x86/x64）がビルドされることを確認
  - PR 作成時にワークフローがトリガーされることを確認
  - アーティファクトがダウンロード可能なことを確認
  - キャッシュヒット時のビルド時間短縮を確認
  - _Requirements: 2.4, 8.3_

- [x] 10. ドキュメント整合性の確認と更新
  - SOUL.md: コアバリュー・設計原則との整合性確認
  - TEST_COVERAGE.md: CI/CD テストのマッピング追加
  - README.md: CI バッジ追加（該当する場合）
  - steering/tech.md: CI/CD セクション更新（該当する場合）
  - _Requirements: すべて（1.1-8.4）_

## Task Summary

- **合計**: 10 タスク（9 実装タスク + 1 検証・ドキュメントタスク）
- **要件カバレッジ**: 全 8 要件（1.1-8.4）を網羅
- **並列実行可能**: タスク 1-5（環境セットアップまで）は並列実行可能
- **推定時間**: 各タスク 1-2 時間、合計 10-20 時間

## Dependencies

- タスク 6（ビルド）はタスク 1-5 完了後に実行
- タスク 7（テスト）はタスク 6 完了後に実行
- タスク 8（アーティファクト）はタスク 6 完了後に実行
- タスク 9（検証）はすべてのタスク完了後に実行
- タスク 10（ドキュメント）は最終タスク
