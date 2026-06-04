# Implementation Plan

- [ ] 1. Foundation: 依存追加とビルド時埋め込み
- [x] 1.1 pasta_lua への zip / md5 依存追加
  - `[dependencies]` に zip（runtime の zip 解凍用）を追加
  - `[build-dependencies]` に zip・md5 を追加（runtime は md5 不使用＝マーカー比較のみ）
  - `cargo build -p pasta_lua` が依存を解決して成功すること
  - _Requirements: 4.1, 4.2_
  - _Boundary: pasta_lua Cargo.toml_

- [x] 1.2 build.rs で決定論 zip と基準ダイジェストを生成・埋め込み
  - `pasta_scripts/` ツリー全体（socket/mime 等の同梱 Lua を含む。scriptlibs は対象外）を `OUT_DIR` の zip へアーカイブ
  - エントリ名ソート・mtime 固定・権限固定・固定圧縮レベルで、同一ソースから常にバイト同一の zip を生成
  - 生成 zip の MD5 を算出し `cargo:rustc-env=PASTA_SCRIPTS_MD5` で公開
  - ツリーを再帰walk し各ファイル・各サブディレクトリごとに `cargo:rerun-if-changed` を発行
  - 同一ソースで2回ビルドして zip がバイト同一・MD5 同一になり、`pasta_scripts/` の1ファイル変更で MD5 が変化すること
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_
  - _Depends: 1.1_
  - _Boundary: pasta_lua build.rs_

- [ ] 2. Core: 起動時自己展開モジュール
- [x] 2.1 LoaderError に自己展開失敗 variant を追加
  - 対象パスと元の I/O エラーを保持する自己展開失敗 variant を定義
  - `cargo build` が通り、ERROR ログに事実・対象パス・原因を載せられる型であること
  - _Requirements: 3.1, 3.3_
  - _Boundary: loader/error.rs_

- [x] 2.2 マーカー比較と高速パス
  - 内蔵 zip（`include_bytes!`）と基準ダイジェスト（`env!`）をコンパイル時定数として参照
  - 自己展開先（base_dir 相対 `profile/pasta/pasta_scripts/`）の `.md5` マーカーを読み、基準ダイジェストと文字列比較
  - 一致時は書き込み・ディスク再ハッシュをせず Skipped を返し、使用中の版を DEBUG ログに記録
  - 一致シナリオで自己展開先へ一切書き込みが発生しないこと
  - _Requirements: 1.1, 1.2, 1.5, 1.6, 5.4_
  - _Depends: 1.2, 2.1_
  - _Boundary: loader/extract.rs_

- [x] 2.3 アトミック展開・マーカー書き込み・ログ・フォールバック
  - 欠落／不一致時、自己展開先と同一ボリュームの一時領域へ全展開→成功確認→アトミック入れ替え（旧退避→新差し込み→旧削除）
  - `.md5` マーカーを入れ替え成功後に最後に書き込む
  - 操作対象を自己展開先と一時領域のみに限定し、`scripts/` および他のゴーストファイルに触れない
  - 解凍済み生ファイルとして配置し、再展開時に更新後の版を INFO ログに記録
  - 展開／入れ替えの失敗時は自己展開先の直前状態を保全し、自己展開失敗エラーを返す
  - 完了時、自己展開先のファイル集合＝内蔵 zip のエントリ集合（orphan なし）かつ `.md5`＝基準ダイジェストとなること（各分岐の網羅検証はタスク 5.1 に委譲）
  - _Requirements: 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 5.5_
  - _Depends: 2.2_
  - _Boundary: loader/extract.rs_

- [ ] 3. Integration: ローダ統合と検索パス整合
- [x] 3.1 PastaLoader に自己展開ステップを非致命で統合
  - base_dir 確定後・ファイル発見前（Phase 2.5）に自己展開を呼び出す
  - 失敗時は ERROR ログ（事実・対象パス・ドリフト未解消）を出力し、起動を中断せず継続
  - 正常時は package.path 構築前に自己展開先が整合済みであること
  - _Requirements: 3.1, 3.2, 6.1_
  - _Depends: 2.3_
  - _Boundary: loader/mod.rs_

- [x] 3.2 フレームワーク検索パスを自己展開先へ更新
  - 既定検索パスと hello-pasta の `pasta.toml` `[loader]` の `pasta_scripts` を `profile/pasta/pasta_scripts` へ置換（`scripts` は依然上位に維持）
  - `package.path` 内で `scripts` が `profile/pasta/pasta_scripts` より前に並び、ユーザー上書きが優先されること
  - _Requirements: 6.2, 6.3_
  - _Depends: 3.1_
  - _Boundary: loader/config.rs（既定値）, hello-pasta pasta.toml `[loader]`_

- [ ] 4. Distribution: hello-pasta 配布構成の整合
- [x] 4.1 master 同梱の撤去
  - `release.ps1` の `pasta_scripts` コピー手順を削除
  - コミット済み `ghost/master/pasta_scripts/` をリポジトリから撤去
  - `.md5` 生成・整合をリリース側で行わない（dll 所有）ことを構成上確定
  - リリース手順に `pasta_scripts` コピーが残らず、撤去後も自己展開で hello-pasta が起動できること
  - _Requirements: 5.1, 5.4, 6.4_
  - _Depends: 3.1, 3.2_
  - _Boundary: release.ps1, hello-pasta 配布構成（pasta_scripts 撤去）_

- [x] 4.2 サンプルゴースト統合テストの新方式対応
  - `tests/common/mod.rs` の pasta_scripts を master へコピーするヘルパを撤去し、検索パス検証テストを profile 自己展開前提へ更新
  - フレッシュな profile からコピーヘルパに依存せず `pasta_scripts` モジュールが解決し、`cargo test -p pasta_sample_ghost` が通ること
  - _Requirements: 6.4_
  - _Depends: 4.1_
  - _Boundary: pasta_sample_ghost tests_

- [ ] 5. Validation: テスト
- [x] 5.1 (P) 自己展開モジュールの単体テスト
  - 一致→skip（無書き込み・再ハッシュなし）、欠落／不一致→deploy（内容＝内蔵正本・orphan なし）を検証
  - 原子性（展開失敗時に旧を保全）、`.md5` は成功後に最後書き、`scripts/` 不可侵を検証
  - 各シナリオがアサーションで検証されパスすること
  - _Requirements: 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Depends: 2.3_
  - _Boundary: loader/extract.rs tests_

- [x] 5.2 (P) ビルド決定論テスト
  - 同一ソースから生成した zip がバイト同一・MD5 同一であること
  - `pasta_scripts/` の1ファイル変更で MD5 が変化すること
  - 決定論と変化反映がテストで確認できること
  - _Requirements: 4.3, 4.4, 4.5_
  - _Depends: 1.2_
  - _Boundary: build determinism test_

- [x] 5.3 サンプルゴースト統合テスト（起動・検索パス・失敗継続）
  - profile 不在のフレッシュゴーストで初回ロード時に自己展開先が生成され、`require("pasta...")` が解決すること
  - `scripts/` 上書きが優先され、SHIORI 起動挙動が不変であること
  - 自己展開先を書込不可にした際、ERROR ログを出しつつ起動が継続すること
  - 上記が統合テストで観測・検証できること
  - _Requirements: 1.3, 3.1, 3.2, 5.5, 6.2, 6.3, 6.5_
  - _Depends: 3.2, 4.2_
  - _Boundary: pasta_sample_ghost integration_

- [x]* 5.4 (P) ネットワーク更新除外の回帰テスト
  - `updates.txt` / `.nar` 生成で `profile/` 配下（自己展開先）が対象外であることを確認（既存除外動作の回帰確認）
  - 自己展開先がネットワーク更新の管理対象外であることがテストで確認できること
  - _Requirements: 5.2, 5.3_
  - _Depends: 3.2_
  - _Boundary: pasta_check updates/nar regression_

## Implementation Notes
- 環境: このWindowsホストの PowerShell セッションは `NoDefaultCurrentDirectoryInExePath=1` を設定しており、LuaJIT(mlua-sys) の vendored ビルドが exit 101 で死ぬ。cargo build/test の前に同一コマンド内で `Remove-Item Env:\NoDefaultCurrentDirectoryInExePath -ErrorAction SilentlyContinue;` を必ず付与する。
- テスト副作用: `cargo test -p pasta_lua` 実行で `crates/pasta_lua/tests/fixtures/sample.generated.lua` が CRLF 改行のみの差分で working tree に出る（内容差分ゼロ／既存ハーネス挙動）。本機能とは無関係なのでコミットに含めない（`git checkout` で戻す）。
- Phase 2.5 自己展開は `PastaLoader::load` のたびに base_dir 配下 `profile/pasta/pasta_scripts/` を内蔵 zip から生成する。よってテストヘルパが旧 `base_dir/pasta_scripts` へコピーしなくても、新既定検索パス `profile/pasta/pasta_scripts` 経由でフレームワークモジュールが解決される。
- 検索パス既定値（config.rs `default_lua_search_paths`）と hello-pasta `pasta.toml` は更新済み。`scripts` は依然 `profile/pasta/pasta_scripts` の上位（ユーザー上書き優先）。
- 配布構成: hello-pasta の tracked な pasta.toml は2箇所（`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml` と `release/hello-pasta/ghost/master/pasta.toml`）。`dist-src` には pasta.toml は存在しない。
- 横断回帰（フィーチャー検証で検出）: `pasta_shiori` の `shiori_tests.rs` の5テストが、旧 in-ghost `pasta_scripts/pasta/shiori/entry.lua` を削除・改変して「スクリプト不在/カスタム」挙動を検証していたが、自己展開によりフレームワーク entry.lua が常時 `profile/pasta/pasta_scripts` から供給され改変が無効化された。修正: 上書きを最優先のユーザー層 `scripts/pasta/shiori/entry.lua` へ移設（既存 `test_request_parsed_table_fields_accessible_in_lua` と同パターン）。今後 SHIORI スクリプト上書きをテストする際は `scripts/` 層を使うこと。
