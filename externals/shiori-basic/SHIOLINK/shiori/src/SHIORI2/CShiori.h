#pragma once
#include "RawShiori.h"
#include "IniFile.h"

class CShiori: public CRawShiori
{
private:
	UINT mCodePage;
	CString mCodePageText;
	CIniFile mIni;

public:
	CShiori(const HINSTANCE hinst, const HGLOBAL hGlobal_loaddir, const long loaddir_len)
		: CRawShiori(hinst, hGlobal_loaddir, loaddir_len)
	{
		SetCodePage(CP_UTF8);
		CString path = GetShioriDirectory();
		path += _T("SHIOLINK2.ini");
		mIni.Load(path, mCodePage);
		if(mIni.GetCodePage() != mCodePage){
			SetCodePage(mIni.GetCodePage());
			mIni.Load(path, mCodePage);
		}

	}

	virtual ~CShiori(void)
	{
	}

	void SetCodePage(const UINT codePage)
	{
		mCodePage = codePage; 
		CPINFOEX info;
		if(!GetCPInfoEx(GetCodePage(), 0, &info)){
			mCodePageText = _T("");
		}
		else{
			mCodePageText = info.CodePageName;
		}
	}
	UINT GetCodePage(void) const { return mCodePage; }
	void SetUTF8(void){ SetCodePage(CP_UTF8); }
	BOOL IsUTF8(void) const { return GetCodePage() == CP_UTF8; }
	void SetANSI(void){ SetCodePage(CP_ACP); }
	BOOL IsANSI(void) const { return GetCodePage() == CP_ACP; }
	CString GetCodePageName(void) const
	{
		return mCodePageText;
	}

	HGLOBAL AllocString(const CString& text, long& len) const
	{
		LPCSTR ansiStr = CT2CA(text, GetCodePage());
		len = lstrlenA(ansiStr);
		HGLOBAL hText =GlobalAlloc(GMEM_FIXED ,len);
		CopyMemory(hText ,ansiStr ,len);
		return hText;
	}

	CString CreateBatRequestResponse(LPCTSTR reason) const
	{
		CString response;
		response.Format(
			_T("SHIORI/3.0 400 Bad Request\r\n")
			_T("Charset: %s\r\n")
			_T("Sender: SHIORI-BASIC-2\r\n")
			_T("X-SHIORI-BASIC-Reason: %s\r\n")
			_T("\r\n")
			, GetCodePageName()
			, reason);
		return response;
	}

	HGLOBAL Request(const HGLOBAL hGlobal_request, long& len)
	{
		HGLOBAL res = NULL;
		if(res == NULL){
			CString text = CreateBatRequestResponse(_T("NOT RESPONSE"));
			res = AllocString(text, len);
		}
		return res;
	}

};
