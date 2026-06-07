-- lua_specs エントリーポイント
-- このファイルから各テストスイートを require して実行する
--
-- 新しいテストを追加する場合：
--   1. lua_specs/ に *_test.lua ファイルを作成
--   2. 下記の specs テーブルにモジュール名を追加

local specs = {
    "actor_word_test",                   -- actor-word-dictionary feature tests
    "store_save_test",                   -- store-save-table feature tests
    "persistence_spec",                  -- store-save-persistence feature tests
    "act_impl_call_test",                -- ACT_IMPL.call 4段階検索テスト
    "act_test",                          -- pasta.act トークンバッファリファクタリング tests
    "act_grouping_test",                 -- pasta.act グループ化機能 tests (actor-talk-grouping)
    "buf_test",                          -- pasta.buf バッファ抽象（new/new_fallback/backend）テスト (sakura-builder-string-buffer)
    "lua_version_test",                  -- pasta.lua_version ランタイム版整数取得テスト (sakura-builder-string-buffer)
    "sakura_builder_test",               -- pasta.shiori.sakura_builder トークン変換テスト
    "shiori_act_test",                   -- pasta.shiori.act さくらスクリプトビルダーテスト
    "virtual_dispatcher_spec",           -- OnTalk/OnHour virtual event dispatcher tests
    "store_coroutine_test",              -- STORE.co_scene coroutine management tests
    "event_coroutine_test",              -- EVENT.fire coroutine support tests
    "virtual_dispatcher_thread_test",    -- virtual_dispatcher thread return tests
    "event_no_entry_test",               -- EVENT.no_entry thread return tests
    "second_change_thread_test",         -- OnSecondChange thread passthrough tests
    "res_ok_test",                       -- RES.ok nil/empty string handling tests
    "integration_coroutine_test",        -- E2E chain talk and error handling tests
    "act_build_early_return_test",       -- ACT:build() / SHIORI_ACT:build() 早期リターンテスト
    "global_chaintalk_call_test",        -- GLOBAL チェイントーク関数登録・L3解決テスト
    "global_chaintalk_integration_test", -- GLOBAL チェイントーク EVENT.fire 統合テスト
    "persist_spot_position_test",        -- persist-spot-position スポット位置継続保持テスト
    "transfer_req_to_var_test",          -- transfer_req_to_var req→var展開テスト
    "act_method_fallback_test",          -- ACT_IMPL.call Level 4 actメソッドフォールバックテスト
    "act_find_scene_test",               -- ACT_IMPL.find_scene 5段階フォールバック検索テスト
    "global_fallback_integration_test",  -- GLOBAL フォールバック統合テスト & DSL+GLOBAL 優先順位テスト
    "act_find_act_handler_test",         -- ACT_IMPL.find_act_handler / find_handler フォールバック検索テスト
    "proxy_find_handler_test",           -- PROXY_IMPL.find_actor_handler / find_handler 検索テスト
    "set_property_test",                 -- ACT_IMPL.set_property / escape_tag_arg テスト (property-write-helpers)
    "mocks_test",                        -- lua_test.mocks install/reset/custom stub テスト
    "callback_module_test",              -- CALLBACK.try_route / sweep テスト (shiori-async-talk)
    "get_property_test",                 -- get_property バリデーション・タグ発行テスト (shiori-async-talk)
    "store_last_global_scene_test",      -- STORE.last_global_scene フィールドテスト (choice-definition-dsl)
    "act_choice_test",                   -- act:choice / act:choice_timeout トークン蓄積テスト (choice-definition-dsl)
    "act_init_scene_global_record_test",  -- init_scene グローバルシーン名記録テスト (choice-definition-dsl)
    "choice_select_test",                 -- OnChoiceSelectEx 自動ルーティングハンドラテスト (choice-definition-dsl)
    -- 将来のテストスイートをここに追加
    -- "code_generator_test",
    -- "context_test",
}

print("=====================================")
print("Running Lua test suites...")
print("=====================================")

local passed = 0
local failed = 0

for _, spec_name in ipairs(specs) do
    print(string.format("\n[SUITE] %s", spec_name))
    local ok, err = pcall(function()
        require(spec_name)
    end)
    if ok then
        passed = passed + 1
        print(string.format("  ✅ %s loaded successfully", spec_name))
    else
        failed = failed + 1
        print(string.format("  ❌ %s failed: %s", spec_name, tostring(err)))
    end
end

print("\n=====================================")
print(string.format("Test suites: %d passed, %d failed", passed, failed))
print("=====================================")

if failed > 0 then
    error(string.format("%d test suite(s) failed", failed))
end
