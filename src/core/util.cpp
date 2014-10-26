// util.cpp : 便利関数とか。
//

#include "stdafx.h"
#include "util.h"

//============================================================
// 初期化
//============================================================

// メソッド名付きでstd::exceptionを発行します。
void ThrowStdException(LPCSTR funcname, LPCSTR what){
    std::string mes;
    mes += "[";
    mes += funcname;
    mes += "] ";
    mes += what;
    throw std::exception(mes.c_str());
}

void ThrowStdException(LPCSTR funcname, LPCWSTR what){
    USES_CONVERSION;
    auto text = W2CA_CP(what, CP_UTF8);
    ThrowStdException(funcname, text);
}


//============================================================
// ログ出力
//============================================================

FunctionInOutDebugLog::FunctionInOutDebugLog(LPCSTR funcname)
{
    USES_CONVERSION;
    funcName += L"[";
    funcName += A2CW(funcname);
    funcName += L"] ";

    OutputLog(">>> START");
}

FunctionInOutDebugLog::   ~FunctionInOutDebugLog(){
    OutputLog("<<< END");
}

void FunctionInOutDebugLog::OutputLog(LPCSTR message){
    USES_CONVERSION;
    OutputLog(A2CW(message));
}

void FunctionInOutDebugLog::OutputLog(LPCWSTR message){
    std::wstring text(funcName);
    text += message;
    text += L"\n";
    OutputDebugString(text.c_str());
}
