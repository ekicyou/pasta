// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"

#define SHIORI_API_IMPLEMENTS
#include "shiori.h"
#include "util.h"

/**----------------------------------------------------------------------------
 * グローバルインスタンス
 */
static HINSTANCE hinst;
static pasta::App *app;


/**----------------------------------------------------------------------------
 * HGLOBAL関係
 */
// 自動開放
class AutoGrobalFree
{
public:
	HGLOBAL m_hGlobal;
	AutoGrobalFree(HGLOBAL hGlobal) {
		m_hGlobal = hGlobal;
	}
	~AutoGrobalFree() {
		GlobalFree(m_hGlobal);
	}
};

// std::string → HGLOBAL
inline HGLOBAL AllocString(const std::string& test, long& len)
{
	HGLOBAL hText = GlobalAlloc(GMEM_FIXED, test.length());
	CopyMemory(hText, test.data(), test.length());
	len = (long)test.length();
	return hText;
}


/* ----------------------------------------------------------------------------
 * 栞 Method / load
 */
SHIORI_API BOOL __cdecl load(HGLOBAL hGlobal_loaddir, long loaddir_len)
{
	AutoGrobalFree autoFree(hGlobal_loaddir);
	if (app != NULL) {
		delete app;
		app = NULL;
	}
	std::string loaddir((const char*)hGlobal_loaddir, (size_t)loaddir_len);
	try{
		app = new pasta::App(hinst, loaddir);
		return true;
	}
	catch (const std::exception&){
		return false;
	}
	catch(...){
		return false;
	}
}

/* ----------------------------------------------------------------------------
 * 栞 Method / unload
 */
SHIORI_API BOOL __cdecl unload(void)
{
	try{
		if (app != NULL) {
			delete app;
			app = NULL;
		}
		return true;
	}
	catch (...){
		return false;
	}
}

/* ----------------------------------------------------------------------------
 * 栞 Method / request
 */
SHIORI_API HGLOBAL __cdecl request(HGLOBAL hGlobal_request, long& len)
{
	AutoGrobalFree autoFree(hGlobal_request);
	std::string request((const char *)hGlobal_request, len);
	std::string response;
	try{
		bool rc = app->request(request, response);
		if (!rc) {
			CreateBatRequestResponse(response, "Request return false");
		}
	}
	catch (const std::exception& e){
		CreateBatRequestResponse(response, e.what(), app->CP());
	}
	catch (const char* e){
		CreateBatRequestResponse(response, e, app->CP());
	}
	catch (...){
		CreateBatRequestResponse(response, "Unnone Exception");
	}

	return AllocString(response, len);
}

/**----------------------------------------------------------------------------
 * Dllエントリーポイント
 */
extern "C" __declspec(dllexport) BOOL WINAPI DllMain(
	HINSTANCE hinstDLL,  // DLL モジュールのハンドル
	DWORD fdwReason,     // 関数を呼び出す理由
	LPVOID lpvReserved   // 予約済み
	)
{
	switch (fdwReason) {
	case    DLL_PROCESS_ATTACH: // プロセス接続
		hinst = hinstDLL;
		break;

	case    DLL_PROCESS_DETACH: // プロセス切り離し
		unload();
		break;

	case    DLL_THREAD_ATTACH:  // スレッド接続
		break;

	case    DLL_THREAD_DETACH:  // スレッド切り離し
		break;
	}
	return true;
}

// EOF