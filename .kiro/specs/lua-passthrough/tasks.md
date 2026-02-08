# Implementation Plan: lua-passthrough

## Task Breakdown

- [x] 1. `init.*` ファイル検出・拒否機能の実装
- [x] 1.1 (P) `init.lua`/`init.pasta` 検出ロジックの実装
  - `discover_all_files` 内で検出された各ファイルのファイル名を検査
  - `init.lua` または `init.pasta` が検出された場合は `LoaderError::InvalidFileName` を返す
  - エラーメッセージにファイルパスを含め、ユーザーに具体的な問題箇所を提示
  - _Requirements: 1.4_

- [x] 1.2 (P) `init.*` 拒否のユニットテスト
  - `init.lua` が辞書ディレクトリに存在する場合のテスト
  - `init.pasta` が辞書ディレクトリに存在する場合のテスト
  - どちらも `LoaderError::InvalidFileName` が返されることを検証
  - _Requirements: 1.4_

- [x] 2. `.lua` ファイル検出機能の実装
- [x] 2.1 `.lua` パターン生成ロジックの実装
  - `pasta_patterns` の各要素の拡張子を `.lua` に変換（例: `dic/*/*.pasta` → `dic/*/*.lua`）
  - `profile/` ディレクトリは既存の `is_in_profile_dir` で自動除外される
  - glob パターン変換失敗時は警告ログを出力し、`.lua` 検出をスキップ
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2.2 `.pasta` と `.lua` ファイルの検出統合
  - `discover_all_files` で `.pasta` パターンと `.lua` パターンを両方検出
  - 検出された `.pasta` と `.lua` のファイルリストを返す
  - 既存の `discover_files` 関数を2回呼び出す実装パターン
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2.3 (P) `.lua` 検出のユニットテスト
  - `dic/*/*.pasta` → `dic/*/*.lua` パターン変換が正しいことを検証
  - `profile/` ディレクトリ内の `.lua` が除外されることを検証
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 3. モジュール名ベース衝突判定の実装
- [x] 3.1 衝突チェックロジックの実装
  - 各 `.pasta` ファイルを `CacheManager::source_to_module_name()` でモジュール名に変換
  - `.pasta` 由来のモジュール名を HashSet に格納
  - 各 `.lua` ファイルのモジュール名と照合し、衝突する `.lua` をリストから除外
  - 除外された `.lua` ファイルごとに警告ログを出力（ファイルパスとモジュール名を含む）
  - _Requirements: 1.5, 1.6_

- [x] 3.2 (P) モジュール名ベース衝突判定のユニットテスト
  - `foo/bar.pasta` と `foo/bar.lua` が同じモジュール名を生成し、`.lua` が除外されることを検証
  - 別ディレクトリの同名ファイル（`dic/a/helper.pasta` と `dic/b/helper.lua`）は衝突しないことを検証
  - _Requirements: 1.5, 1.6_

- [x] 4. `.lua` パススルー処理の実装
- [x] 4.1 拡張子ベース分岐処理の実装
  - `process_incremental` で各ファイルの拡張子を判定
  - `.pasta` ファイルは既存のトランスパイルフロー（`parse_str` → `transpile` → `save_cache`）
  - `.lua` ファイルは直接読み込み → `save_cache` でキャッシュにコピー
  - `.lua` ファイルは `parse_str` および `transpile` に渡さない
  - _Requirements: 2.1, 2.2_

- [x] 4.2 `.lua` ファイルのモジュール名生成
  - コピーした `.lua` ファイルのモジュール名を `CacheManager::source_to_module_name()` で生成
  - モジュール名を `module_names` リストに追加（`.pasta` 由来と同列）
  - `generate_scene_dic` で `.pasta` と `.lua` の両方が require エントリとして含まれる
  - _Requirements: 2.3, 2.6_

- [x] 4.3 インクリメンタル更新とスキップロジック
  - `.lua` ファイルに対しても `CacheManager::needs_transpile()` でタイムスタンプ比較
  - キャッシュが最新の場合はコピーをスキップし、`skipped` カウントを増加
  - キャッシュが古い場合のみ再コピー
  - _Requirements: 2.4, 2.5_

- [x] 4.4 ProcessStats 構造体の拡張
  - `TranspileStats` を `ProcessStats` に改名
  - `copied: usize` フィールドを追加
  - `.lua` ファイルのコピー成功時に `copied` カウントを増加
  - 既存の `transpiled`, `skipped`, `failed` フィールドは維持
  - _Requirements: 2.7_

- [x] 4.5 `.lua` コピー失敗時のエラーハンドリング
  - `.lua` ファイルの読み込み失敗時に警告ログを出力
  - `.lua` キャッシュ書き込み失敗時に警告ログを出力（既存の `save_cache` エラー処理パターン）
  - `failed` カウントを増加し、処理を継続（致命的エラーとしない）
  - _Requirements: 2.7_

- [x] 4.6 (P) `.lua` パススルー処理のユニットテスト
  - `.lua` ファイルが `parse_str` / `transpile` に渡されないことを検証
  - `ProcessStats` の `copied` カウントが正しく増加することを検証
  - _Requirements: 2.1, 2.2, 2.7_

- [x] 5. 孤立キャッシュ検出の統合
- [x] 5.1 (P) `.lua` ソースを孤立検出に含める実装
  - `find_orphaned_caches` 呼び出し時に `.lua` ソースパスを `source_paths` に含める
  - 既存の孤立検出ロジックがそのまま機能することを確認（CacheManager 変更不要）
  - _Requirements: 3.1, 3.2_

- [x] 5.2 (P) 孤立キャッシュ検出のユニットテスト
  - `.lua` ファイルを削除後、対応するキャッシュが孤立として検出されることを検証
  - `.pasta` と `.lua` のキャッシュが同じロジックで処理されることを検証
  - _Requirements: 3.1, 3.2_

- [x] 6. 統合テスト
- [x] 6.1 (P) `init.*` 拒否の統合テスト
  - `init.lua` が辞書ディレクトリに存在する場合、`PastaLoader::load` がエラーを返すことを検証
  - `init.pasta` が辞書ディレクトリに存在する場合、`PastaLoader::load` がエラーを返すことを検証
  - _Requirements: 1.4_

- [x] 6.2 (P) `.lua` パススルー E2E テスト
  - 辞書ディレクトリに `.lua` ファイルを配置
  - `PastaLoader::load` で正常にロードされることを検証
  - キャッシュディレクトリに `.lua` ファイルがコピーされることを確認
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 6.3 (P) `.pasta` + `.lua` 混在テスト
  - `.pasta` と `.lua` の両方のファイルが存在するシナリオ
  - `scene_dic.lua` に両方の require が含まれることを検証
  - 両方のモジュールがランタイムで読み込まれることを確認
  - _Requirements: 2.6_

- [x] 6.4 (P) モジュール名衝突の統合テスト
  - `foo/bar.pasta` と `foo/bar.lua` が存在する場合
  - `.pasta` のみが処理され、`.lua` が無視されることを検証
  - 警告ログが出力されることを確認
  - _Requirements: 1.5, 1.6_

- [x] 6.5 (P) インクリメンタル更新の統合テスト
  - `.lua` ファイルの内容を変更
  - キャッシュが更新されることを検証
  - タイムスタンプが最新の場合はスキップされることを確認
  - _Requirements: 2.4, 2.5_

- [x] 7. ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認（今回は既存Loaderパイプラインへの拡張のため変更不要の見込み）
  - doc/spec/ - 言語仕様の更新不要（`.lua` パススルーは実装機能であり、DSL文法に影響なし）
  - GRAMMAR.md - 文法リファレンスの同期不要（同上）
  - TEST_COVERAGE.md - 新規テストのマッピング追加（6件の統合テスト）
  - crates/pasta_lua/README.md - Loader機能説明にLuaパススルー機能を追記
  - .kiro/steering/workflow.md - 完了基準（DoD）への影響確認

## Requirements Coverage

| Requirement | Task Mapping |
|-------------|--------------|
| 1.1 | 2.1, 2.2, 2.3 |
| 1.2 | 2.1, 2.2, 2.3 |
| 1.3 | 2.1, 2.2, 2.3 |
| 1.4 | 1.1, 1.2, 6.1 |
| 1.5 | 3.1, 3.2, 6.4 |
| 1.6 | 3.1, 3.2, 6.4 |
| 2.1 | 4.1, 4.6, 6.2 |
| 2.2 | 4.1, 4.6, 6.2 |
| 2.3 | 4.2, 6.2 |
| 2.4 | 4.3, 6.5 |
| 2.5 | 4.3, 6.5 |
| 2.6 | 4.2, 6.3 |
| 2.7 | 4.4, 4.5, 4.6 |
| 3.1 | 5.1, 5.2 |
| 3.2 | 5.1, 5.2 |
