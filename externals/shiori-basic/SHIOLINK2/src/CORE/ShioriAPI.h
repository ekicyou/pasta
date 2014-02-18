// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK2.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#pragma once
#include "util.h"

class CShioriAPI
{
private:
	HINSTANCE mInst;
	CPath     mLoadDir;

public:
	CShioriAPI(HINSTANCE hInst, LPCTSTR loadDir);
	virtual ~CShioriAPI(void);
public:
	// リクエストを処理し、応答を作成します。
	bool Request(LPCSTR req, const long reqLength, CharArray &res);
};
