#include "stdafx.h"
#include "core.h"

//---------------------------------------------------------------------
// std::string → HGLOBAL
static HGLOBAL AllocString(const std::string& test, long& len)
{
    HGLOBAL hText = GlobalAlloc(GMEM_FIXED, test.length());
    CopyMemory(hText, test.data(), test.length());
    len = (long)test.length();
    return hText;
}

//---------------------------------------------------------------------
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

//---------------------------------------------------------------------
// 解放
Core::~Core()
{
}


//---------------------------------------------------------------------
// 初期化
Core::Core()
    :concurrency::agent()
{
}


//---------------------------------------------------------------------
// load
BOOL Core::load(HINSTANCE hinst, HGLOBAL hGlobal_loaddir, long loaddir_len){
    AutoGrobalFree autoFree(hGlobal_loaddir);


}


//---------------------------------------------------------------------
// unload
BOOL Core::unload(){


}


//---------------------------------------------------------------------
// 初期化
HGLOBAL Core::request(HGLOBAL hGlobal_request, long& len){
    AutoGrobalFree autoFree(hGlobal_request);


}
