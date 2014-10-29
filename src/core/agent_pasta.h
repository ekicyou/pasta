#pragma once

#include "agent_shiori.h"

namespace pasta{
    // Javascript栞[PASTA] 非同期エージェント
    class Agent :public shiori::Agent{
    public:
        explicit Agent(const HINSTANCE hinst);
        explicit Agent(const HINSTANCE hinst, concurrency::Scheduler& scheduler);
        explicit Agent(const HINSTANCE hinst, concurrency::ScheduleGroup& group);

        virtual ~Agent();

        virtual void LoadAction() override;
        virtual void UnLoadAction() override;
        virtual void NotifyAction(const std::string& req) override;
        virtual void GetAction(const std::string& req) override;

    private:
        void InitModuleFileIO();
        void InitModuleShiori();

    private:
        duk_context *ctx;
        duk_idx_t idx_global;
        duk_idx_t idx_func_load;
        duk_idx_t idx_func_unload;
        duk_idx_t idx_func_notify;
        duk_idx_t idx_func_get;

    public:
        // JavaScript組み込み関数を登録します。
        void RegModuleFuncs(LPCSTR name, const duk_function_list_entry* funcs);

        // evalを実行します。
        std::string eval(LPCSTR utf8text);

        // 指定モジュールのjavascriptコードを読み込む。読み込めない場合は例外。
        void LoadJS(LPCSTR moduleName);

        // モジュール用のファイルをread openし、FILE*を返す。
        FILE* OpenReadModuleFile(LPCSTR fname);
    };
}