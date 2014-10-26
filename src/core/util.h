#pragma once

#include <windows.h>
#include <string>


//-------------------------------------------------------------
// ���[�e�B���e�B�֐��F�����ϊ�
//-------------------------------------------------------------

// std::string �� std::wstring�i���P�[���ˑ��j
inline std::wstring ToWideStr(const std::string &str)
{
    USES_CONVERSION;
    return A2CW(str.c_str());
}
// std::wstring �� std::string�i���P�[���ˑ��j
inline std::string ToMultStr(const std::wstring &wstr)
{
    USES_CONVERSION;
    return W2CA(wstr.c_str());
}

// std::string �� std::wstring�i�R�[�h�y�[�W�w��j
inline std::wstring ToWideStr(const std::string &str, int cp)
{
    USES_CONVERSION;
    return A2CW_CP(str.c_str(), cp);
}
// std::wstring �� std::string�i�R�[�h�y�[�W�w��j
inline std::string ToMultStr(const std::wstring &wstr, int cp)
{
    USES_CONVERSION;
    return W2CA_CP(wstr.c_str(), cp);
}

#define A2CW_UTF8(_str_) (A2CW_CP(_str_,CP_UTF8))


//-------------------------------------------------------------
// ���[�e�B���e�B�֐��F��O�o��
//-------------------------------------------------------------

// ���\�b�h���t����std::exception�𔭍s���܂��B
void ThrowStdException(LPCSTR funcname, LPCSTR what);


// ���\�b�h���t����std::exception�𔭍s���܂��B
#define THROW_EX(what)  ThrowStdException(__FUNCTION__,what)

// ������
#define NOT_IMPLMENT    THROW_EX("not implment") 



//-------------------------------------------------------------
// ���[�e�B���e�B�֐��F���O�o�͊֌W
//-------------------------------------------------------------

class FunctionInOutDebugLog{
public:
    FunctionInOutDebugLog(LPCSTR funcname);
    ~FunctionInOutDebugLog();

    void OutputLog(LPCSTR message);
    void OutputLog(LPCWSTR message);

private:
    std::wstring funcName;
};

#ifdef DEBUG
    #define FUNC_START      FunctionInOutDebugLog __func_start_debuglog__(__FUNCTION__);
#else
    #define FUNC_START      ;
#endif


#ifdef DEBUG
    #define DEBUG_MESSAGE(mes)  __func_start_debuglog__.OutputLog(mes);
#else
    #define DEBUG_MESSAGE(mes)  ;
#endif



