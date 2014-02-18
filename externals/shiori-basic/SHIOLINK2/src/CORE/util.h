// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK2.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#pragma once

class CharArray : public CAtlArray<CHAR>{};

void CreateBatRequestResponse(CharArray &res, LPCSTR  msg);
void CreateBatRequestResponse(CharArray &res, LPCTSTR msg);

CString GetWinErrMessage(const HRESULT hr);

class Pushd
{
private:
	CString mOldDir;

public:
	Pushd(LPCTSTR newdir);
	~Pushd();
};

class ScopeLog
{
private:
	CString mFuncName;

public:
	ScopeLog(LPCTSTR fname);
	~ScopeLog();
};


#ifdef _DEBUG

#define LOG(fname,...)                              \
{                                                   \
    CString _log_fmt;                               \
    _log_fmt.Format(__VA_ARGS__);                   \
    ATLTRACE2(_T("[%s]  %s\n"), fname, _log_fmt);   \
}

#define SCOPE_LOG(fname) ScopeLog __scolelog__(fname);

#else

#define LOG(fname,fmt,...)
#define SCOPE_LOG(fname)

#endif

// EOF