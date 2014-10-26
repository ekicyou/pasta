// pasta_agent.cpp : javascript���Ăяo��SHIORI�{�́B
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"

//-------------------------------------------------------------
// ���[�e�B���e�B�֐��F�G���[�R�[���o�b�N
//-------------------------------------------------------------

// �G���[�̃R�[���o�b�N�֐��ireturn���Ȃ�����O�ɕϊ����Ė߂��j
static void FatalFunc(duk_context *ctx, int code, const char *msg){
    FUNC_START;
    USES_CONVERSION;

    std::wstring mes(L"duktape FATAL! code=(");
    mes += code;
    mes += L") ";
    mes += A2CW_CP(msg, CP_UTF8);

    DEBUG_MESSAGE(mes.c_str());

    THROW_EX(W2CA(mes.c_str()));
}

#define duk_create_heap_pasta()  (duk_create_heap(NULL, NULL, NULL, NULL, FatalFunc))

//-------------------------------------------------------------
// Load����
//-------------------------------------------------------------


void pasta::Agent::LoadAction(){
    FUNC_START;
    USES_CONVERSION;

#ifdef DEBUG
    {
        std::wstring mes(L"loaddir = [");
        mes.append(this->loaddir);
        mes.append(L"]");
        DEBUG_MESSAGE(mes.c_str());
    }
#endif

    // VM�쐬
    ctx = duk_create_heap_pasta();
    if (!ctx) { THROW_EX("FAIL duk_create_heap_default"); }
    SetPasta(ctx, this);

    // JavaScript�g�ݍ��݃I�u�W�F�N�g�̍쐬
    InitFileIO();
    InitShiori();

    // [shiori.js]�u�[�g�X�g���b�v

}

//-------------------------------------------------------------
// Unload����
//-------------------------------------------------------------

void pasta::Agent::UnLoadAction() {
    FUNC_START;
    USES_CONVERSION;

    // VM�̉��
    duk_destroy_heap(ctx);
}


// ����^�C�~���O��Unload�����s����Ă��Ȃ���ΌĂяo���B
pasta::Agent::~Agent(){
    FUNC_START;
    UnLoad();
}


//-------------------------------------------------------------
// Notify����
//-------------------------------------------------------------
void pasta::Agent::NotifyAction(const std::wstring& req){
    FUNC_START;

    NOT_IMPLMENT;
}

//-------------------------------------------------------------
// Get����
//-------------------------------------------------------------
void pasta::Agent::GetAction(const std::wstring& req){
    FUNC_START;

    NOT_IMPLMENT;
}

//-------------------------------------------------------------
// ���W���[���o�^
//-------------------------------------------------------------
void pasta::Agent::RegModuleFuncs(LPCSTR name, const duk_function_list_entry* funcs){
    FUNC_START;

    duk_push_global_object(ctx);
    duk_push_object(ctx);
    duk_put_function_list(ctx, -1, funcs);
    duk_put_prop_string(ctx, -2, name);
    duk_pop(ctx);
}

// EOF