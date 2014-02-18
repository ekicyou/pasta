#pragma once
#include "shiori.h"

class CRawShiori
{
private:
	HINSTANCE mInst;
	CString mDir;

protected:
	CRawShiori(const HINSTANCE hinst, const HGLOBAL hGlobal_loaddir, const long loaddir_len)
	{
		mInst = hinst;
		mDir = CString((LPCSTR)hGlobal_loaddir, loaddir_len);
	}

public:
	virtual ~CRawShiori(void)
	{
	}
	HINSTANCE GetShioriInstance(void) const { return mInst; }
	CString GetShioriDirectory(void) const { return mDir; }

	virtual HGLOBAL Request(const HGLOBAL hGlobal_request, long& len) = 0;
};
