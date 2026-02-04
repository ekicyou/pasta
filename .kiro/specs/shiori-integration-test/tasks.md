# Implementation Plan

## Overview
pasta_sample_ghost の hello-pasta ゴースト定義を活用した pasta_shiori インテグレーションテストの実装。決定的なテスト環境を構築し、PastaShiori::load および request メソッドの動作を検証する。

## Tasks

- [ ] 1. ゴースト定義の修正
- [ ] 1.1 (P) boot.pasta の OnBoot シーン削減
  - `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/boot.pasta` から行11-13のOnBootシーンを削除
  - 行16-18のOnBootシーン（単語辞書呼び出しなし）のみを残す
  - ランダム要素（＠起動挨拶）を完全に排除し決定的な動作を保証
  - _Requirements: 1.1_

- [ ] 1.2 (P) pasta.toml への [talk] セクション追加
  - `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml` に [talk] セクションを追加
  - ウェイト設定5項目を定義: script_wait_normal=50, script_wait_period=1000, script_wait_comma=500, script_wait_strong=500, script_wait_leader=200
  - デフォルト値を明示してテスト期待値を確定
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

- [ ] 1.3 pasta_sample_ghost のリビルド
  - `cargo build -p pasta_sample_ghost` でビルド実行
  - `crates/pasta_sample_ghost/setup.bat` を実行してゴースト定義を再生成
  - i686-pc-windows-msvc ターゲットでの動作を確認
  - _Requirements: 1.2, 1.3_

- [ ] 2. テストユーティリティの拡張
- [ ] 2.1 (P) copy_sample_ghost_to_temp 関数の実装
  - `crates/pasta_shiori/tests/common/mod.rs` に新規関数を追加
  - pasta_sample_ghost の hello-pasta/ghost/master をテンポラリディレクトリにコピー
  - 既存の copy_dir_recursive を活用（profile ディレクトリ自動スキップ）
  - CARGO_MANIFEST_DIR からの相対パス解決を実装
  - TempDir を返してテスト環境を提供
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 3. インテグレーションテストの実装
- [ ] 3.1 テストファイルの作成
  - `crates/pasta_shiori/tests/shiori_sample_ghost_test.rs` を新規作成
  - common モジュールのインポート設定
  - PastaShiori のインポート設定
  - _Requirements: 6.1, 6.2, 6.3_

- [ ] 3.2 PastaShiori::load のテスト実装
  - test_load_hello_pasta 関数を実装
  - copy_sample_ghost_to_temp でテスト環境を構築
  - PastaShiori::default() でインスタンス作成
  - load メソッドを呼び出して Ok(true) を検証
  - ランタイム初期化の成功を確認
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 3.3 OnBoot リクエストのテスト実装
  - test_onboot_response 関数を実装
  - 完全な SHIORI/3.0 プロトコル形式のリクエストを構築（Charset, Sender, SecurityLevel, ID, Reference0 ヘッダー、CRLF 改行）
  - request メソッドで OnBoot リクエストを送信
  - 200 OK 応答を検証
  - Value ヘッダーの存在を確認
  - スポット切り替えタグ（\0, \1）の検証
  - 表情変更タグ（\s[通常]）の検証
  - ウェイトタグ（\_w[）の検証（pasta.toml [talk] セクション反映確認）
  - テキスト内容（「起動したよ～。」「さあ、始めようか。」）の検証
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [ ] 4. テストの実行と検証
- [ ] 4.1 インテグレーションテストの実行
  - `cargo test -p pasta_shiori --test shiori_sample_ghost_test` でテスト実行
  - すべてのテストが成功することを確認
  - エラー出力がないことを確認
  - _Requirements: 4.1, 4.2, 4.3, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [ ] 4.2 全体テストスイートの実行
  - `cargo test --all` で回帰テストを実行
  - 既存テストへの影響がないことを確認
  - すべてのテストが合格することを検証
  - _Requirements: 全要件_

- [ ] 5. ドキュメント整合性の確認と更新
- [ ] 5.1 SOUL.md との整合性確認
  - コアバリュー（日本語フレンドリー、UNICODE識別子、yield型、宣言的フロー）への影響を確認
  - 設計原則（行指向文法、前方一致、UI独立性）との整合性を検証
  - Phase 0完了基準（DoD）の進捗への影響を評価
  - 必要に応じて SOUL.md を更新

- [ ] 5.2 TEST_COVERAGE.md の更新
  - 新規テスト test_load_hello_pasta のマッピングを追加
  - 新規テスト test_onboot_response のマッピングを追加
  - 要件カバレッジ情報を更新

- [ ] 5.3 クレート README の確認
  - pasta_shiori/README.md でテスト構造の記載を確認
  - pasta_sample_ghost/README.md で hello-pasta の説明を確認
  - API変更がある場合は反映（今回は変更なし）

- [ ] 5.4 steering ファイルの確認
  - structure.md のテストファイル配置規約との整合性を確認
  - tech.md の依存関係情報が最新であることを確認
  - workflow.md のテスト実行手順との整合性を確認
  - 必要に応じてステアリングファイルを更新

## Requirements Coverage

| Requirement ID | Summary | Covered by Tasks |
|----------------|---------|------------------|
| 1.1, 1.2, 1.3 | OnBoot シーン修正とビルド | 1.1, 1.3 |
| 2.1-2.6 | [talk] セクション追加 | 1.2 |
| 3.1-3.3 | テスト環境セットアップ | 2.1 |
| 4.1-4.3 | PastaShiori::load 検証 | 3.2, 4.1 |
| 5.1-5.6 | OnBoot リクエスト検証 | 3.3, 4.1 |
| 6.1-6.3 | テストファイル配置 | 3.1 |

## Task Summary

- **Total**: 5 major tasks, 15 sub-tasks
- **Parallel Execution**: Tasks 1.1, 1.2, 2.1 can run in parallel (marked with (P))
- **Estimated Time**: 1-2 hours per sub-task (total: ~15-30 hours)
- **Testing**: Integration tests included in tasks 3.2, 3.3, 4.1, 4.2
- **Documentation**: Final validation in task 5

## Notes

- **Prerequisites**: i686-pc-windows-msvc ターゲットがインストールされていること（CI 環境用）
- **Build Order**: タスク1.3（setup.bat実行）は1.1と1.2の完了後に実行
- **Test Dependencies**: テスト実行（タスク4）はすべての実装タスク完了後
