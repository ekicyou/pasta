// pasta_agent.cpp : javascriptを呼び出すSHIORI本体。
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"

//-------------------------------------------------------------
// ユーティリティ関数：エラーコールバック
//-------------------------------------------------------------

// エラーのコールバック関数（returnしない→例外に変換して戻す）
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
// Load処理
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

    // VM作成
    ctx = duk_create_heap_pasta();
    if (!ctx) { THROW_EX("FAIL duk_create_heap_default"); }
    SetPasta(ctx, this);

    // JavaScript組み込みオブジェクトの作成
    InitFileIO();
    InitShiori();

    // [shiori.js]ブートストラップ

}

//-------------------------------------------------------------
// Unload処理
//-------------------------------------------------------------

void pasta::Agent::UnLoadAction() {
    FUNC_START;
    USES_CONVERSION;

    // VMの解放
    duk_destroy_heap(ctx);
}


// 解放タイミングでUnloadが実行されていなければ呼び出す。
pasta::Agent::~Agent(){
    FUNC_START;
    UnLoad();
}


//-------------------------------------------------------------
// Notify処理
//-------------------------------------------------------------
void pasta::Agent::NotifyAction(const std::wstring& req){
    FUNC_START;

    NOT_IMPLMENT;
}

//-------------------------------------------------------------
// Get処理
//-------------------------------------------------------------
void pasta::Agent::GetAction(const std::wstring& req){
    FUNC_START;

    NOT_IMPLMENT;
}

//-------------------------------------------------------------
// モジュール登録
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