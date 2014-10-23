#pragma once

#include "agent_shiori.h"

namespace pasta{
    class Agent;

    typedef duk_ret_t(*pasta_c_function)(duk_context *ctx, Agent *pasta);

    class Func {
    public:
        Func(Agent *pasta, const char* key, const pasta_c_function pastafunc, const duk_int_t nargs)
            : key(key)
            , nargs(nargs)
            , func([pasta, pastafunc](duk_context *ctx){ return pastafunc(ctx, pasta); })
        {
        }

        const char* key;
        const duk_int_t nargs;
        const std::function<duk_ret_t(duk_context *ctx)> func;
    };


    // Javascript栞[PASTA] 非同期エージェント
    class Agent :public shiori::Agent{
    
    public:
        virtual ~Agent();

        virtual void LoadAction() override;
        virtual void UnLoadAction() override;
        virtual void NotifyAction(const std::wstring& req) override;
        virtual void GetAction(const std::wstring& req) override;

    private:
        void InitFileIO();
        void InitShiori();
        std::vector<Func> FileIOFuncs;
        std::vector<Func> ShioriFuncs;


    private:
        // 組み込み関数の登録
        void RegModule(const char* moduleName, const std::vector<Func> &funcs);

	private:
		duk_context *ctx;
	};


}