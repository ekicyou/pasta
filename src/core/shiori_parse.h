#pragma once

#include <windows.h>
#include <string>
#include <vector>


namespace shiori{

    class KeyValuePair{
    public:
        const std::wstring key;
        const std::wstring value;

        KeyValuePair(const std::wstring &key, const std::wstring &value)
            :key(key), value(value){}
    };

    // SHIORI3.0 REQUEST�𐳋K�\���ŉ�͂��A���ʂ�Ԃ��܂��B
    const std::tr1::wcmatch matchShioriRequest(LPCWSTR text);



}