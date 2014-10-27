// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"
#include "util.h"

/**----------------------------------------------------------------------------
 * SHIORI 3.0 RESPONSE
 */
// RESPOSE 400: Bad Request
std::string CreateBatRequestResponse(LPCSTR reason)
{
    std::string res =
        "SHIORI/3.0 400 Bad Request\r\n"
        "Charset: UTF-8\r\n"
        "Sender: PASTA\r\n"
        "X-PASTA-Reason: ";
    res += reason;
    res += "\r\n\r\n";
    return res;
}

std::string CreateBatRequestResponse(LPCSTR reason, const int cp){
    if (cp == CP_UTF8)return CreateBatRequestResponse(reason);

    USES_CONVERSION;
    auto message = W2A_CP(A2W_CP(reason, cp), CP_UTF8);
    return CreateBatRequestResponse(reason);
}

/**----------------------------------------------------------------------------
 * カレントディレクトリ移動＆復帰
 */

Pushd::Pushd(LPCTSTR newdir)
    :mOldDir()
{
    TCHAR buf[_MAX_PATH + 1];
    GetCurrentDirectory(sizeof(buf), buf);
    mOldDir = buf;
    BOOL rc = SetCurrentDirectory(newdir);
    if (!rc) AtlThrow(FAILED(ERROR_CURRENT_DIRECTORY));
}

Pushd::~Pushd()
{
    if (mOldDir.IsEmpty()) return;
    SetCurrentDirectory(mOldDir);
}

// EOF