// mod_file_io.cpp : 組み込み関数：fileアクセス関係
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"



//-------------------------------------------------------------
// オープンされたFILE*をバッファとしてスタックに積む。
static duk_ret_t push_file_to_buffer(duk_context *ctx, FILE* f){
    duk_ret_t rc = DUK_RET_ERROR;

    if (fseek(f, 0, SEEK_END) != 0) goto clean;
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) goto clean;
    auto buf = duk_push_fixed_buffer(ctx, (size_t)len);
    auto got = fread(buf, 1, len, f);
    if (got != (size_t)len) goto clean;
    rc = 1;

clean:
    return rc;
}


//-------------------------------------------------------------
// オープンされたFILE*を文字列としてスタックに積む。
static duk_ret_t push_file_to_string(duk_context *ctx, FILE* f){
    duk_ret_t rc = DUK_RET_ERROR;
    char *buf = NULL;

    // 読み込み
    if (fseek(f, 0, SEEK_END) != 0) goto clean;
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) goto clean;
    buf = (char *)malloc(len);
    if (!buf)                       goto clean;
    auto got = fread(buf, 1, len, f);
    if (got != (size_t)len)         goto clean;
    duk_push_lstring(ctx, buf, got);
    rc = 1;

clean:
    if (buf) free(buf);
    return rc;
}





//-------------------------------------------------------------
// ファイルをバッファとして読み込みます。
static duk_ret_t readfile(duk_context *ctx){
    auto pasta = pasta::GetPasta(ctx);
    FUNC_START(pasta->cp);

    // 初期化＆引数取得
    duk_ret_t rc = DUK_RET_ERROR;
    auto fname = duk_to_string(ctx, 0);

    // ファイルオープン＆読み込み
    auto f = pasta->OpenReadModuleFile(fname);
    if (!f)goto clean;
    rc = push_file_to_buffer(ctx, f);

clean:
    if (f) fclose(f);
    return rc;
}


//-------------------------------------------------------------
// ファイルをテキストとして読み込みます。
static duk_ret_t readtext(duk_context *ctx){
    auto pasta = pasta::GetPasta(ctx);
    FUNC_START(pasta->cp);

    // 初期化＆引数取得
    duk_ret_t rc = DUK_RET_ERROR;
    auto fname = duk_to_string(ctx, 0);

    // ファイルオープン＆読み込み
    auto f = pasta->OpenReadModuleFile(fname);
    if (!f)goto clean;
    rc = push_file_to_string(ctx, f);

clean:
    if (f) fclose(f);
    return rc;
}

//-------------------------------------------------------------
// ユーザーファイルをテキストとして読み込みます。
static duk_ret_t readuser(duk_context *ctx){
    USES_CONVERSION;
    auto pasta = pasta::GetPasta(ctx);
    FUNC_START(pasta->cp);

    // 初期化＆引数取得
    duk_ret_t rc = DUK_RET_ERROR;
    auto fname = duk_to_string(ctx, 0);
    char *buf = NULL;

    // ファイルオープン
    auto f = pasta->OpenReadUserFile(fname);
    if (!f)goto clean;
    rc = push_file_to_string(ctx, f);

clean:
    if (f) fclose(f);
    return rc;
}


//-------------------------------------------------------------
// モジュール登録
//-------------------------------------------------------------

static duk_function_list_entry funcs[] = {
        { "readfile", readfile, 1 },
        { "readtext", readtext, 1 },
        { "readuser", readuser, 1 },
        { NULL, NULL, 0 }
};

void pasta::Agent::InitModuleFileIO(){
    RegModuleFuncs("libfs", funcs);
}

// EOF