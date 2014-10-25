// pasta_agent.cpp : javascript���Ăяo��SHIORI�{�́B
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"

//-------------------------------------------------------------
// ���[�e�B���e�B�֐��F�����ϊ�
//-------------------------------------------------------------

// std::string �� std::wstring�i���P�[���ˑ��j
inline std::wstring ToWideStr(const std::string &str)
{
    USES_CONVERSION;
    return A2CW(str.c_str());
}
// std::wstring �� std::string�i���P�[���ˑ��j
inline std::string ToMultStr(const std::wstring &wstr)
{
    USES_CONVERSION;
    return W2CA(wstr.c_str());
}

// std::string �� std::wstring�i�R�[�h�y�[�W�w��j
inline std::wstring ToWideStr(const std::string &str, int cp)
{
    USES_CONVERSION;
    return A2CW_CP(str.c_str(), cp);
}
// std::wstring �� std::string�i�R�[�h�y�[�W�w��j
inline std::string ToMultStr(const std::wstring &wstr, int cp)
{
    USES_CONVERSION;
    return W2CA_CP(wstr.c_str(), cp);
}


//-------------------------------------------------------------
// ���[�e�B���e�B�֐��F�G���[�R�[���o�b�N
//-------------------------------------------------------------

// �G���[�̃R�[���o�b�N�֐��ireturn���Ȃ�����O�ɕϊ����Ė߂��j
static void FatalFunc(duk_context *ctx, int code, const char *msg){
    USES_CONVERSION;
    std::wstring mes(L"duktape FATAL! code=(");
    mes += code;
    mes += L") ";
    mes += A2CW_CP(msg, CP_UTF8);

    OutputDebugString(L"[FatalFunc]");
    OutputDebugString(mes.c_str());
    OutputDebugString(L"\n");

    throw std::exception(W2CA(mes.c_str()));
}

#define duk_create_heap_pasta()  (duk_create_heap(NULL, NULL, NULL, NULL, FatalFunc))

//-------------------------------------------------------------
// Load����
//-------------------------------------------------------------


void pasta::Agent::LoadAction(){
    USES_CONVERSION;

#ifdef DEBUG
    {
        std::wstring mes(L"[pasta::Agent::LoadAction](");
        mes.append(this->loaddir);
        mes.append(L")\n");
        OutputDebugString(mes.c_str());
    }
#endif

    // VM�쐬
    ctx = duk_create_heap_pasta();
    if (!ctx) { throw std::exception("FAIL duk_create_heap_default"); }
    SetPasta(ctx, this);

    // JavaScript�g�ݍ��݃I�u�W�F�N�g�̍쐬
    InitFileIO();
    InitShiori();

    // [shiori.js]�u�[�g�X�g���b�v


    OutputDebugString(L"[pasta::Agent::LoadAction]�I���I\n");
}

//-------------------------------------------------------------
// Unload����
//-------------------------------------------------------------

void pasta::Agent::UnLoadAction() {
    OutputDebugString(L"[pasta::Agent::UnLoadAction]START\n");

    // VM�̉��
    duk_destroy_heap(ctx);
    OutputDebugString(L"[pasta::Agent::UnLoadAction]END\n");
}


// ����^�C�~���O��Unload�����s����Ă��Ȃ���ΌĂяo���B
pasta::Agent::~Agent(){
    OutputDebugString(L"[pasta::Agent::destructor]START\n");
    UnLoad();
    OutputDebugString(L"[pasta::Agent::destructor]END\n");
}


//-------------------------------------------------------------
// Notify����
//-------------------------------------------------------------
void pasta::Agent::NotifyAction(const std::wstring& req){
    throw std::exception("not implment");
}

//-------------------------------------------------------------
// Get����
//-------------------------------------------------------------
void pasta::Agent::GetAction(const std::wstring& req){
    throw std::exception("not implment");
}

//-------------------------------------------------------------
// ���W���[���o�^
//-------------------------------------------------------------
void pasta::Agent::RegModuleFuncs(LPCSTR name, const duk_function_list_entry* funcs){
    duk_push_global_object(ctx);
    duk_push_object(ctx);
    duk_put_function_list(ctx, -1, funcs);
    duk_put_prop_string(ctx, -2, name);
    duk_pop(ctx);
}

// EOF