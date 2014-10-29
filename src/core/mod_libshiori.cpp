// mod_shiorilib.cpp : 組み込み関数：shiori関係
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"

//-------------------------------------------------------------
// SHIORI レスポンス応答を返します。
static duk_ret_t response(duk_context *ctx){
    // 初期化＆引数取得
    auto ghost = pasta::GetPasta(ctx);
    FUNC_START(ghost->cp);
    auto res = duk_to_string(ctx, 0);

    // GETレスポンス発行。
    ghost->Response(res);

    return 1;
}

//-------------------------------------------------------------
// デバッグ出力を行います。
static duk_ret_t debugstring(duk_context *ctx){
    // 初期化＆引数取得
    auto ghost = pasta::GetPasta(ctx);
    auto cp = ghost->cp;
#ifdef DEBUG
    std::string text(duk_to_string(ctx, 0));
    text += "\n";
    USES_CONVERSION;
    OutputDebugString(A2CW_CP(text.c_str(), cp));
#endif

    return 1;
}

//-------------------------------------------------------------
// モジュール登録
//-------------------------------------------------------------

static duk_function_list_entry funcs[] = {
        { "response", response, 1 },
        { "debugstring", debugstring, 1 },
        { NULL, NULL, 0 }
};

void pasta::Agent::InitModuleShiori(){
    RegModuleFuncs("libshiori", funcs);
}

// EOF