// mod_file_io.cpp : 組み込み関数：fileアクセス関係
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"



// SHIORI レスポンス応答を返します。
static duk_ret_t response(duk_context *ctx){
    FUNC_START;
    USES_CONVERSION;

    // 初期化＆引数取得
    auto ghost = pasta::GetPasta(ctx);
    auto res = A2CW_UTF8(duk_to_string(ctx, 0));

    ghost->Response(res);

    return 1;
}


//-------------------------------------------------------------
// モジュール登録
//-------------------------------------------------------------

static duk_function_list_entry funcs[] = {
        { "response", response, 1 },
        { NULL, NULL, 0 }
};

void pasta::Agent::InitShiori(){
    RegModuleFuncs("Shiori", funcs);
}

// EOF