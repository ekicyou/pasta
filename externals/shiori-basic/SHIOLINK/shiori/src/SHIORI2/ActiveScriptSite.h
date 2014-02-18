// Windows Script Host サポート
#pragma once
#include <activscp.h>

class CActiveScriptSite :
	public IActiveScriptSite
{
private:
	ULONG m_dwRef; // Reference count

public:
	IUnknown *m_pUnkScriptObject; // Pointer to your object that is exposed
                                  // to the script engine in GetItemInfo().
         
	// new delete methods...
	CActiveScriptSite();
	~CActiveScriptSite();

	// IUnknown methods...
	HRESULT STDMETHODCALLTYPE QueryInterface(REFIID riid, void **ppvObject);
	ULONG   STDMETHODCALLTYPE AddRef(void);
	ULONG   STDMETHODCALLTYPE Release(void);
         
	// IActiveScriptSite methods...
	HRESULT STDMETHODCALLTYPE GetLCID(LCID *plcid);
	HRESULT STDMETHODCALLTYPE GetItemInfo(LPCOLESTR pstrName, DWORD dwReturnMask, IUnknown **ppunkItem, ITypeInfo **ppti);
    HRESULT STDMETHODCALLTYPE GetDocVersionString(BSTR *pbstrVersion);
    HRESULT STDMETHODCALLTYPE OnScriptTerminate(const VARIANT *pvarResult, const EXCEPINFO *pexcepInfo);
    HRESULT STDMETHODCALLTYPE OnStateChange(SCRIPTSTATE ssScriptState);
    HRESULT STDMETHODCALLTYPE OnScriptError(IActiveScriptError *pscriptError);
    HRESULT STDMETHODCALLTYPE OnEnterScript(void);
    HRESULT STDMETHODCALLTYPE OnLeaveScript(void);
};

// EOF