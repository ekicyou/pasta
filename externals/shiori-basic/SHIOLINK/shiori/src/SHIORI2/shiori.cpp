// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK2.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"
#define SHIORI_API_IMPLEMENTS
#include "CShiori.h"

/**----------------------------------------------------------------------------
 * グローバルインスタンス
 */
static HINSTANCE hinst;
static CRawShiori *shiori = NULL;


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
 * 栞 Method / load
 */
SHIORI_API BOOL __cdecl load(HGLOBAL hGlobal_loaddir, long loaddir_len)
{
   AutoGrobalFree autoFree(hGlobal_loaddir);
   if(!shiori){
	   delete shiori;
	   shiori = NULL;
   }
   shiori = new CShiori(hinst, hGlobal_loaddir, loaddir_len);
   return shiori != NULL;
}

/* ----------------------------------------------------------------------------
 * 栞 Method / unload
 */
SHIORI_API BOOL __cdecl unload(void)
{
   if(!shiori){
	   delete shiori;
	   shiori = NULL;
   }
   return true;
}

/* ----------------------------------------------------------------------------
 * 栞 Method / request
 */
SHIORI_API HGLOBAL __cdecl request(HGLOBAL hGlobal_request, long& len)
{
   AutoGrobalFree autoFree(hGlobal_request);
   return shiori->Request(hGlobal_request, len);
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
      hinst =hinstDLL;
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