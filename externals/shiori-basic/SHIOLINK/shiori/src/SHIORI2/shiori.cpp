// ----------------------------------------------------------------------------
// �ʃv���Z�X�ʐMSHIORI SHIOLINK2.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"
#define SHIORI_API_IMPLEMENTS
#include "CShiori.h"

/**----------------------------------------------------------------------------
 * �O���[�o���C���X�^���X
 */
static HINSTANCE hinst;
static CRawShiori *shiori = NULL;


/**----------------------------------------------------------------------------
 * HGLOBAL�֌W
 */
// �����J��
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
 * �x Method / load
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
 * �x Method / unload
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
 * �x Method / request
 */
SHIORI_API HGLOBAL __cdecl request(HGLOBAL hGlobal_request, long& len)
{
   AutoGrobalFree autoFree(hGlobal_request);
   return shiori->Request(hGlobal_request, len);
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