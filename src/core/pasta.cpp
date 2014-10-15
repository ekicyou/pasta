// pasta.cpp : javascriptを呼び出すSHIORI本体。
//

#include "stdafx.h"
#include "pasta.h"

//-------------------------------------------------------------
// UTIL関数
//-------------------------------------------------------------

// std::string → std::wstring（ロケール依存）
inline std::wstring ToWideStr(const std::string &str)
{
    USES_CONVERSION;
    return A2CW(str.c_str());
}
// std::wstring → std::string（ロケール依存）
inline std::string ToMultStr(const std::wstring &wstr)
{
    USES_CONVERSION;
    return W2CA(wstr.c_str());
}


// std::string → std::wstring（コードページ指定）
inline std::wstring ToWideStr(const std::string &str, int cp)
{
    USES_CONVERSION;
    return A2CW_CP(str.c_str(), cp);
}
// std::wstring → std::string（コードページ指定）
inline std::string ToMultStr(const std::wstring &wstr, int cp)
{
    USES_CONVERSION;
    return W2CA_CP(wstr.c_str(), cp);
}


// エラーのコールバック関数（returnしない→例外に変換して戻す）
static void FatalFunc(duk_context *ctx, int code, const char *msg){
    USES_CONVERSION;
    std::wstring mes(L"duktape FATAL! code=(");
    mes += code;
    mes += L") ";
    mes += A2CW_CP(msg, CP_UTF8);

    OutputDebugString(L"[FatalFunc]");
    OutputDebugString(mes.c_str());
    OutputDebugString(L"\n");

    throw std::exception(W2CA(mes.c_str()));
}

#define duk_create_heap_pasta()  (duk_create_heap(NULL, NULL, NULL, NULL, FatalFunc))



//-------------------------------------------------------------
// メインルーチン
//-------------------------------------------------------------

void pasta::Agent::Run(){

}

// EOF