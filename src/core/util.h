#pragma once

#include <windows.h>
#include <string>


//-------------------------------------------------------------
// ユーティリティ関数：文字変換
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
