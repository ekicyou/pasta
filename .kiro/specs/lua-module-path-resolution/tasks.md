# Implementation Plan

## タスク概要

本実装計画は、Lua標準`require()`ルールに準拠したモジュールパス解決機構への移行を段階的に実施します。6つの要件を7つの主要タスクに分解し、コア機能の実装から統合テストまでを網羅します。

---

## 実装タスク

### Phase 1: コア機能実装

- [ ] 1. デフォルトLua検索パスの拡張 (P)
  - `default_lua_search_paths()`関数を修正し、`user_scripts`を第2優先度として追加
  - 検索パス順序: `save/lua` → `user_scripts` → `scripts` → `cache/lua` → `scriptlibs`
  - `user_scripts`ディレクトリが存在しない場合でもエラーを発生させない設計を確認
  - _Requirements: 1.1, 1.2, 1.4, 6.1_

- [ ] 2. requireヘルパー関数の実装 (P)
  - `lua_require(lua: &Lua, module_name: &str) -> LuaResult<Value>`関数を追加
  - Luaの`globals().get("require")`でrequire関数を取得し、モジュール名で呼び出す
  - モジュールが見つからない場合はLuaエラーをそのまま伝播
  - エラーメッセージにモジュール名を含め、デバッグ可能にする
  - _Requirements: 2.4, 2.5, 2.6_

- [ ] 3. デフォルトmain.luaの作成 (P)
  - `crates/pasta_lua/scripts/main.lua`を新規作成
  - 空のテーブルを返すデフォルト実装
  - コメントでユーザー向け使用例（辞書登録、初期化処理）を含める
  - LuaDocアノテーションで`@module main`を明記
  - _Requirements: 4.1, 4.2, 4.4_

### Phase 2: 初期化シーケンス変更

- [ ] 4. 初期化シーケンスの再構成
- [ ] 4.1 main.lua読み込みのrequire化
  - `from_loader_with_scene_dic()`内で`lua_require()`を使用してmain.luaを読み込む
  - エラー時は警告ログのみで初期化継続（entry.luaと同様の扱い）
  - 読み込みタイミング: `register_finalize_scene()`の直後、entry.lua読み込み前
  - 成功時はデバッグログでモジュール名を記録
  - _Requirements: 2.1, 3.2, 3.3, 4.2_

- [ ] 4.2 entry.lua読み込みのrequire化
  - 既存の直接ファイル読み込みを`lua_require("pasta.shiori.entry")`に置き換え
  - エラー時の警告ログメッセージを維持
  - package.pathに従った検索により、ユーザーによる上書きを可能にする
  - _Requirements: 2.3_

- [ ] 4.3 scene_dic.lua生成場所の変更とrequire化
  - `CacheManager::generate_scene_dic()`で生成パスを`cache/pasta/scene_dic.lua`に変更
  - 生成前に`pasta`サブディレクトリを作成（`fs::create_dir_all`）
  - 旧パス（`cache/scene_dic.lua`）が存在する場合は削除（失敗時は警告のみ）
  - `load_scene_dic()`を`lua_require("pasta.scene_dic")`呼び出しに置き換え
  - _Requirements: 2.2_

### Phase 3: サンプルゴースト更新

- [ ] 5. pasta_sample_ghostテンプレートの更新 (P)
  - `pasta.toml.template`の`lua_search_paths`に`user_scripts`を追加
  - コメントで`user_scripts`の用途（ユーザー作成スクリプト）を説明
  - 検索パス順序がrequirements通りであることを確認
  - _Requirements: 5.1_

### Phase 4: テストと検証

- [ ] 6. ユニットテストの実装
- [ ] 6.1 検索パス拡張のテスト (P)
  - `default_lua_search_paths()`の戻り値に`user_scripts`が含まれることを検証
  - 検索パス順序が正しいことを確認（5要素、第2位が`user_scripts`）
  - _Requirements: 1.2_

- [ ] 6.2 requireヘルパー関数のテスト (P)
  - 正常系: 存在するモジュールの読み込み成功を検証
  - 異常系: 存在しないモジュールでエラーが返ることを検証
  - エラーメッセージにモジュール名が含まれることを確認
  - _Requirements: 2.4, 2.5_

- [ ] 7. 統合テストの実装
- [ ] 7.1 user_scripts優先順位検証テスト
  - `user_scripts/main.lua`と`scripts/main.lua`を両方配置
  - `user_scripts/main.lua`が優先的に読み込まれることを検証
  - 読み込まれたモジュールの内容が正しいことを確認
  - _Requirements: 1.3, 4.3_

- [ ] 7.2 初期化順序検証テスト
  - main.lua内でscene_dicファイナライズ前の状態であることを検証
  - 辞書登録APIが利用可能であることを確認
  - scene_dic読み込み後に登録した辞書が反映されていることを検証
  - _Requirements: 3.1, 3.2, 3.4_

- [ ] 7.3 デフォルトmain.lua動作検証テスト
  - user_scriptsにmain.luaを配置しない状態でPastaLoaderを実行
  - エラーなく初期化が完了することを検証
  - デフォルトの空main.luaが読み込まれたことをログで確認
  - _Requirements: 4.2_

- [ ] 7.4 scene_dic require化検証テスト
  - `require("pasta.scene_dic")`でscene_dicが解決されることを検証
  - 新パス（`cache/pasta/scene_dic.lua`）からの読み込みを確認
  - 旧パスが存在する場合、削除されることを確認
  - _Requirements: 2.2_

- [ ] 7.5* pasta_sample_ghost生成テスト
  - setup.bat実行後のゴーストに`user_scripts`ディレクトリが含まれることを検証
  - 生成されたpasta.tomlの検索パス設定が正しいことを確認
  - 生成されたゴーストが正常に起動することを検証
  - _Requirements: 5.2, 5.3, 5.4_

- [ ] 7.6* 後方互換性検証テスト
  - 既存のpasta.toml（`user_scripts`なし）でPastaLoaderを実行
  - デフォルト設定で`user_scripts`が検索パスに追加されることを検証
  - 既存のscriptsディレクトリ構造が正常に動作することを確認
  - _Requirements: 6.1, 6.2, 6.3_

---

## タスク実施順序

**Phase 1 → Phase 2 → Phase 3 → Phase 4** の順で実施。

- Phase 1の3タスク（1, 2, 3）は並列実行可能 `(P)`
- Phase 2は4.1 → 4.2 → 4.3の順で実施（初期化順序の依存関係）
- Phase 3（タスク5）はPhase 1完了後に並列実行可能 `(P)`
- Phase 4の各テストは対応する実装完了後に実行可能

**注**: `*`マークのタスクはMVP後に実施可能なオプショナルテスト
