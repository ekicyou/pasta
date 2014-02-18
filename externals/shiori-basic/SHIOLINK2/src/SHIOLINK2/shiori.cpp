// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK2.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"

#include "shiori.h"
#include "ShioriAPI.h"

/**----------------------------------------------------------------------------
 * グローバルインスタンス
 */
static HINSTANCE hinst;
static CShioriAPI *api = NULL;

/**----------------------------------------------------------------------------
 * HGLOBAL関係
 */
// 自動開放
class AutoGrobalFree
{
public:
	HGLOBAL m_hGlobal;
    AutoGrobalFree(HGLOBAL hGlobal) {
        m_hGlobal =hGlobal;
    }
    ~AutoGrobalFree() {
        GlobalFree(m_hGlobal);
    }
};

/* ----------------------------------------------------------------------------
 * unload_impl
 */
static BOOL unloadImpl(void)
{
    if (api != NULL) {
        delete api;
        api = NULL;
    }
    return true;
}

/* ----------------------------------------------------------------------------
 * 栞 Method / load
 */
SHIORI_API BOOL __cdecl load(HGLOBAL hGlobal_loaddir, long loaddir_len)
{
	SCOPE_LOG(_T(__FUNCTION__));
    AutoGrobalFree autoFree(hGlobal_loaddir);
    unloadImpl();
    CAtlString sLoaddir((LPSTR)hGlobal_loaddir, (int)loaddir_len);
    CPath loaddir(sLoaddir);
    ATLTRACE2(_T("[SHIORI::load] loaddir = %s\n"), (LPCTSTR)loaddir);
    api = new CShioriAPI(hinst, loaddir);
    return true;
}

/* ----------------------------------------------------------------------------
 * 栞 Method / unload
 */
SHIORI_API BOOL __cdecl unload(void)
{
	SCOPE_LOG(_T(__FUNCTION__));
    return unloadImpl();
}

/* ----------------------------------------------------------------------------
 * 栞 Method / request
 */
SHIORI_API HGLOBAL __cdecl request(HGLOBAL hGlobal_request, long& len)
{
	SCOPE_LOG(_T(__FUNCTION__));
    AutoGrobalFree autoFree(hGlobal_request);
    CharArray res;
	_ATLTRY
	{
	    bool rc = api->Request((LPCSTR) hGlobal_request, len, res);
	    if (!rc) {
	        CreateBatRequestResponse(res ,"SHIOLINK2 API return false");
		}
	}
	_ATLCATCH(e)
	{
		CString mes(_T("SHIOLINK2 WIN32 ERROR "));
		mes += GetWinErrMessage(e.m_hr);
	    CreateBatRequestResponse(res , mes);
	}
	_ATLCATCHALL()
	{
	    CreateBatRequestResponse(res ,"SHIOLINK2 UNNONE ERROR");
	}

    // 応答情報の作成
    HGLOBAL hRES =GlobalAlloc(GMEM_FIXED ,res.GetCount());
    CopyMemory(hRES ,res.GetData() ,res.GetCount());
    len =(long)res.GetCount();
    return hRES;
}

/**----------------------------------------------------------------------------
 * Dllエントリーポイント
 */

#ifdef _MANAGED
#pragma managed(push, off)
#endif

extern "C" __declspec(dllexport) BOOL WINAPI DllMain(
      HINSTANCE hinstDLL,  // DLL モジュールのハンドル
      DWORD fdwReason,     // 関数を呼び出す理由
      LPVOID lpvReserved   // 予約済み
   )
{
    switch (fdwReason) {
    case DLL_PROCESS_ATTACH: // プロセス接続
		hinst =hinstDLL;
		break;
	case DLL_PROCESS_DETACH: // プロセス切り離し
		unloadImpl();
		break;
	case DLL_THREAD_ATTACH:  // スレッド接続
		break;
	case DLL_THREAD_DETACH:  // スレッド切り離し
		break;
	}
	return true;
}

#ifdef _MANAGED
#pragma managed(pop)
#endif

// EOF