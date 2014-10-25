#pragma once

#include "agent_shiori.h"

namespace pasta{

    // Javascript�x[PASTA] �񓯊��G�[�W�F���g
    class Agent :public shiori::Agent{
    
    public:
        virtual ~Agent();

        virtual void LoadAction() override;
        virtual void UnLoadAction() override;
        virtual void NotifyAction(const std::wstring& req) override;
        virtual void GetAction(const std::wstring& req) override;

    private:
        // JavaScript�g�ݍ��݊֐���o�^���܂��B
        void RegModuleFuncs(LPCSTR name, const duk_function_list_entry* funcs);
        void InitFileIO();
        void InitShiori();

	private:
		duk_context *ctx;
	};


}