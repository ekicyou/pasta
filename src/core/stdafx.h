// stdafx.h : 標準のシステム インクルード ファイルのインクルード ファイル、または
// 参照回数が多く、かつあまり変更されない、プロジェクト専用のインクルード ファイル
// を記述します。
//

#pragma once

#include "targetver.h"

#define WIN32_LEAN_AND_MEAN             // Windows ヘッダーから使用されていない部分を除外します。



// TODO: プログラムに必要な追加ヘッダーをここで参照してください。
#include <xtal.h>
#include <xtal_macro.h>
#include <xtal_details.h>
#include <xtal_lib/xtal_cstdiostream.h>    // CStdioStdStreamLibのため
#include <xtal_lib/xtal_winthread.h>       // WinThreadLibのため
#include <xtal_lib/xtal_winfilesystem.h>   // WinFilesystemLibのため
#include <xtal_lib/xtal_chcode.h>          // SJISChCodeLibのため
#include <xtal_lib/xtal_errormessage.h>    // bind_error_messageのため
