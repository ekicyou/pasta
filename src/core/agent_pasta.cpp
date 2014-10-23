// pasta_agent.cpp : javascript���Ăяo��SHIORI�{�́B
//

#include "stdafx.h"
#include "agent_pasta.h"

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

    // �g�ݍ��݃I�u�W�F�N�g�̍쐬
    InitFileIO();


    OutputDebugString(L"[pasta::Agent::LoadAction]�I���I\n");
}

//-------------------------------------------------------------
// Unload����
//-------------------------------------------------------------

void pasta::Agent::UnLoadAction() {
    OutputDebugString(L"[pasta::Agent::Unload]�J�n�I\n");


    // VM�̉��
    duk_destroy_heap(ctx);
    OutputDebugString(L"[pasta::Agent::Unload]�I���I\n");
}


// ����^�C�~���O��Unload�����s����Ă��Ȃ���ΌĂяo���B
pasta::Agent::~Agent(){
    UnLoad();
}


//-------------------------------------------------------------
// Notify����
//-------------------------------------------------------------
void  pasta::Agent::NotifyAction(const std::wstring& req){
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

void pasta::Agent::RegModule(const char* moduleName, const std::vector<Func> &funcs){
    auto entrys = std::vector<duk_function_list_entry>(funcs.size() + 1);
    auto count = funcs.size();
    for (int i = 0; i < count; i++){
        auto &f = funcs[i];
        entrys[i].key = f.key;
        entrys[i].nargs = f.nargs;
        auto v = f.func.target<duk_ret_t(*)(duk_context *ctx)>();
        entrys[i].value = (duk_c_function)v;
    }
    entrys[count].key = NULL;
    entrys[count].nargs = NULL;
    entrys[count].value = NULL;

    duk_push_global_object(ctx);
    duk_push_object(ctx);  /* -> [ ... global obj ] */
    duk_put_function_list(ctx, -1, entrys.data());
    duk_put_prop_string(ctx, -2, moduleName);  /* -> [ ... global ] */
    duk_pop(ctx);
}



// EOF