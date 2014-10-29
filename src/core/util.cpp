// util.cpp : �֗��֐��Ƃ��B
//

#include "stdafx.h"
#include "util.h"

//============================================================
// ������
//============================================================

// ���\�b�h���t����std::exception�𔭍s���܂��B
void ThrowStdException(LPCSTR funcname, LPCSTR what){
    std::string mes;
    mes += "[";
    mes += funcname;
    mes += "] ";
    mes += what;
    throw std::exception(mes.c_str());
}

//============================================================
// ���O�o��
//============================================================

FunctionInOutDebugLog::FunctionInOutDebugLog(const int cp, LPCSTR funcname)
    :cp(cp)
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

void FunctionInOutDebugLog::OutputRaw(LPCWSTR message){
    OutputDebugString(message);
}

void FunctionInOutDebugLog::OutputRaw(LPCSTR message){
    USES_CONVERSION;
    OutputRaw(A2CW(message));
}

void FunctionInOutDebugLog::OutputLog(LPCWSTR message){
    std::wstring text(funcName);
    text += message;
    text += L"\n";
    OutputRaw(text.c_str());
}

void FunctionInOutDebugLog::OutputLog(LPCSTR message){
    USES_CONVERSION;
    OutputLog(A2CW(message));
}

void FunctionInOutDebugLog::OutputLog(std::string& message){
    OutputLog(message.c_str());
}
void FunctionInOutDebugLog::OutputLog(std::wstring& message){
    OutputLog(message.c_str());
}

// EOF