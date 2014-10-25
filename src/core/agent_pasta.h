#pragma once

#include "agent_shiori.h"

namespace pasta{

    // Javascript栞[PASTA] 非同期エージェント
    class Agent :public shiori::Agent{
    
    public:
        virtual ~Agent();

        virtual void LoadAction() override;
        virtual void UnLoadAction() override;
        virtual void NotifyAction(const std::wstring& req) override;
        virtual void GetAction(const std::wstring& req) override;

    private:
        // JavaScript組み込み関数を登録します。
        void RegModuleFuncs(LPCSTR name, const duk_function_list_entry* funcs);
        void InitFileIO();
        void InitShiori();

	private:
		duk_context *ctx;
	};


}