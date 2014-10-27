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
static pasta::Agent *app;

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
 * �x Method / load
 */
SHIORI_API BOOL __cdecl load(HGLOBAL hGlobal_loaddir, long loaddir_len)
{
    USES_CONVERSION;
    AutoGrobalFree autoFree(hGlobal_loaddir);
    if (app != NULL) {
        delete app;
        app = NULL;
    }
    std::string loaddir((const char*)hGlobal_loaddir, (size_t)loaddir_len);
    auto wLoadDir = A2CW(loaddir.c_str());
    try{
        unload();
        app = new pasta::Agent(hinst);
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
    AutoGrobalFree autoFree(hGlobal_request);
    const std::string request((const char *)hGlobal_request, len);
    std::string response;
    try{
        const std::wstring wreq(A2CW_CP(request.c_str(), CP_UTF8));
        auto wres = app->Request(wreq);
    }
    catch (const std::exception& e){
        CreateBatRequestResponse(response, e.what(), app->cp);
    }
    catch (const char* e){
        CreateBatRequestResponse(response, e, app->cp);
    }
    catch (...){
        CreateBatRequestResponse(response, "Unnone Exception");
    }

    return AllocString(response, len);
}

// EOF