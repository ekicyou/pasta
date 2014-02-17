// stdafx.h : 標準のシステム インクルード ファイルのインクルード ファイル、または
// 参照回数が多く、かつあまり変更されない、プロジェクト専用のインクルード ファイル
// を記述します。
//

#pragma once

#include "targetver.h"

#define WIN32_LEAN_AND_MEAN             // Windows ヘッダーから使用されていない部分を除外します。
// Windows ヘッダー ファイル:
#include <windows.h>


#define _ATL_CSTRING_EXPLICIT_CONSTRUCTORS      // 一部の CString コンストラクターは明示的です。

#include <atlbase.h>
#include <atlstr.h>

// TODO: プログラムに必要な追加ヘッダーをここで参照してください。

// PSL Setting  (https://github.com/Silica/PSL/wiki/compile-option)
#define PSL_OPTIMIZE_TAILCALL	// 末尾呼び出し最適化
#define PSL_OPTIMIZE_IN_COMPILE	// コンパイル時最適化(これ以下)を有効にする
#define PSL_THREAD_SAFE	// 上のオプションを無効にし、ローカルstatic変数を使わなくなる

#include "..\\..\\..\\..\\PSL\\PSL\\PSL.h"
