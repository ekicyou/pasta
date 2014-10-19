// mod_file_io.cpp : 組み込み関数：fileアクセス関係
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "util.h"



static duk_ret_t loadModuleFile(duk_context *ctx, pasta::Agent* pasta){
    throw std::exception("not_impl");
}



void pasta::Agent::InitFileIO(){

    auto funcs = std::vector<Func>();

    auto module = "FileIO";

    funcs.push_back(Func(this, "loadModuleFile", loadModuleFile, 2));


    // 登録
    auto entrys = std::vector<duk_function_list_entry>(funcs.size() + 1);
    auto count = funcs.size();
    for (int i = 0; i < count; i++){
        Func& f = funcs[i];
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
    duk_put_prop_string(ctx, -2, module);  /* -> [ ... global ] */
    duk_pop(ctx);
}


