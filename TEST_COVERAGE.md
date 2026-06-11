# Test Coverage Map

このドキュメントは、[SOUL.md](SOUL.md) で定義されたコア機能と、それを検証するテストの対応関係を示します。

**最終更新**: 2026-06-12（`review-improvement-loop` セル 3.64 — ループ成果の台帳同期・全件数を実測値へ更新）  
**総テスト数**: 2091テスト（Rust workspace 1956＋VSCode拡張 135・全パス ✅）

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

| 機能                         | テストファイル                                 | 状態   | 説明                                            |
| ---------------------------- | ---------------------------------------------- | ------ | ----------------------------------------------- |
| グローバルシーン（＊）       | `transpiler_basic_test.rs`<br>`parser_test.rs` | ✅ 完了 | シーン定義パース                                |
| ローカルシーン（・）         | `transpiler_basic_test.rs`<br>`parser_test.rs` | ✅ 完了 | サブシーン定義                                  |
| アクター定義（％）           | `actor_code_block_test.rs`                     | ✅ 完了 | アクターコードブロック                          |
| 属性定義（＆）               | フィクスチャあり                               | 🔶 部分 | `transpiler2/attribute_inheritance.pasta`       |
| 単語定義（＠）               | `actor_word_dictionary_test.rs`                | ✅ 完了 | 単語定義・参照                                  |
| 複数キー単語定義（＠k1、k2） | `ast_test.rs`                                  | ✅ 完了 | 6テスト（2キー・3キー・半角カンマ・各スコープ） |
| 変数定義（＄）               | フィクスチャあり                               | 🔶 部分 | `transpiler2/variable_scope.pasta`              |
| Call文（＞）                 | `transpiler_scene_test.rs`                     | ✅ 完了 | 制御フロー                                      |
| コメント行（＃）             | 暗黙的テスト                                   | 🔶 部分 | 明示的テストなし                                |
| アクション行（発言）         | `transpiler_basic_test.rs`                     | ✅ 完了 | キャラクター発言                                |
| Luaコードブロック            | `actor_code_block_test.rs`                     | ✅ 完了 | 関数定義                                        |
| バイトオフセット             | `span_byte_offset_test.rs`                     | ✅ 完了 | エラー位置特定                                  |
| さくらスクリプト記号タグ     | `sakura_symbol_tag_test.rs`                    | ✅ 完了 | `-+*?&` 5文字タグパース（7テスト）              |
| キューコマンド行（！/!）     | `cue_cmd_test.rs`                              | ✅ 完了 | 63テスト（AST型・PEG文法・パース・推定）        |
| プロパティスコープ（＄％）   | `property_scope_test.rs`                       | ✅ 完了 | 16テスト（property-dsl-extension）              |
| 選択肢行（＠？）             | `choice_line_test.rs`                          | ✅ 完了 | 選択肢行パース（省略形・括弧形・半角対応）      |
| 動的コール（＞式）           | `dynamic_call_test.rs`                         | ✅ 完了 | 7テスト（CallTarget::Dynamic）                  |
| 式文（＄＝expr）             | `var_set_none_test.rs`                         | ✅ 完了 | 8テスト（名前なしVarSet）                       |
| ParseError 公開API           | `error_api_test.rs`                            | ✅ 完了 | 16テスト（コンストラクタ・Display・行列位置）   |
| 式AST（二項演算・括弧）      | `expr_parse_test.rs`                           | ✅ 完了 | 14テスト（左結合・全角数値・キーワード引数）    |

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
| Registry境界回帰テスト     | `scene_registry.rs`<br>`word_registry.rs`<br>`scene_table_tests.rs`<br>`word_table_test.rs`<br>`random.rs`<br>`error.rs`（各内テスト） | ✅ 完了 | 38テスト（merge_from/register_global_raw/解決境界/セレクタ契約/エラー表示文言） |

### 2.3 Transpiler層テスト（Lua変換）

| 機能                   | テストファイル                                                                              | 状態   | 説明                               |
| ---------------------- | ------------------------------------------------------------------------------------------- | ------ | ---------------------------------- |
| 包括的制御フロー       | `transpiler_basic_test.rs`<br>`transpiler_comparison_test.rs`<br>`transpiler_scene_test.rs` | ✅ 完了 | 24テストケース（3ファイルに分割）  |
| 変数スコープ           | フィクスチャあり                                                                            | 🔶 部分 | Local/Global変数                   |
| Call/末尾Call最適化    | `transpiler_scene_test.rs`                                                                  | ✅ 完了 | 自動判定                           |
| CueCommandパススルー   | `cue_command_passthrough_test.rs`                                                           | ✅ 完了 | 5テスト（Lua変換スキップ検証）     |
| 複数キー単語登録       | `transpiler.rs`（インライン）                                                               | ✅ 完了 | 7テスト（登録・Lua出力・後方互換） |
| エンコーディング       | `pasta_lua_encoding_test.rs`                                                                | ✅ 完了 | 文字エンコード                     |
| プロパティLua変換      | `property_scope_codegen_test.rs`                                                            | ✅ 完了 | 10テスト（property-dsl-extension） |
| プロパティトークン保全 | `property_token_preservation_test.rs`                                                       | ✅ 完了 | 3テスト（property-dsl-extension）  |
| 選択肢Lua変換          | `scope_gen.rs`                                                                              | ✅ 完了 | 選択肢行→act:choice()変換          |
| 選択肢タイムアウトLua変換 | `scope_gen.rs`                                                                           | ✅ 完了 | !select→act:choice_timeout()変換   |

### 2.4 Runtime層テスト（実行エンジン）

| 機能                                  | テストファイル                                                    | 状態   | 説明                                                             |
| ------------------------------------- | ----------------------------------------------------------------- | ------ | ---------------------------------------------------------------- |
| Luaスクリプトローダー                 | `loader_startup_test.rs`<br>`loader_lifecycle_test.rs`            | ✅ 完了 | スクリプト読み込み（2ファイルに分割）                            |
| 標準ライブラリモジュール              | `stdlib_modules_test.rs`                                          | ✅ 完了 | stdlib機能                                                       |
| 正規表現モジュール                    | `stdlib_regex_test.rs`                                            | ✅ 完了 | 14テスト                                                         |
| Lua単体テスト実行                     | `lua_unittest_runner.rs`                                          | ✅ 完了 | Luaテストランナー                                                |
| STORE.save/CTX注入                    | `store_save_test.lua`                                             | ✅ 完了 | 10テスト（永続変数・参照同一性）                                 |
| STORE コルーチン連携                  | `store_coroutine_test.lua`                                        | ✅ 完了 | 7テスト                                                          |
| STORE 直前グローバルシーン記録        | `store_last_global_scene_test.lua`                                | ✅ 完了 | 3テスト                                                          |
| 永続化仕様（save/load 劣化動作）      | `persistence_spec.lua`                                            | ✅ 完了 | 12テスト                                                         |
| 秒変化スレッド                        | `second_change_thread_test.lua`                                   | ✅ 完了 | 3テスト                                                          |
| RES OK レスポンス生成                 | `res_ok_test.lua`                                                 | ✅ 完了 | 7テスト                                                          |
| CONFIG.actor→STORE.actors初期化       | `config_actors_initialization_test.rs`                            | ✅ 完了 | 8テスト（pasta.tomlアクター設定）                                |
| SHIORIレスポンスビルダー              | `shiori_res_test.rs`                                              | ✅ 完了 | 14テスト                                                         |
| SHIORIイベントディスパッチ            | `shiori_event_dispatch_test.rs`<br>`shiori_event_handler_test.rs` | ✅ 完了 | 27テスト（2ファイルに分割）                                      |
| SHIORI_ACT さくらスクリプト生成       | `shiori_act_test.lua`                                             | ✅ 完了 | 43テスト（日時転記 transfer_date_to_var 6テストを含むファイル全体実測 — 旧 47＋7 の二重計上を是正） |
| ACT トークンバッファ（親クラス）      | `act_test.lua`                                                    | ✅ 完了 | 36テスト（act-token-buffer-refactor）                            |
| ACT トークングループ化                | `act_grouping_test.lua`                                           | ✅ 完了 | 25テスト（actor-talk-grouping、sakura_script grouping追加）      |
| sakura_builder トークン変換           | `sakura_builder_test.lua`                                         | ✅ 完了 | 52テスト（ファイル全体実測 — スポット/sakura_script/choice/string-buffer 系を含む） |
| RuntimeConfig libs配列                | `runtime_test.rs`（外部化）                                       | ✅ 完了 | 17テスト（外部化済み）                                           |
| LuaConfig TOML設定                    | `loader_config_test.rs`（外部化）                                 | ✅ 完了 | 6テスト（外部化済み）                                            |
| さくらスクリプトウェイト挿入          | `sakura_script_basic_test.rs`<br>`sakura_script_output_test.rs`   | ✅ 完了 | 22テスト（2ファイルに分割）                                      |
| さくらスクリプト記号タグトークナイズ  | `tokenizer.rs` 内テスト                                           | ✅ 完了 | 6テスト（`-+*?&` タグ認識）                                      |
| EVENT.fire コルーチン対応             | `event_coroutine_test.lua`                                        | ✅ 完了 | 16テスト（resume_until_valid含む）                               |
| resume_until_valid nil yieldスキップ  | `event_coroutine_test.lua`                                        | ✅ 完了 | 6テスト（coroutine-resume-loop）                                 |
| CALLBACK モジュール（非同期通信基盤） | `callback_module_test.lua`                                        | ✅ 完了 | 21テスト（shiori-async-talk）                                    |
| get_property バリデーション・タグ発行 | `get_property_test.lua`                                           | ✅ 完了 | 18テスト（shiori-async-talk）                                    |
| set_property バリデーション・反映     | `set_property_test.lua`                                           | ✅ 完了 | 18テスト                                                         |
| REQ→変数転記                          | `transfer_req_to_var_test.lua`                                    | ✅ 完了 | 8テスト                                                          |
| プロキシハンドラ解決                  | `proxy_find_handler_test.lua`                                     | ✅ 完了 | 12テスト                                                         |
| scripts検索パス優先順位               | `loader_startup_test.rs`                                          | ✅ 完了 | 2テスト（lua-module-path-resolution）                            |
| main.lua初期化順序                    | `loader_lifecycle_test.rs`                                        | ✅ 完了 | 2テスト（lua-module-path-resolution）                            |
| scene_dic require化                   | `loader_lifecycle_test.rs`                                        | ✅ 完了 | 3テスト（lua-module-path-resolution）                            |
| lua_requireヘルパー関数               | `runtime_test.rs`（外部化）                                       | ✅ 完了 | 3テスト（lua-module-path-resolution）                            |
| GLOBAL チェイントーク関数登録         | `global_chaintalk_call_test.lua`                                  | ✅ 完了 | 2テスト（yield-continuation-token・ファイル計 6 = 登録 2＋L3解決 4） |
| GLOBAL L3解決 + yield動作             | `global_chaintalk_call_test.lua`                                  | ✅ 完了 | 4テスト（yield-continuation-token）                              |
| act:find_scene 5段階フォールバック    | `act_find_scene_test.lua`                                         | ✅ 完了 | 13テスト（event-handler-call-equivalence）                       |
| GLOBAL フォールバック統合             | `global_fallback_integration_test.lua`                            | ✅ 完了 | 7テスト（event-handler-call-equivalence）                        |
| EVENT.fire チェイントーク統合         | `global_chaintalk_integration_test.lua`                           | ✅ 完了 | 5テスト（yield-continuation-token）                              |
| @pasta_log ログブリッジ               | `log_module_test.rs`<br>`log_integration_test.rs`                 | ✅ 完了 | 24+7テスト（lua-logging）                                        |
| PastaLuaRuntime 公開 API・@pasta_log 境界 | `runtime_api_test.rs`                                         | ✅ 完了 | 20+2+2テスト（review-improvement-loop 3.16〜3.18 — exec_file/exec_named/register_module/from_loader/@pasta_config 変換・深さゲート回帰・深さ境界ペア） |
| @pasta_log スタックレベル検証         | `log_stack_level_test.rs`                                         | ✅ 完了 | 2テスト（lua-logging）                                           |
| スポット位置永続化（STORE連携）       | `persist_spot_position_test.lua`                                  | ✅ 完了 | 8テスト（persist-spot-position）                                 |
| sakura_builder スポット処理           | `sakura_builder_test.lua`                                         | ✅ 完了 | スポット切替検証（上記 52 テストに含む）                         |
| sakura_builder sakura_script処理      | `sakura_builder_test.lua`                                         | ✅ 完了 | act-sakura-script-method 検証（上記 52 テストに含む）            |
| 選択肢さくらスクリプト変換            | `sakura_builder_test.lua`                                         | ✅ 完了 | choice/choice_timeoutトークン→\q[],\![set,choicetimeout]変換（上記 52 テストに含む） |
| Action::SakuraScript アクター紐付け   | `snapshot_test.rs`                                                | ✅ 完了 | 1スナップショット（act-sakura-script-method）                    |
| Luaパススルー（init.*拒否）           | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（lua-passthrough）                                       |
| Luaパススルー（.lua検出・コピー）     | `lua_passthrough_test.rs`                                         | ✅ 完了 | 3テスト（lua-passthrough）                                       |
| Luaパススルー（モジュール名衝突）     | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（lua-passthrough）                                       |
| Luaパススルー（パススルー処理）       | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（lua-passthrough）                                       |
| Luaパススルー（インクリメンタル）     | `lua_passthrough_test.rs`                                         | ✅ 完了 | 1テスト（lua-passthrough）                                       |
| Luaパススルー（孤立キャッシュ）       | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（lua-passthrough）                                       |
| トランスパイル失敗中止                | `lua_passthrough_test.rs`                                         | ✅ 完了 | 2テスト（load-error-logging）                                    |
| ログファイル名固定（Rotation::NEVER） | `logger.rs`                                                       | ✅ 完了 | 1テスト（load-error-logging）                                    |
| load失敗→requestエラー伝搬            | `shiori_tests.rs`                                                 | ✅ 完了 | 1テスト（load-error-logging）                                    |
| 非同期コールバック統合（SHIORI層）    | `async_callback_integration_test.rs`                              | ✅ 完了 | 12テスト（shiori-async-talk、property-dsl-extension追加2テスト） |
| OnChoiceSelectEx 自動ルーティング     | `choice_select_test.lua`（Luaテストスイート）                     | ✅ 完了 | 選択肢コールバック→シーン自動解決                                |

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
| DocumentManager / AnalysisEngine | `analysis_test.rs`（外部化）  | ✅ 完了 | 15テスト（コメント走査・改行正規化含む）          |
| 変数代入行トークン生成           | `var_set_token_test.rs`       | ✅ 完了 | 10テスト（マーカー/名前/演算子/RHS 各種・スコープ） |
| 解析エンジン no-panic 境界       | `analyze_robustness_test.rs`  | ✅ 完了 | 1テスト（敵対的入力コーパス 40 種）               |
| ドキュメント管理・位置変換境界   | `document.rs`（インライン）   | ✅ 完了 | 11テスト（逆転 range no-op ハードニング回帰含む） |
| エラー型 Display 契約            | `error.rs`（インライン）      | ✅ 完了 | 2テスト                                           |

### 2.7 VSCode拡張テスト（pasta-vscode-extension）

| 機能                              | テストファイル                                | 状態   | 説明                                                       |
| --------------------------------- | --------------------------------------------- | ------ | ---------------------------------------------------------- |
| TextMate文法（全角/半角マーカー） | `editors/vscode/src/test/tmGrammar.test.ts`   | ✅ 完了 | 27テスト（9構文×全角半角+アクション行ほか）                |
| デバッグアダプタファクトリ        | `editors/vscode/src/test/debugAdapterFactory.test.ts` | ✅ 完了 | 16テスト（attach 解決・ポート既定値）             |
| ソース表示トグル                  | `editors/vscode/src/test/sourcePresentationToggle.test.ts` | ✅ 完了 | 17テスト                                     |
| VSCodeモジュール直接テスト        | `editors/vscode/src/test/vscodeModules.test.ts` | ✅ 完了 | 45テスト（esbuild --alias:vscode モックで実モジュール検証。旧 wasmBridge.test.ts / integration.test.ts は 3.57 で本テストへ統合・削除済み） |
| E2E/ビルド検証                    | `editors/vscode/src/test/e2e.test.ts`         | ✅ 完了 | 30テスト（マニフェスト・文法・ビルド構成・フォールバック） |
| WASM transport Rustテスト         | `crates/pasta_lsp/src/transport.rs`           | ✅ 完了 | 9テスト（WASM型変換・severity 全分岐・JSON 形状・シリアライズ） |

### 2.6 統合テスト（E2E）

| 機能                          | テストファイル                                                                                      | 状態   | 説明                                                                                                       |
| ----------------------------- | --------------------------------------------------------------------------------------------------- | ------ | ---------------------------------------------------------------------------------------------------------- |
| SHIORI.DLL インターフェース   | `shiori_lifecycle_test.rs`                                                                          | ✅ 完了 | 5テスト全パス                                                                                              |
| SHIORI リクエスト処理         | `lua_request_test.rs`                                                                               | ✅ 完了 | 29テスト（X-Pasta-Time時刻注入 5・review-improvement-loop 3.35/3.37 エラーパス＋スタック安全追加分含む）   |
| ShioriTestEnv統合テスト       | `shiori_test_env_test.rs`<br>`common/response.rs`                                                   | ✅ 完了 | 5+9テスト（ShioriTestEnv E2E・ShioriResponseパーサー）                                                     |
| Runtime E2E                   | `runtime_scene_test.rs`<br>`runtime_syntax_test.rs`                                                 | ✅ 完了 | 16テスト（2ファイルに分割）                                                                                |
| Finalize Scene                | `finalize_scene_test.rs`                                                                            | ✅ 完了 | 14テスト                                                                                                   |
| ローカルシーン call E2E       | `local_scene_call_test.rs`                                                                          | ✅ 完了 | 3テスト（finalize経路・重複・前方一致）                                                                    |
| Virtual Event Dispatcher      | `virtual_event_dispatch_test.rs`<br>`virtual_event_config_test.rs`<br>`virtual_dispatcher_spec.lua`<br>`virtual_dispatcher_thread_test.lua` | ✅ 完了 | Rust 16+15テスト（2ファイルに分割）＋Lua spec 44テスト＋スレッド 7テスト<br>おしゃべり頻度SAVE永続化（SAVE>toml>default, floor/clamp/Inf guard）含む |
| Sample Ghost Integration      | `shiori_sample_ghost_test.rs`                                                                       | ✅ 完了 | 2テスト（hello-pasta実ゴースト使用）                                                                       |
| Sample Ghost スクリプト整合性 | `scripts.rs::test_script_expression_names_defined_in_actors`                                        | ✅ 完了 | 1テスト（表情名↔辞書定義一致検証）                                                                         |
| Sample Ghost 構成検証         | `dist_src_validation_test.rs::test_ghost_directory_structure`                                       | ✅ 完了 | 1テスト（ghosts/hello-pasta/ 8ファイル存在確認）※テストファイル名は旧称                                    |
| Sample Ghost 画像構造検証     | `integration_test.rs::test_generated_images_structure`                                              | ✅ 完了 | 1テスト（shell/master/*.png 18枚＋surfaces.txt）                                                           |
| チェイントーク E2E            | `runtime_scene_test.rs`                                                                             | ✅ 完了 | 2テスト（yield-continuation-token Pasta→Lua→実行）                                                         |

---

## 3. Phase 0完了基準との対応

[SOUL.md Section 6.6](SOUL.md#66-phase-0完了基準definition-of-done) のDoD項目とテストの対応：

| DoD項目                             | 対応テスト                    | 状態                           |
| ----------------------------------- | ----------------------------- | ------------------------------ |
| SPECIFICATION.md 全マーカー定義完了 | -                             | ✅ ドキュメント                 |
| 全角/半角対応表の完全性             | -                             | ✅ ドキュメント                 |
| cargo test pasta_core 100%パス      | 全pasta_coreテスト            | ✅ 105テスト                    |
| cargo test pasta_dsl 100%パス       | 全pasta_dslテスト             | ✅ 239テスト                    |
| cargo test pasta_lua 100%パス       | 全pasta_luaテスト             | ✅ 1240テスト                   |
| cargo test pasta_shiori 100%パス    | 全pasta_shioriテスト          | ✅ 157テスト                    |
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
- `pasta_lua/pasta_scripts/pasta/`から完全なLuaモジュールセットを`pasta_shiori/tests/support/scripts/pasta/`にコピー
- `copy_fixture_to_temp()`のコピー順序を修正（サポートファイル→フィクスチャの順）

---

## 5. テストカバレッジサマリー

| クレート     | テスト数  | パス      | 失敗  | カバレッジ評価                                             |
| ------------ | --------- | --------- | ----- | ---------------------------------------------------------- |
| pasta_check  | 65        | 65        | 0     | 優秀（review-improvement-loop 3.42/3.43 で CLI 終了コード・リリース検証 34テスト追加 31→65） |
| pasta_dsl    | 239       | 239       | 0     | 優秀（cue_cmd 63テスト含む、review-improvement-loop で ParseError API・式 AST 33テスト追加） |
| pasta_core   | 105       | 105       | 0     | ⭐⭐⭐⭐⭐ 優秀（review-improvement-loop で境界回帰38テスト追加） |
| pasta_lua    | 1240      | 1240      | 0     | 優秀（cue_command_passthrough 5テスト・mocks 8テスト含む、review-improvement-loop で code_gen 直接ユニット31・enc-log-search 38・FFI 長ガード回帰2・loader-core 27＋junction 回帰1・loader-io 21＋ハードニング回帰4・runtime API 28＋log 深さゲート回帰2＋境界ペア2・sakura_script エッジケース21・debug-core 未到達経路14＋Content-Length 上限ガード回帰1・debug-dap/breakpoints 劣化経路10・scopes frameId オーバーフロー飽和回帰1・debug-hook/inspect 未到達経路10・debug-session stop_loop ルーティング/純粋判定12＋scopes 飽和回帰1・debug-source_map 未到達経路11＋stale 残留是正回帰1・debug-wiring 未到達経路13テスト追加 1240/0） |
| pasta_lsp    | 112       | 112       | 0     | ⭐⭐⭐⭐⭐ 優秀（review-improvement-loop 3.40 で var_set トークン・コメント走査ほか18テスト追加 92→110、3.41 で逆転 range ハードニング回帰＋敵対的入力コーパス2テスト追加 110→112） |
| pasta_shiori | 157       | 157       | 0     | ⭐⭐⭐⭐⭐ 優秀（ShioriTestEnv・X-Pasta-Time、review-improvement-loop 3.35/3.37 で windows.rs FFI 層・エラーパス・HGLOBAL リーク回帰・スタック安全 37 テスト追加 120→157） |
| pasta_sample_ghost | 38  | 38        | 0     | 優秀（review-improvement-loop 3.44/3.45 でスクリプト整合・画像構造ほか追加。Rust 実測 38/0） |
| pasta-vscode | 135       | 135       | 0     | ⭐⭐⭐⭐⭐ 優秀（tmGrammar 27＋debugAdapterFactory 16＋sourcePresentationToggle 17＋vscodeModules 45＋e2e 30 — npm run test 実測） |
| **合計**     | **2091**  | **2091**  | **0** | **100%パス率（Rust workspace 1956＋vscode 135 — 2026-06-12 実測。Lua spec 598 件は lua_unittest_runner 経由で workspace 数値に内包）** |

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
