// mod_file_io.cpp : 組み込み関数：fileアクセス関係
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "util.h"



// SHIORI レスポンス応答を返します。
static duk_ret_t response(duk_context *ctx, pasta::Agent* pasta){
    USES_CONVERSION;
    OutputDebugString(L"[pasta::Shiori::response]開始！\n");

    // 引数
    auto res = A2CW_UTF8(duk_to_string(ctx, 0));
    pasta->Response(res);

    return 1;
}




void pasta::Agent::InitShiori(){

    auto module = "Shiori";
    auto &funcs = ShioriFuncs;

    funcs.push_back(Func(this, "response", response, 1));

    RegModule(module, funcs);
}


