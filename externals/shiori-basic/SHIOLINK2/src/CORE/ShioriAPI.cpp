// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK2.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "StdAfx.h"
#include "ShioriAPI.h"

CShioriAPI::CShioriAPI(HINSTANCE hInst, LPCTSTR loadDir)
	:mInst(hInst)
	,mLoadDir(loadDir)
{
	mLoadDir.AddBackslash();
	LOG(_T(__FUNCTION__), _T("loaddir=%s"), (LPCTSTR)mLoadDir);
}

CShioriAPI::~CShioriAPI(void)
{
}

// リクエストを処理し、応答を作成します。
bool CShioriAPI::Request(LPCSTR req, const long reqLength, CharArray &res)
{
	return false;
}
