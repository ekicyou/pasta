// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#ifndef SHIORI_H__
#define SHIORI_H__

#include <windows.h>

/* ----------------------------------------------------------------------------
 * 栞 I/F 関数の宣言があるだけ?.
 *
 * 使い方:
 *
 * #include <windows.h>
 * #define  SHIORI_API_IMPLEMENTS
 * #include "shiori.h"
 *
 * SHIORI_API_IMPLEMENTS を define しておくと,
 * 関数を dll-export する様になります.
 * define されていなければ, dll-import する様になります.
 *
 */

/* ----------------------------------------------------------------------------
 * import/export マクロ
 */
#ifndef SHIORI_API_IMPORT
#  ifdef __cplusplus
#    define SHIORI_API_IMPORT extern "C" __declspec(dllimport)
#  else
#    define SHIORI_API_IMPORT __declspec(dllimport)
#  endif
#endif

#ifndef SHIORI_API_EXPORT
#  ifdef __cplusplus
#    define SHIORI_API_EXPORT extern "C" __declspec(dllexport)
#  else
#    define SHIORI_API_EXPORT __declspec(dllexport)
#  endif
#endif

#ifndef SHIORI_API
#  ifdef SHIORI_API_IMPLEMENTS
#    define SHIORI_API SHIORI_API_EXPORT
#  else
#    define SHIORI_API SHIORI_API_IMPORT
#  endif
#endif

/* ----------------------------------------------------------------------------
 * 栞 メソッド
 */
SHIORI_API BOOL    __cdecl load(HGLOBAL    hGlobal_loaddir, long  loaddir_len);
SHIORI_API HGLOBAL __cdecl request(HGLOBAL hGlobal_request, long& len);
SHIORI_API BOOL    __cdecl unload(void);

#endif
