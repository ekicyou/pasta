// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK2.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
// パイプ通信プロセス管理
#pragma once
#include "util.h"
#include "pipe.h"

class CPipeProcess
{
private:
	CServerPipe mPipe;
	STARTUPINFO mStartInfo;
	PROCESS_INFORMATION mProcessInfo;
	CHandle mProcess;

public:
	CPipeProcess(LPCTSTR str_commandline, LPCTSTR str_wrkdir, bool viewConsole = false);
	virtual ~CPipeProcess(void);
	const CString GetReqName(void) const;
	const CString GetResName(void) const;

	void Write(LPCSTR buf, DWORD length);
	void Read (LPSTR  buf, DWORD length);

	void Write(const CStringA& text);
	void WriteNetString(const CharArray& buf);
	void WriteNetString(const CStringA& text);

	bool ReadNetString(CharArray& buf, LPSTR& pStart, int& length);
	bool ReadNetString(CharArray& buf, CStringA& text);
};
