// stdafx.h : 標準のシステム インクルード ファイルのインクルード ファイル、または
// 参照回数が多く、かつあまり変更されない、プロジェクト専用のインクルード ファイル
// を記述します。
//

#pragma once

// 下で指定された定義の前に対象プラットフォームを指定しなければならない場合、以下の定義を変更してください。
// 異なるプラットフォームに対応する値に関する最新情報については、MSDN を参照してください。
#ifndef WINVER				// Windows 2000 以降のバージョンに固有の機能の使用を許可します。
#define WINVER 0x0500		// これを Windows の他のバージョン向けに適切な値に変更してください。
#endif

#ifndef _WIN32_WINNT		// Windows 2000 以降のバージョンに固有の機能の使用を許可します。              
#define _WIN32_WINNT 0x0500	// これを Windows の他のバージョン向けに適切な値に変更してください。
#endif						

#ifndef _WIN32_IE			// IE 6.0 以降のバージョンに固有の機能の使用を許可します。
#define _WIN32_IE 0x0500	// これを IE. の他のバージョン向けに適切な値に変更してください。
#endif

#define WIN32_LEAN_AND_MEAN		// Windows ヘッダーから使用されていない部分を除外します。
// Windows ヘッダー ファイル:
#include <windows.h>


#define _ATL_CSTRING_EXPLICIT_CONSTRUCTORS	// 一部の CString コンストラクタは明示的です。

#include <atlbase.h>
#include <atlstr.h>
#include <atlfile.h>
#include <atlcoll.h>

// TODO: プログラムに必要な追加ヘッダーをここで参照してください。
