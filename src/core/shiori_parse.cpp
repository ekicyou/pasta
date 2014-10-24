// shiori_parse.cpp : SHIORI���N�G�X�g�̕���
//

#include "stdafx.h"
#include "shiori_parse.h"
#include "util.h"
#include <regex>


// SHIORI REQUEST�̐��K�\��

/*
GET SHIORI/3.0
key1: value1
key2: value2
...
(��s)
*/

/*
NOTIFY SHIORI/3.0
key1: value1
key2: value2
...
(��s)
*/

#define T(x)            L ## x
#define IDENTIFIER      T("([$a-zA-Z_][$0-9a-zA-Z_-]*)")
#define CRLF            T("\\r\\n")

#define SHIORI_VER      T("SHIORI/3.0")
#define SHIORI_HEADER   IDENTIFIER T(" ") SHIORI_VER CRLF
#define SHIORI_VALUE    IDENTIFIER T(": (.*?)") CRLF

#define SHIORI_REQUEST  T("^") SHIORI_HEADER T("(") SHIORI_VALUE T(")*") CRLF T("$")





// SHIORI3.0 REQUEST��Key��Value�̃y�A�z��ɕ������܂��B
const std::tr1::wcmatch shiori::matchShioriRequest(LPCWSTR text){

    // regex
    const std::tr1::wregex re(SHIORI_REQUEST);

    // match
    std::tr1::wcmatch match;
    std::tr1::regex_match(text, match, re);
    return match;
}
