// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------

#pragma once

// 下で指定された定義の前に対象プラットフォームを指定しなければならない場合、以下の定義を変更してください。
// 異なるプラットフォームに対応する値に関する最新情報については、MSDN を参照してください。
#ifndef WINVER    // Windows 2000 以降のバージョンに固有の機能の使用を許可します。
#define WINVER 0x0500  // これを Windows の他のバージョン向けに適切な値に変更してください。
#endif

#ifndef _WIN32_WINNT  // Windows 2000 以降のバージョンに固有の機能の使用を許可します。                   
#define _WIN32_WINNT 0x0500 // これを Windows の他のバージョン向けに適切な値に変更してください。
#endif

#ifndef _WIN32_WINDOWS  // Windows 98 以降のバージョンに固有の機能の使用を許可します。
#define _WIN32_WINDOWS 0x0410 // これを Windows Me またはそれ以降のバージョン向けに適切な値に変更してください。
#endif

#ifndef _WIN32_IE   // IE 6.0 以降のバージョンに固有の機能の使用を許可します。
#define _WIN32_IE 0x0600 // これを IE. の他のバージョン向けに適切な値に変更してください。
#endif

#define WIN32_LEAN_AND_MEAN  // Windows ヘッダーから使用されていない部分を除外します。
// Windows ヘッダー ファイル:
#include <windows.h>



// TODO: プログラムに必要な追加ヘッダーをここで参照してください。
#include <atlbase.h>
#include <atlconv.h>
#include <string>
#include <sstream>
#include <fstream>
#include <queue>
#include <algorithm>

#define POCO_STATIC
#include <Poco/String.h>
#include <Poco/UnicodeConverter.h>
#include <Poco/Path.h>
#include <Poco/UUIDGenerator.h>
#include <Poco/DateTime.h>
#include <Poco/AutoPtr.h>
#include <Poco/Buffer.h>

#define SHIORI_API_IMPLEMENTS
#include "shiori.h"
#include "ProcessProxy.h"
#include "Util.h"

#define SHIORI_EXE_NAME  "SHIORI.exe"

// EOF