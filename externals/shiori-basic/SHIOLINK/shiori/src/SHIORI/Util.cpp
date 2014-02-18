// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"

/**----------------------------------------------------------------------------
 * SHIORI 3.0 RESPONSE
 */
// RESPOSE 400: Bad Request
void CreateBatRequestResponse(std::string& response, const char* reason)
{
   response =
      "SHIORI/3.0 400 Bad Request\r\n"
      "Charset: UTF-8\r\n"
      "Sender: SHIORI-BASIC\r\n"
      "X-SHIORI-BASIC-Reason: ";
   response += reason;
   response += "\r\n\r\n";
}


/**----------------------------------------------------------------------------
 * カレントディレクトリ移動＆復帰
 */

Pushd::Pushd(const std::wstring& newdir):
      olddir()
{
   Poco::UnicodeConverter::toUTF16(Poco::Path::current(), olddir);
   SetCurrentDirectoryW(newdir.c_str());
}

Pushd::~Pushd()
{
   SetCurrentDirectoryW(olddir.c_str());
}

// EOF