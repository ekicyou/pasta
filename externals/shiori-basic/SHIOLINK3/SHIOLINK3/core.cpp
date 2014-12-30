#include "stdafx.h"
#include "core.h"

//---------------------------------------------------------------------
// std::string �� HGLOBAL
static HGLOBAL AllocString(const std::string& test, long& len)
{
    HGLOBAL hText = GlobalAlloc(GMEM_FIXED, test.length());
    CopyMemory(hText, test.data(), test.length());
    len = (long)test.length();
    return hText;
}

//---------------------------------------------------------------------
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

//---------------------------------------------------------------------
// ���
Core::~Core()
{
}


//---------------------------------------------------------------------
// ������
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
// ������
HGLOBAL Core::request(HGLOBAL hGlobal_request, long& len){
    AutoGrobalFree autoFree(hGlobal_request);


}
