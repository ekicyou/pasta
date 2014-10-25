// mod_file_io.cpp : 組み込み関数：fileアクセス関係
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"



// SHIORI レスポンス応答を返します。
static duk_ret_t response(duk_context *ctx){
    USES_CONVERSION;
    OutputDebugString(L"[pasta::Shiori::response]開始！\n");

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
    duk_push_global_object(ctx);
    duk_push_object(ctx);
    duk_put_function_list(ctx, -1, funcs);
    duk_put_prop_string(ctx, -2, "Shiori");
    duk_pop(ctx);
}

// EOF