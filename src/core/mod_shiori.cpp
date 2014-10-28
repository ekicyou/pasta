// mod_file_io.cpp : 組み込み関数：fileアクセス関係
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"

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
// モジュール登録
//-------------------------------------------------------------

static duk_function_list_entry funcs[] = {
        { "response", response, 1 },
        { NULL, NULL, 0 }
};

void pasta::Agent::InitShiori(){
    RegModuleFuncs("shiori", funcs);
}

// EOF