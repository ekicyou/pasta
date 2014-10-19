#pragma once

#include "agent_shiori.h"

namespace pasta{
    class Agent;

    typedef duk_ret_t(*pasta_c_function)(duk_context *ctx, Agent *pasta);

    class PastaFunc {
    public:
        PastaFunc(Agent *pasta, const char* key, const pasta_c_function pastafunc, const duk_int_t value)
            : key(key)
            , value(value)
        {
            func = [pasta, pastafunc](duk_context *ctx){ return pastafunc(ctx, pasta); };
        }

        const char* key;
        const duk_int_t value;
        std::function<duk_ret_t(duk_context *ctx)> func;
    };


    // Javascript�x[PASTA] �񓯊��G�[�W�F���g
    class Agent :public shiori::Agent{
    
    public:
        virtual ~Agent();

        virtual void LoadAction() override;
        virtual void UnLoadAction() override;
        virtual void NotifyAction(const std::wstring& req) override;
        virtual const std::wstring GetAction(const std::wstring& req) override;
        virtual void GetAfterAction() override;

    private:
        void InitFileIO();

	private:
		duk_context *ctx;
	};


}