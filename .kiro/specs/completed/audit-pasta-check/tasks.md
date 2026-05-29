# Implementation Plan

- [x] 1. ファイルコピーの安全性強化
- [x] 1.1 (P) copy.rsにシンボリックリンクスキップとパストラバーサル防御を追加
  - `copy_dir_inner`でエントリのファイルタイプを取得し、シンボリックリンク（`is_symlink()`）をスキップする
  - `dst_path`が`dst`ディレクトリの配下であることを検証するチェックを追加（`starts_with`による防御）
  - シンボリックリンクを含むディレクトリのコピーでシンボリックリンクがスキップされることを確認するテストを追加
  - `cargo test -p pasta_check copy` で既存テストと新規テストが全パスする
  - _Requirements: 1.1, 1.4, 2.1_
  - _Boundary: copy.rs_

- [x] 2. NARアーカイブ作成の安全性強化
- [x] 2.1 (P) nar.rsにシンボリックリンクスキップとZIPエントリ名検証を追加
  - `add_dir_to_zip`でエントリのファイルタイプを取得し、シンボリックリンクをスキップする
  - 生成される相対パスに`..`コンポーネントが含まれないことをデバッグアサーションで検証する
  - `map_err`によるエラー変換パターンをRustイディオムに沿って簡潔化する
  - シンボリックリンクを含むディレクトリからのNAR作成でシンボリックリンクが除外されるテストを追加
  - `cargo test -p pasta_check nar` で既存テストと新規テストが全パスする
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 5.1, 5.3_
  - _Boundary: nar.rs_

- [x] 3. 更新ファイル生成のデッドコード除去と安全性強化
- [x] 3.1 (P) update_files.rsからデッドコード除去、シンボリックリンクスキップ追加、MD5コメント追記
  - `generate_updates2_dau`関数と`#[allow(dead_code)]`アトリビュートを除去する
  - `collect_files_recursive`でシンボリックリンクをスキップする処理を追加する
  - `calculate_md5`にSSP仕様準拠の非暗号学的ファイル変更検出用途であることをコードコメントとして追記する
  - `map_err`によるエラー変換パターンを簡潔化する
  - 不要な中間変数があれば削減する
  - `generate_updates2_dau`除去後もupdates.txt生成が正常動作することを既存テストで確認する
  - `cargo test -p pasta_check update_files` で既存テストが全パスする
  - _Requirements: 1.1, 1.2, 2.1, 3.1, 3.2, 4.1, 4.2, 5.1, 5.2, 5.3_
  - _Boundary: update_files.rs_

- [x] 4. リグレッション検証
- [x] 4.1 全テスト実行と外部振る舞い不変性確認
  - `cargo test -p pasta_check` で全ユニットテスト・統合テストがパスすることを確認する
  - `cargo clippy -p pasta_check` で警告が発生しないことを確認する
  - release.rsの既存テスト（`test_execute_release_full_pipeline`, `test_execute_release_with_copy`）が変更後もパスし、CLI出力・生成ファイル・NARの互換性が維持されていることを確認する
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  - _Depends: 1.1, 2.1, 3.1_
