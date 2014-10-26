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

    // �u�[�g�X�g���b�v[loader.js]
    // �u�[�g�X�g���b�v�R�[�h��[Duktape.modSearch]���������邱��
    LoadJS(L"loader.js");

    // [shiori.js]�u�[�g�X�g���b�v
    // �u�[�g�X�g���b�v�R�[�h�͍Œ��
    //  [Shiori.load(dir)]
    //  [Shiori.unload()]
    //  [Shiori.get(req)]
    //  [Shiori.notify(req)]
    // �̎������s�����Ƃ�O��Ƃ���B
    LoadJS(L"shiori.js");

    // load����[ Shiori.load(dir) �̌Ăяo��]


}

//-------------------------------------------------------------
// Unload����
//-------------------------------------------------------------

void pasta::Agent::UnLoadAction() {
    FUNC_START;
    USES_CONVERSION;

    // Unload����[ Shiori.unload() �̌Ăяo��]

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

    // Notify����[ Shiori.notify(req) �̌Ăяo��]

}

//-------------------------------------------------------------
// Get����
//-------------------------------------------------------------
void pasta::Agent::GetAction(const std::wstring& req){
    FUNC_START;

    NOT_IMPLMENT;
    // Get����[ Shiori.get(req) �̌Ăяo��]
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

//-------------------------------------------------------------
// �X�N���v�g���[�h
//-------------------------------------------------------------

inline void LoadJSThrow(LPCWSTR moduleName, LPCWSTR what){
    std::wstring mes;
    mes += L"FAIL (";
    mes += moduleName;
    mes += L") ";
    mes += what;
    ThrowStdException("pasta::Agent::LoadJS", mes.c_str());
}


// �w�胂�W���[����javascript�R�[�h��ǂݍ��ށB
void pasta::Agent::LoadJS(LPCWSTR moduleName){
    FUNC_START;
    USES_CONVERSION;

    // �ϐ�
    auto duk = ctx;

    // �t�@�C���I�[�v��
    auto f = OpenReadModuleFile(moduleName);
    if (!f) LoadJSThrow(moduleName, L"not found");
    AUTO_CLOSE(f);

    // �ǂݍ���
    if (fseek(f, 0, SEEK_END) != 0) LoadJSThrow(moduleName, L"seek error");
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) LoadJSThrow(moduleName, L"seek error");
    auto src = (char *)malloc(len+1);
    src[len] = NULL;
    if (!src)                       LoadJSThrow(moduleName, L"malloc error");
    DISPOSE_LAMBDA([src](){free(src); });
    auto got = fread(src, 1, len, f);
    if (got != (size_t)len)         LoadJSThrow(moduleName, L"read error");

    // �R���p�C��
    duk_push_string(duk, W2A_CP(moduleName, CP_UTF8));
    DISPOSE_LAMBDA([duk](){duk_pop(duk); });


    if (duk_pcompile_lstring_filename(duk, 0, src, len) != 0) {
        std::wstring what(L"compile failed: ");
        what += A2CW_CP(duk_safe_to_string(duk, -1), CP_UTF8);
        LoadJSThrow(moduleName, what.c_str());
    }
    else {
        duk_call(duk, 0);      /* [ func ] -> [ result ] */
        auto rc = duk_safe_to_string(duk, -1);
        DEBUG_MESSAGE(rc);
    }

    return;



}

//============================================================
// eval
//============================================================

static int eval_raw(duk_context *ctx) {
    duk_eval(ctx);
    return 1;
}

static int tostring_raw(duk_context *ctx) {
    duk_to_string(ctx, -1);
    return 1;
}

std::string pasta::Agent::eval(const char * utf8text){
    duk_push_string(ctx, utf8text);
    duk_safe_call(ctx, eval_raw, 1 /*nargs*/, 1 /*nrets*/);
    duk_safe_call(ctx, tostring_raw, 1 /*nargs*/, 1 /*nrets*/);
    auto text = duk_get_string(ctx, -1);
    std::string rc(text);
    duk_pop(ctx);
    return rc;
}


//-------------------------------------------------------------
// IO
//-------------------------------------------------------------
static wchar_t * preLoadPath[] = {
    L"duktape",
    L"modules",
    L"lib",
    L"js",
    L".",
    NULL,
};

FILE* pasta::Agent::OpenReadModuleFile(LPCWSTR fname){
    FUNC_START;

    if (!fname) return NULL;
    for (int i = 0;; i++){
        const auto pre = preLoadPath[i];
        if (!pre) return NULL;
        auto p = std::tr2::sys::wpath(loaddir);
        p /= pre;
        p /= fname;

        // �t�@�C�����J��
        auto f = _wfopen(p.string().c_str(), L"rb");
        if (f) return f;
    }
}


// EOF