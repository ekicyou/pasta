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
    USES_CONVERSION;
    auto ghost = pasta::GetPasta(ctx);
    FUNC_START(ghost->cp);

    std::wstring mes(L"duktape FATAL! code=(");
    mes += code;
    mes += L") ";
    mes += A2CW_CP(msg, ghost->cp);

    DEBUG_MESSAGE(mes.c_str());

    THROW_EX(W2CA(mes.c_str()));
}

#define duk_create_heap_pasta()  (duk_create_heap(NULL, NULL, NULL, NULL, FatalFunc))

//-------------------------------------------------------------
// �f�X�g���N�^�EUnload����
//-------------------------------------------------------------

void pasta::Agent::UnLoadAction() {
    FUNC_START(cp);
    USES_CONVERSION;

    // Shiori.unload�֐��̎��s
    duk_push_global_object(ctx);
    if (!duk_get_prop_string(ctx, -1, "Shiori"))    THROW_EX("object Shiori not found");
    if (!duk_get_prop_string(ctx, -1, "unload"))    THROW_EX("object Shiori.unload not found");
    /* -> [ global Shiori unload ] */
    if (!duk_is_function(ctx, -1))                  THROW_EX("object Shiori.unload is't function");
    duk_call(ctx, 0);
    /* -> [ global Shiori <retval> ] */
    duk_pop_3(ctx);

    // VM�̉��
    duk_destroy_heap(ctx);
}

// ����^�C�~���O��Unload�����s����Ă��Ȃ���ΌĂяo���B
pasta::Agent::~Agent(){
    FUNC_START(cp);
    UnLoad();
}

//-------------------------------------------------------------
// �R���X�g���N�^�ELoad���u
//-------------------------------------------------------------

pasta::Agent::Agent(const HINSTANCE hinst)
    :shiori::Agent(CP_UTF8, hinst)
{}

pasta::Agent::Agent(const HINSTANCE hinst, concurrency::Scheduler& scheduler)
    : shiori::Agent(CP_UTF8, hinst, scheduler)
{}
pasta::Agent::Agent(const HINSTANCE hinst, concurrency::ScheduleGroup& group)
    : shiori::Agent(CP_UTF8, hinst, group)
{}

void pasta::Agent::LoadAction(){
    FUNC_START(cp);
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
    LoadJS("loader.js");

    // [shiori.js]�u�[�g�X�g���b�v
    // �u�[�g�X�g���b�v�R�[�h�͍Œ��
    //  [Shiori.load(dir)]
    //  [Shiori.unload()]
    //  [Shiori.get(req)]
    //  [Shiori.notify(req)]
    // �̎������s�����Ƃ�O��Ƃ���B
    LoadJS("shiori.js");

    // Shiori.load�֐��̎��s
    duk_push_global_object(ctx);
    if (!duk_get_prop_string(ctx, -1, "Shiori"))    THROW_EX("object Shiori not found");
    if (!duk_get_prop_string(ctx, -1, "load"))      THROW_EX("object Shiori.load not found");
    /* -> [ global Shiori load ] */
    if (!duk_is_function(ctx, -1))                  THROW_EX("object Shiori.load is't function");
    duk_call(ctx, 0);   
    /* -> [ global Shiori <retval> ] */
    duk_pop_3(ctx);
}

//-------------------------------------------------------------
// Notify����
//-------------------------------------------------------------
void pasta::Agent::NotifyAction(const std::string& req){
    FUNC_START(cp);

    // Shiori.notify(req)�֐��̎��s
    duk_push_global_object(ctx);
    if (!duk_get_prop_string(ctx, -1, "Shiori"))    THROW_EX("object Shiori not found");
    if (!duk_get_prop_string(ctx, -1, "notify"))    THROW_EX("object Shiori.notify not found");
    /* -> [ global Shiori notify ] */
    if (!duk_is_function(ctx, -1))                  THROW_EX("object Shiori.notify is't function");
    duk_push_string(ctx, req.c_str());
    duk_call(ctx, 1);
    /* -> [ global Shiori <retval> ] */
    duk_pop_3(ctx);
}

//-------------------------------------------------------------
// Get����
//-------------------------------------------------------------
void pasta::Agent::GetAction(const std::string& req){
    FUNC_START(cp);

    // Shiori.get(req)�֐��̎��s
    duk_push_global_object(ctx);
    if (!duk_get_prop_string(ctx, -1, "Shiori"))    THROW_EX("object Shiori not found");
    if (!duk_get_prop_string(ctx, -1, "get"))       THROW_EX("object Shiori.get not found");
    /* -> [ global Shiori get ] */
    if (!duk_is_function(ctx, -1))                  THROW_EX("object Shiori.get is't function");
    duk_push_string(ctx, req.c_str());
    duk_call(ctx, 1);
    /* -> [ global Shiori <retval> ] */
    duk_pop_3(ctx);
}

//-------------------------------------------------------------
// ���W���[���o�^
//-------------------------------------------------------------
void pasta::Agent::RegModuleFuncs(LPCSTR name, const duk_function_list_entry* funcs){
    FUNC_START(cp);

    duk_push_global_object(ctx);
    duk_push_object(ctx);
    duk_put_function_list(ctx, -1, funcs);
    duk_put_prop_string(ctx, -2, name);
    duk_pop(ctx);
}

//-------------------------------------------------------------
// �X�N���v�g���[�h
//-------------------------------------------------------------

inline void LoadJSThrow(LPCSTR moduleName, LPCSTR what){
    std::string mes;
    mes += "FAIL (";
    mes += moduleName;
    mes += ") ";
    mes += what;
    ThrowStdException("pasta::Agent::LoadJS", mes.c_str());
}

// �w�胂�W���[����javascript�R�[�h��ǂݍ��ށB
void pasta::Agent::LoadJS(LPCSTR moduleName){
    FUNC_START(cp);

    // �ϐ�
    auto duk = ctx;

    // �t�@�C���I�[�v��
    auto f = OpenReadModuleFile(moduleName);
    if (!f) LoadJSThrow(moduleName, "not found");
    AUTO_CLOSE(f);

    // �ǂݍ���
    if (fseek(f, 0, SEEK_END) != 0) LoadJSThrow(moduleName, "seek error");
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) LoadJSThrow(moduleName, "seek error");
    auto src = (char *)malloc(len + 1);
    src[len] = NULL;
    if (!src)                       LoadJSThrow(moduleName, "malloc error");
    DISPOSE_LAMBDA([src](){free(src); });
    auto got = fread(src, 1, len, f);
    if (got != (size_t)len)         LoadJSThrow(moduleName, "read error");

    // �R���p�C��
    duk_push_string(duk, moduleName);
    DISPOSE_LAMBDA([duk](){duk_pop(duk); });

    if (duk_pcompile_lstring_filename(duk, 0, src, len) != 0) {
        std::string what("compile failed: ");
        what += duk_safe_to_string(duk, -1);
        LoadJSThrow(moduleName, what.c_str());
    }
    else {
        duk_call(duk, 0);      /* [ func ] -> [ result ] */
        auto rc = duk_safe_to_string(duk, -1);
        DEBUG_MESSAGE("script loaded");
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

std::string pasta::Agent::eval(LPCSTR utf8text){
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
    L"lib/ts",
    L"lib/js",
    L"js",
    L".",
    NULL,
};

FILE* pasta::Agent::OpenReadModuleFile(LPCSTR fname){
    FUNC_START(cp);
    USES_CONVERSION;

    if (!fname) return NULL;
    auto wfname = A2W_CP(fname, cp);

    for (int i = 0;; i++){
        const auto pre = preLoadPath[i];
        if (!pre) return NULL;
        auto p = std::tr2::sys::wpath(loaddir);
        p /= pre;
        p /= wfname;

        // �t�@�C�����J��
        auto text = p.string();
        auto strpath = text.c_str();
        auto f = _wfopen(strpath, L"rb");
        if (f){
#ifdef DEBUG
            std::wstring mes(L"open module path = [");
            mes += strpath;
            mes += L"]";
            DEBUG_MESSAGE(mes.c_str());
#endif
            return f;
        }
    }
}

// EOF