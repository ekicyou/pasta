// ----------------------------------------------------------------------------
// �ʃv���Z�X�ʐMSHIORI SHIOLINK3.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#pragma once
#include <windows.h>

/* ----------------------------------------------------------------------------
* import/export �}�N��
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
* �x ���\�b�h
*/
SHIORI_API BOOL    __cdecl load(HGLOBAL    hGlobal_loaddir, long  loaddir_len);
SHIORI_API HGLOBAL __cdecl request(HGLOBAL hGlobal_request, long& len);
SHIORI_API BOOL    __cdecl unload(void);
// EOF