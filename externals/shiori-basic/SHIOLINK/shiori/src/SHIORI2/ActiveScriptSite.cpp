// Windows Script Host サポート
#include "stdafx.h"
#include "ActiveScriptSite.h"

CActiveScriptSite::CActiveScriptSite()
{
	m_dwRef = 1;
}

CActiveScriptSite::~CActiveScriptSite()
{
}

HRESULT STDMETHODCALLTYPE CActiveScriptSite::QueryInterface(REFIID riid, void **ppvObject)
{
	*ppvObject = NULL;
	return E_NOTIMPL;
}

ULONG CActiveScriptSite::AddRef(void)
{
	return ++m_dwRef;
}

ULONG CActiveScriptSite::Release(void) 
{
	if(--m_dwRef == 0) return 0;
	return m_dwRef;
}


// IActiveScriptSite methods...
HRESULT STDMETHODCALLTYPE CActiveScriptSite::GetLCID(LCID *plcid) 
{
	return S_OK;
}

HRESULT STDMETHODCALLTYPE CActiveScriptSite::GetItemInfo(
	LPCOLESTR pstrName, DWORD dwReturnMask, IUnknown **ppunkItem, ITypeInfo **ppti) 
{
	// Is it expecting an ITypeInfo?
	if(ppti) {
		// Default to NULL.
		*ppti = NULL;
		// Return if asking about ITypeInfo...
		if(dwReturnMask & SCRIPTINFO_ITYPEINFO)
			return TYPE_E_ELEMENTNOTFOUND;
	}
	// Is the engine passing an IUnknown buffer?
	if(ppunkItem) {
		// Default to NULL.
		*ppunkItem = NULL;
		// Is Script Engine looking for an IUnknown for our object?
		if(dwReturnMask & SCRIPTINFO_IUNKNOWN) {
			// Check for our object name...
			if (!_wcsicmp(L"MyObject", pstrName)) {
				// Provide our object.
				*ppunkItem = m_pUnkScriptObject;
				// Addref our object...
				m_pUnkScriptObject->AddRef();
			}
		}
	}
	return S_OK;
}

HRESULT STDMETHODCALLTYPE CActiveScriptSite::GetDocVersionString(BSTR *pbstrVersion) 
{
	return S_OK;
}

HRESULT STDMETHODCALLTYPE CActiveScriptSite::OnScriptTerminate(
	const VARIANT *pvarResult,
    const EXCEPINFO *pexcepInfo) 
{
	return S_OK;
}

HRESULT STDMETHODCALLTYPE CActiveScriptSite::OnStateChange(SCRIPTSTATE ssScriptState) 
{
	return S_OK;
}

HRESULT STDMETHODCALLTYPE CActiveScriptSite::OnScriptError(
            IActiveScriptError *pscriptError) 
{
	CComBSTR bstr;
	pscriptError->GetSourceLineText(&bstr);
	ATLTRACE2(
		_T("IActiveScriptSite::OnScriptError()\n")
		_T("Line: %s\n"), (LPCTSTR) bstr);
	return S_OK;
}

HRESULT STDMETHODCALLTYPE CActiveScriptSite::OnEnterScript(void) 
{
	return S_OK;
}

HRESULT STDMETHODCALLTYPE CActiveScriptSite::OnLeaveScript(void) 
{
	return S_OK;
}

// EOF