#include "stdafx.h"
#include "core.h"

//---------------------------------------------------------------------
// std::string Å® HGLOBAL
static HGLOBAL AllocString(const std::string& test, long& len)
{
    HGLOBAL hText = GlobalAlloc(GMEM_FIXED, test.length());
    CopyMemory(hText, test.data(), test.length());
    len = (long)test.length();
    return hText;
}

//---------------------------------------------------------------------
// é©ìÆäJï˙
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
// âï˙
Core::~Core()
{
}


//---------------------------------------------------------------------
// èâä˙âª
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
// èâä˙âª
HGLOBAL Core::request(HGLOBAL hGlobal_request, long& len){
    AutoGrobalFree autoFree(hGlobal_request);


}
