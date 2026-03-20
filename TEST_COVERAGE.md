# Test Coverage Map

このドキュメントは、[SOUL.md](SOUL.md) で定義されたコア機能と、それを検証するテストの対応関係を示します。

**最終更新**: 2026-03-15（`word-multi-key` 複数キー単語定義反映）  
**総テスト数**: 880+テスト（全パス ✅）

---

## 1. コアバリューのテストカバレッジ

| コアバリュー                            | 対応テスト                                                                             | 状態   | テスト数 |
| --------------------------------------- | -------------------------------------------------------------------------------------- | ------ | -------- |
| 日本語フレンドリー（全角キーワード）    | `transpiler_basic_test.rs`<br>`transpiler_comparison_test.rs`                          | ✅ 完了 | 24       |
| UNICODE識別子（日本語シーン名・変数名） | `japanese_identifier_test.rs`<br>`ucid_test.rs`                                        | ✅ 完了 | 2<br>3   |
| yield型エンジン（継続出力）             | `transpiler_basic_test.rs`<br>`global_chaintalk_*_test.lua`<br>`runtime_scene_test.rs` | ✅ 完了 | 13       |
| 宣言的フロー（Call制御）                | `transpiler_scene_test.rs`                                                             | ✅ 完了 | -        |

---

## 2. DSL文法機能のテストカバレッジ

### 2.1 Parser層テスト（文法解析）

| 機能                     | テストファイル                                 | 状態   | 説明                                      |
| ------------------------ | ---------------------------------------------- | ------ | ----------------------------------------- |
| グローバルシーン（＊）   | `transpiler_basic_test.rs`<br>`parser_test.rs` | ✅ 完了 | シーン定義パース                          |
| ローカルシーン（・）     | `transpiler_basic_test.rs`<br>`parser_test.rs` | ✅ 完了 | サブシーン定義                            |
| アクター定義（％）       | `actor_code_block_test.rs`                     | ✅ 完了 | アクターコードブロック                    |
| 属性定義（＆）           | フィクスチャあり                               | 🔶 部分 | `transpiler2/attribute_inheritance.pasta` |
| 単語定義（＠）           | `actor_word_dictionary_test.rs`                | ✅ 完了 | 単語定義・参照                            |
| 複数キー単語定義（＠k1、k2）| `ast_test.rs`                               | ✅ 完了 | 6テスト（2キー・3キー・半角カンマ・各スコープ）|
| 変数定義（＄）           | フィクスチャあり                               | 🔶 部分 | `transpiler2/variable_scope.pasta`        |
| Call文（＞）             | `transpiler_scene_test.rs`                     | ✅ 完了 | 制御フロー                                |
| コメント行（＃）         | 暗黙的テスト                                   | 🔶 部分 | 明示的テストなし                          |
| アクション行（発言）     | `transpiler_basic_test.rs`                     | ✅ 完了 | キャラクター発言                          |
| Luaコードブロック        | `actor_code_block_test.rs`                     | ✅ 完了 | 関数定義                                  |
| バイトオフセット         | `span_byte_offset_test.rs`                     | ✅ 完了 | エラー位置特定                            |
| さくらスクリプト記号タグ | `sakura_symbol_tag_test.rs`                    | ✅ 完了 | `-+*?&` 5文字タグパース（7テスト）        |
| キューコマンド行（！/!） | `cue_cmd_test.rs`                              | ✅ 完了 | 63テスト（AST型・PEG文法・パース・推定）  |

### 2.2 Registry層テスト（シーン/単語テーブル）

| 機能                       | テストファイル                                                   | 状態   | 説明                        |
| -------------------------- | ---------------------------------------------------------------- | ------ | --------------------------- |
| シーン前方一致検索         | `search_module_test.rs`<br>`fallback_search_integration_test.rs` | ✅ 完了 | 前方一致+ランダム選択       |
| シーンサイクリングリセット | `scene_table_tests.rs`（`#[path]`テスト）                        | ✅ 完了 | 循環リセット検証（4テスト） |
| 単語前方一致検索           | `search_module_test.rs`                                          | ✅ 完了 | 単語辞書検索                |
| 単語ランダム選択           | `search_module_test.rs`                                          | 🔶 部分 | ランダム性検証              |
| アクター単語辞書           | `actor_word_dictionary_test.rs`                                  | ✅ 完了 | アクタースコープ単語        |
| finalize_scene処理         | `finalize_scene_test.rs`                                         | ✅ 完了 | シーン初期化                |
| SCENE.search() API         | `scene_search_test.rs`                                           | ✅ 完了 | 14テスト                    |

### 2.3 Transpiler層テスト（Lua変換）

| 機能                 | テストファイル                                                                              | 状態   | 説明                              |
| -------------------- | ------------------------------------------------------------------------------------------- | ------ | --------------------------------- |
| 包括的制御フロー     | `transpiler_basic_test.rs`<br>`transpiler_comparison_test.rs`<br>`transpiler_scene_test.rs` | ✅ 完了 | 24テストケース（3ファイルに分割） |
| 変数スコープ         | フィクスチャあり                                                                            | 🔶 部分 | Local/Global変数                  |
| Call/末尾Call最適化  | `transpiler_scene_test.rs`                                                                  | ✅ 完了 | 自動判定                          |
| CueCommandパススルー | `cue_command_passthrough_test.rs`                                                           | ✅ 完了 | 5テスト（Lua変換スキップ検証）    |
| 複数キー単語登録     | `transpiler.rs`（インライン）                                                               | ✅ 完了 | 7テスト（登録・Lua出力・後方互換）        |
| エンコーディング     | `pasta_lua_encoding_test.rs`                                                                | ✅ 完了 | 文字エンコード                    |

### 2.4 Runtime層テスト（実行エンジン）

| 機能                                 | テストファイル                                                    | 状態   | 説明                                         |
| ------------------------------------ | ----------------------------------------------------------------- | ------ | -------------------------------------------- |
| Luaスクリプトローダー                | `loader_startup_test.rs`<br>`loader_lifecycle_test.rs`            | ✅ 完了 | スクリプト読み込み（2ファイルに分割）        |
| 標準ライブラリモジュール             | `stdlib_modules_test.rs`                                          | ✅ 完了 | stdlib機能                                   |
| 正規表現モジュール                   | `stdlib_regex_test.rs`                                            | ✅ 完了 | 14テスト                                     |
| Lua単体テスト実行                    | `lua_unittest_runner.rs`                                          | ✅ 完了 | Luaテストランナー                            |
| STORE.save/CTX注入                   | `store_save_test.lua`                                             | ✅ 完了 | 永続変数・参照同一性                         |
| CONFIG.actor→STORE.actors初期化      | `config_actors_initialization_test.rs`                            | ✅ 完了 | 8テスト（pasta.tomlアクター設定）            |
| SHIORIレスポンスビルダー             | `shiori_res_test.rs`                                              | ✅ 完了 | 14テスト                                     |
| SHIORIイベントディスパッチ           | `shiori_event_dispatch_test.rs`<br>`shiori_event_handler_test.rs` | ✅ 完了 | 27テスト（2ファイルに分割）                  |
| SHIORI_ACT さくらスクリプト生成      | `shiori_act_test.lua`                                             | ✅ 完了 | 47テスト（transfer_date_to_var 7テスト追加） |
| SHIORI_ACT 日時転記機能              | `shiori_act_test.lua`                                             | ✅ 完了 | 7テスト（onhour-date-var-transfer）          |
| ACT トークンバッファ（親クラス）     | `act_test.lua`                                                    | ✅ 完了 | 32テスト（act-token-buffer-refactor）        |
| ACT トークングループ化               | `act_grouping_test.lua`                                           | ✅ 完了 | 23テスト（actor-talk-grouping、sakura_script grouping追加）  |
| sakura_builder トークン変換          | `sakura_builder_test.lua`                                         | ✅ 完了 | 24テスト（act-token-buffer-refactor）        |
| RuntimeConfig libs配列               | `runtime_test.rs`（外部化）                                       | ✅ 完了 | 17テスト（外部化済み）                       |
| LuaConfig TOML設定                   | `loader_config_test.rs`（外部化）                                 | ✅ 完了 | 6テスト（外部化済み）                        |
| さくらスクリプトウェイト挿入         | `sakura_script_basic_test.rs`<br>`sakura_script_output_test.rs`   | ✅ 完了 | 22テスト（2ファイルに分割）                  |
| さくらスクリプト記号タグトークナイズ | `tokenizer.rs` 内テスト                                           | ✅ 完了 | 6テスト（`-+*?&` タグ認識）                  |
| EVENT.fire コルーチン対応            | `event_coroutine_test.lua`                                        | ✅ 完了 | 16テスト（resume_until_valid含む）           |
| resume_until_valid nil yieldスキップ | `event_coroutine_test.lua`                                        | ✅ 完了 | 6テスト（coroutine-resume-loop）             |
| user_scripts検索パス優先順位         | `loader_startup_test.rs`                                          | ✅ 完了 | 2テスト（lua-module-path-resolution）        |
| main.lua初期化順序                   | `loader_lifecycle_test.rs`                                        | ✅ 完了 | 2テスト（lua-module-path-resolution）        |
| scene_dic require化                  | `loader_lifecycle_test.rs`                                        | ✅ 完了 | 3テスト（lua-module-path-resolution）        |
| lua_requireヘルパー関数              | `runtime_test.rs`（外部化）                                       | ✅ 完了 | 3テスト（lua-module-path-resolution）        |
| GLOBAL チェイントーク関数登録        | `global_chaintalk_call_test.lua`                                  | ✅ 完了 | 6テスト（yield-continuation-token）          |
| GLOBAL L3解決 + yield動作            | `global_chaintalk_call_test.lua`                                  | ✅ 完了 | 4テスト（yield-continuation-token）          |
| EVENT.fire チェイントーク統合        | `global_chaintalk_integration_test.lua`                           | ✅ 完了 | 5テスト（yield-continuation-token）          |
| @pasta_log ログブリッジ              | `log_module_test.rs`<br>`log_integration_test.rs`                 | ✅ 完了 | 24+7テスト（lua-logging）                    |
| @pasta_log スタックレベル検証        | `log_stack_level_test.rs`                                         | ✅ 完了 | 2テスト（lua-logging）                       |
| スポット位置永続化（STORE連携）      | `persist_spot_position_test.lua`                                  | ✅ 完了 | 5テスト（persist-spot-position）             |
| sakura_builder スポット処理          | `sakura_builder_test.lua`                                         | ✅ 完了 | 3テスト（persist-spot-position追加分）       |
| sakura_builder sakura_script処理     | `sakura_builder_test.lua`                                         | ✅ 完了 | 3テスト（act-sakura-script-method追加分）    |
| Action::SakuraScript アクター紐付け  | `snapshot_test.rs`                                                | ✅ 完了 | 1スナップショット（act-sakura-script-method）|
| Luaパススルー（init.*拒否）          | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（lua-passthrough）                   |
| Luaパススルー（.lua検出・コピー）    | `lua_passthrough_test.rs`                                         | ✅ 完了 | 3テスト（lua-passthrough）                   |
| Luaパススルー（モジュール名衝突）    | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（lua-passthrough）                   |
| Luaパススルー（パススルー処理）      | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（lua-passthrough）                   |
| Luaパススルー（インクリメンタル）    | `lua_passthrough_test.rs`                                         | ✅ 完了 | 1テスト（lua-passthrough）                   |
| Luaパススルー（孤立キャッシュ）      | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（lua-passthrough）                   |
| トランスパイル失敗中止               | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（load-error-logging）                |
| ログファイル名固定（Rotation::NEVER）| `logger.rs`                                                       | ✅ 完了 | 1テスト（load-error-logging）                |
| load失敗→requestエラー伝搬           | `shiori_tests.rs`                                                 | ✅ 完了 | 1テスト（load-error-logging）                |

### 2.5 LSP層テスト（Language Server）

| 機能                             | テストファイル                | 状態   | 説明                                              |
| -------------------------------- | ----------------------------- | ------ | ------------------------------------------------- |
| セマンティックトークン識別       | `semantic_token_test.rs`      | ✅ 完了 | 9テスト（17トークンタイプ）                       |
| 全角/半角マーカー同等認識        | `fullwidth_halfwidth_test.rs` | ✅ 完了 | 5テスト                                           |
| 日本語識別子トークン化           | `japanese_identifier_test.rs` | ✅ 完了 | 5テスト                                           |
| UTF-8→UTF-16位置変換             | `utf16_conversion_test.rs`    | ✅ 完了 | 12テスト（サロゲートペア含む）                    |
| LSPライフサイクル統合            | `lsp_lifecycle_test.rs`       | ✅ 完了 | 4テスト                                           |
| ドキュメント同期                 | `document_sync_test.rs`       | ✅ 完了 | 4テスト（増分更新含む）                           |
| Diagnostics通知                  | `diagnostics_test.rs`         | ✅ 完了 | 6テスト                                           |
| パーサークラッシュ回復           | `crash_recovery_test.rs`      | ✅ 完了 | 4テスト（catch_unwind保護）                       |
| 部分パーストークン提供           | `partial_token_test.rs`       | ✅ 完了 | 5テスト（Phase 1→2→3フォールバック）              |
| キューコマンドトークン生成       | `cue_command_token_test.rs`   | ✅ 完了 | 10テスト（4形式・全角半角・引数種別・混在・診断） |
| 部分パースAPI                    | `partial_parse_test.rs`       | ✅ 完了 | 28テスト（pasta_dslクレート）                     |
| DocumentManager / AnalysisEngine | `analysis_test.rs`（外部化）  | ✅ 完了 | 12テスト（外部化済み）                            |

### 2.7 VSCode拡張テスト（pasta-vscode-extension）

| 機能                              | テストファイル                                | 状態   | 説明                                                       |
| --------------------------------- | --------------------------------------------- | ------ | ---------------------------------------------------------- |
| TextMate文法（全角/半角マーカー） | `editors/vscode/src/test/tmGrammar.test.ts`   | ✅ 完了 | 20テスト（9構文×全角半角+アクション行2）                   |
| WasmBridgeユニットテスト          | `editors/vscode/src/test/wasmBridge.test.ts`  | ✅ 完了 | 10テスト（初期化・解析・エラー・dispose）                  |
| SemanticTokens/Diagnostics統合    | `editors/vscode/src/test/integration.test.ts` | ✅ 完了 | 19テスト（トークン定数・キャッシュ・デバウンス）           |
| E2E/ビルド検証                    | `editors/vscode/src/test/e2e.test.ts`         | ✅ 完了 | 30テスト（マニフェスト・文法・ビルド構成・フォールバック） |
| WASM transport Rustテスト         | `crates/pasta_lsp/src/transport.rs`           | ✅ 完了 | 7テスト（WASM型変換・severity・シリアライズ）              |

### 2.6 統合テスト（E2E）

| 機能                          | テストファイル                                                                                      | 状態   | 説明                                               |
| ----------------------------- | --------------------------------------------------------------------------------------------------- | ------ | -------------------------------------------------- |
| SHIORI.DLL インターフェース   | `shiori_lifecycle_test.rs`                                                                          | ✅ 完了 | 5テスト全パス                                      |
| SHIORI リクエスト処理         | `lua_request_test.rs`                                                                               | ✅ 完了 | 18+テスト                                          |
| Runtime E2E                   | `runtime_scene_test.rs`<br>`runtime_syntax_test.rs`                                                 | ✅ 完了 | 16テスト（2ファイルに分割）                        |
| Finalize Scene                | `finalize_scene_test.rs`                                                                            | ✅ 完了 | 14テスト                                           |
| ローカルシーン call E2E       | `local_scene_call_test.rs`                                                                          | ✅ 完了 | 3テスト（finalize経路・重複・前方一致）            |
| Virtual Event Dispatcher      | `virtual_event_dispatch_test.rs`<br>`virtual_event_config_test.rs`<br>`virtual_dispatcher_spec.lua` | ✅ 完了 | 15+20テスト（2ファイルに分割）                     |
| Sample Ghost Integration      | `shiori_sample_ghost_test.rs`                                                                       | ✅ 完了 | 2テスト（hello-pasta実ゴースト使用）               |
| Sample Ghost スクリプト整合性 | `scripts.rs::test_script_expression_names_defined_in_actors`                                        | ✅ 完了 | 1テスト（表情名↔辞書定義一致検証）                 |
| Sample Ghost dist-src 検証    | `dist_src_validation_test.rs::test_dist_src_directory_structure`                                    | ✅ 完了 | 1テスト（dist-src/ 8ファイル存在確認）             |
| Sample Ghost 画像構造検証     | `integration_test.rs::test_generated_images_structure`                                              | ✅ 完了 | 1テスト（shell/master/*.png 18枚＋surfaces.txt）   |
| チェイントーク E2E            | `runtime_scene_test.rs`                                                                             | ✅ 完了 | 2テスト（yield-continuation-token Pasta→Lua→実行） |

---

## 3. Phase 0完了基準との対応

[SOUL.md Section 6.6](SOUL.md#66-phase-0完了基準definition-of-done) のDoD項目とテストの対応：

| DoD項目                             | 対応テスト                    | 状態                           |
| ----------------------------------- | ----------------------------- | ------------------------------ |
| SPECIFICATION.md 全マーカー定義完了 | -                             | ✅ ドキュメント                 |
| 全角/半角対応表の完全性             | -                             | ✅ ドキュメント                 |
| cargo test pasta_core 100%パス      | 全pasta_coreテスト            | ✅ 67テスト                     |
| cargo test pasta_dsl 100%パス       | 全pasta_dslテスト             | ✅ 62テスト                     |
| cargo test pasta_lua 100%パス       | 全pasta_luaテスト             | ✅ 233テスト                    |
| cargo test pasta_shiori 100%パス    | `shiori_lifecycle_test.rs`    | ✅ 全パス                       |
| comprehensive_control_flow検証      | `transpiler_snapshot_test.rs` | ✅ 8スナップショット            |
| スナップショットテスト整備          | insta crate                   | ✅ 実装済み                     |
| 最適化レベルの文書化                | OPTIMIZATION.md               | ✅ 完了                         |
| ドキュメント整合性検証              | -                             | ✅ 本セッションで検証           |
| TEST_COVERAGE.md作成                | -                             | ✅ 本ドキュメント               |
| 未テスト領域の特定                  | -                             | ✅ 本ドキュメント Section 4参照 |
| シーンテーブル設計レビュー          | SCENE_TABLE_REVIEW.md         | ✅ 完了                         |
| Call文の実装                        | `transpiler_scene_test.rs`    | ✅ 完了                         |
| リグレッションテスト整備            | 全483テスト                   | ✅ 完了                         |

---

## 4. 未テスト領域・改善点

### 4.1 明示的テストが不足している領域

| 機能                                | 現状                   | 推奨アクション                                 |
| ----------------------------------- | ---------------------- | ---------------------------------------------- |
| コメント行（＃）パース              | ✅ 明示的テスト追加済み | `test_comment_line_explicit_parse()`           |
| 属性定義（＆）の継承                | ✅ 明示的テスト追加済み | `test_attribute_inheritance()`                 |
| 変数スコープ（Local/Global/System） | ✅ 明示的テスト追加済み | `test_variable_scope_complete()`               |
| 単語ランダム選択の検証              | ✅ 明示的テスト追加済み | `test_word_random_selection_and_replacement()` |
| エラーメッセージの具体性            | ✅ 明示的テスト追加済み | `test_error_message_specificity()`             |

### 4.2 Golden Test（スナップショットテスト）未整備

現在、`comprehensive_control_flow.pasta` に対応する `.rn` ファイルが存在しますが、これはrune時代の遺物です。Lua出力に対するスナップショットテストが必要です。

**推奨**: `insta` crateを使用したスナップショットテスト導入

### 4.3 pasta_shiori テスト状況 ✅ 解決済み

以下5テストが修正完了：

- ✅ `test_shiori_load_sets_globals`
- ✅ `test_shiori_request_calls_pasta_scene`
- ✅ `test_shiori_request_increments_counter`
- ✅ `test_shiori_unload_creates_marker`
- ✅ `test_shiori_lifecycle_lua_execution_verified`

**修正内容**: 
- `pasta_lua/scripts/pasta/`から完全なLuaモジュールセットを`pasta_shiori/tests/support/scripts/pasta/`にコピー
- `copy_fixture_to_temp()`のコピー順序を修正（サポートファイル→フィクスチャの順）

---

## 5. テストカバレッジサマリー

| クレート     | テスト数 | パス     | 失敗  | カバレッジ評価                              |
| ------------ | -------- | -------- | ----- | ------------------------------------------- |
| pasta_dsl    | 153      | 153      | 0     | 優秀（cue_cmd 63テスト含む）                |
| pasta_core   | 67       | 67       | 0     | ⭐⭐⭐⭐⭐ 優秀                                  |
| pasta_lua    | 483      | 483      | 0     | 優秀（cue_command_passthrough 5テスト含む） |
| pasta_lsp    | 79       | 79       | 0     | ⭐⭐⭐⭐⭐ 優秀                                  |
| pasta_shiori | 28       | 28       | 0     | ⭐⭐⭐⭐⭐ 優秀                                  |
| pasta-vscode | 79       | 79       | 0     | ⭐⭐⭐⭐⭐ 優秀                                  |
| **合計**     | **989+** | **989+** | **0** | **100%パス率**                              |

---

## 6. CI/CD テスト環境

### GitHub Actions ワークフロー

| ワークフロー | ファイル                      | トリガー     | 説明                       |
| ------------ | ----------------------------- | ------------ | -------------------------- |
| Build        | `.github/workflows/build.yml` | push/PR/手動 | x86/x64 DLL ビルド・テスト |

### ビルドマトリックス

| ターゲット               | アーキテクチャ | テスト実行           | アーティファクト |
| ------------------------ | -------------- | -------------------- | ---------------- |
| `i686-pc-windows-msvc`   | x86            | ❌ ビルドのみ         | `pasta-dll-x86`  |
| `x86_64-pc-windows-msvc` | x64            | ✅ `cargo test --all` | `pasta-dll-x64`  |

### CI検証項目

- **YAML構文**: GitHub Actions による自動検証
- **ビルド成功**: 両アーキテクチャで pasta.dll 生成
- **テスト全パス**: x64 環境で全テスト実行
- **アーティファクト保存**: 7日間保持

---

## 7. 次のステップ

### Phase 0完了 ✅

全DoD項目を達成しました：

1. ~~**優先度 High**: pasta_shiori 5テスト失敗の修正~~ ✅ 完了
2. ~~**優先度 High**: Golden Test（スナップショットテスト）整備~~ ✅ 完了（8スナップショット）
3. ~~**優先度 Medium**: 最適化レベルの文書化~~ ✅ 完了（OPTIMIZATION.md）
4. ~~**優先度 Medium**: シーンテーブル設計レビュー~~ ✅ 完了（SCENE_TABLE_REVIEW.md）

### Phase 1に向けて

- 属性フィルタリング機能の実装
- comprehensive_control_flow.pastaの文法更新
- パフォーマンスベンチマーク

### 保守・拡張

- 新規テスト追加時は本ドキュメントを更新
- 四半期ごとにテストカバレッジレビュー実施
- Phase 1以降の機能追加時は対応するテストを先に作成（Test-First）

---

**参照**:
- [SOUL.md](SOUL.md) - プロジェクトの憲法
- [.kiro/specs/soul-document/gap-analysis.md](.kiro/specs/soul-document/gap-analysis.md) - ギャップ分析レポート
