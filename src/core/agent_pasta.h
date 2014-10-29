#pragma once

#include "agent_shiori.h"

namespace pasta{
    // Javascript�x[PASTA] �񓯊��G�[�W�F���g
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
        // JavaScript�g�ݍ��݊֐���o�^���܂��B
        void RegModuleFuncs(LPCSTR name, const duk_function_list_entry* funcs);

        // eval�����s���܂��B
        std::string eval(LPCSTR utf8text);

        // �w�胂�W���[����javascript�R�[�h��ǂݍ��ށB�ǂݍ��߂Ȃ��ꍇ�͗�O�B
        void LoadJS(LPCSTR moduleName);

        // ���W���[���p�̃t�@�C����read open���AFILE*��Ԃ��B
        FILE* OpenReadModuleFile(LPCSTR fname);
    };
}