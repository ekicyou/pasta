// ----------------------------------------------------------------------------
// PLUGIN-SHIORI SHIOLINK3.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"
#define SHIORI_API_IMPLEMENTS
#include "shiori.h"
#include "core.h"

#include <memory>

/**----------------------------------------------------------------------------
 * �O���[�o���C���X�^���X
 */
static HINSTANCE hinst;
static std::unique_ptr<Core> core;


/* ----------------------------------------------------------------------------
 * �x Method / load
 */
SHIORI_API BOOL __cdecl load(HGLOBAL hGlobal_loaddir, long loaddir_len)
{
   BOOL rc;
   core.reset(new Core(hinst, hGlobal_loaddir, loaddir_len, rc));
   return rc;
}

/* ----------------------------------------------------------------------------
 * �x Method / unload
 */
SHIORI_API BOOL __cdecl unload(void)
{
    core.reset();
    return TRUE;
}

/* ----------------------------------------------------------------------------
 * �x Method / request
 */
SHIORI_API HGLOBAL __cdecl request(HGLOBAL hGlobal_request, long& len)
{
    return core->request(hGlobal_request, len);
}

/**----------------------------------------------------------------------------
 * Dll�G���g���[�|�C���g
 */
extern "C" __declspec(dllexport) BOOL WINAPI DllMain(
      HINSTANCE hinstDLL,  // DLL ���W���[���̃n���h��
      DWORD fdwReason,     // �֐����Ăяo�����R
      LPVOID lpvReserved   // �\��ς�
   )
{
   switch (fdwReason) {
   case    DLL_PROCESS_ATTACH: // �v���Z�X�ڑ�
      hinst =hinstDLL;
      break;

   case    DLL_PROCESS_DETACH: // �v���Z�X�؂藣��
      unload();
      break;

   case    DLL_THREAD_ATTACH:  // �X���b�h�ڑ�
      break;

   case    DLL_THREAD_DETACH:  // �X���b�h�؂藣��
      break;
   }
   return true;
}

// EOF