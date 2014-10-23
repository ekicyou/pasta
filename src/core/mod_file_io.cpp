// mod_file_io.cpp : 組み込み関数：fileアクセス関係
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "util.h"


static wchar_t * preLoadPath[] = {
    L"lib",
    L"js",
    L".",
    NULL,
};

// モジュールファイルを検索して最初に見つかったものを読み込みオープンして返す。
static FILE* openModuleFile(pasta::Agent* pasta, LPCWSTR fname){
    if (!fname) return NULL;

    const auto &loaddir = pasta->loaddir;
    for (int i = 0;; i++){
        const auto pre = preLoadPath[i];
        if (!pre) return NULL;
        auto p = std::tr2::sys::wpath(pasta->loaddir);
        p /= pre;
        p /= fname;

        // ファイルを開く
        auto f = _wfopen(p.string().c_str(), L"rb");
        if (f) return f;
    }
}


// ファイルをバッファとして読み込みます。
static duk_ret_t readfile(duk_context *ctx, pasta::Agent* pasta){
    USES_CONVERSION;
    OutputDebugString(L"[pasta::FileIO::readfile]開始！\n");

    // 引数
    auto fname = A2CW_UTF8(duk_to_string(ctx, 0));

    // ファイルオープン
    auto f = openModuleFile(pasta, fname);
    if (!f)goto error;

    // 読み込み
    if (fseek(f, 0, SEEK_END) != 0) goto error;
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) goto error;
    auto buf = duk_push_fixed_buffer(ctx, (size_t)len);
    auto got = fread(buf, 1, len, f);
    if (got != (size_t)len) goto error;

    if (f) fclose(f);
    return 1;

error:
    if (f) fclose(f);
    return DUK_RET_ERROR;
}


// ファイルをテキストとして読み込みます。
static duk_ret_t readtext(duk_context *ctx, pasta::Agent* pasta){
    USES_CONVERSION;
    OutputDebugString(L"[pasta::FileIO::readfile]開始！\n");

    // 引数
    auto fname = A2CW_UTF8(duk_to_string(ctx, 0));
    char *buf = NULL;

    // ファイルオープン
    auto f = openModuleFile(pasta, fname);
    if (!f)goto error;

    // 読み込み
    if (fseek(f, 0, SEEK_END) != 0) goto error;
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) goto error;
    buf = (char *)malloc(len);
    if (!buf)                       goto error;
    auto got = fread(buf, 1, len, f);
    if (got != (size_t)len)         goto error;
    duk_push_lstring(ctx, buf, got);

cleanup:
    free(buf);
    fclose(f);
    return 1;

error:
    if (buf) free(buf);
    if (f)   fclose(f);
    return DUK_RET_ERROR;
}


void pasta::Agent::InitFileIO(){

    auto module = "FileIO";
    auto &funcs = FileIOFuncs;

    funcs.push_back(Func(this, "readfile", readfile, 1));
    funcs.push_back(Func(this, "readtext", readtext, 1));

    RegModule(module, funcs);
}


