// ----------------------------------------------------------------------------
// �ʃv���Z�X�ʐMSHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"

#define SHIORI_API_IMPLEMENTS
#include "shiori.h"
#include "util.h"

/**----------------------------------------------------------------------------
 * HGLOBAL�֌W
 */
// �����J��
class AutoGrobal
{
public:
    HGLOBAL m_hGlobal;
    AutoGrobal(HGLOBAL hGlobal) {
        m_hGlobal = hGlobal;
    }
    ~AutoGrobal() {
        GlobalFree(m_hGlobal);
    }
};

// std::string �� HGLOBAL
inline HGLOBAL AllocString(const std::string& test, long& len)
{
    HGLOBAL hText = GlobalAlloc(GMEM_FIXED, test.length());
    CopyMemory(hText, test.data(), test.length());
    len = (long)test.length();
    return hText;
}

/**----------------------------------------------------------------------------
* �O���[�o���C���X�^���X
*/
static HINSTANCE hinst;
static std::unique_ptr<pasta::Agent> app;

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
        hinst = hinstDLL;
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

/* ----------------------------------------------------------------------------
* �x Method / unload
*/
SHIORI_API BOOL __cdecl unload(void)
{
    try{
        if (app != nullptr)app->UnLoad();
        app = NULL;
        return true;
    }
    catch (...){
        return false;
    }
}

/* ----------------------------------------------------------------------------
 * �x Method / load
 */
SHIORI_API BOOL __cdecl load(HGLOBAL hGlobal_loaddir, long loaddir_len)
{
    USES_CONVERSION;
    AutoGrobal auto_loaddir(hGlobal_loaddir);
    std::string loaddir((const char*)hGlobal_loaddir, (size_t)loaddir_len);
    auto wLoadDir = A2CW(loaddir.c_str());
    try{
        unload();
        app = std::make_unique<pasta::Agent>(hinst);
        app->Load(wLoadDir);
        return true;
    }
    catch (const std::exception&){
        return false;
    }
    catch (...){
        return false;
    }
}

/* ----------------------------------------------------------------------------
 * �x Method / request
 */
SHIORI_API HGLOBAL __cdecl request(HGLOBAL hGlobal_request, long& len)
{
    USES_CONVERSION;
    AutoGrobal auto_request(hGlobal_request);
    const std::string req((const char *)hGlobal_request, len);
    try{
        auto res = app->Request(req);
        return AllocString(res, len);
    }
    catch (const std::exception& e){
        auto res = CreateBatRequestResponse(e.what(), app->cp);
        return AllocString(res, len);
    }
    catch (const char* e){
        auto res = CreateBatRequestResponse(e, app->cp);
        return AllocString(res, len);
    }
    catch (...){
        auto res = CreateBatRequestResponse("Unnone Exception");
        return AllocString(res, len);
    }
}

// EOF