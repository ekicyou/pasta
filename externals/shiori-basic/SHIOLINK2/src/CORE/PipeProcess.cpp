// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK2.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
// パイプ通信プロセス管理
#include "StdAfx.h"
#include "PipeProcess.h"

/* ----------------------------------------------------------------------------
 * デストラクタ
 */
CPipeProcess::~CPipeProcess(void)
{
	// 最長１０秒待って、落ちなければ問答無用に叩き落す。
	if(mProcess == NULL) return;
	if(WAIT_OBJECT_0 == WaitForSingleObject(mProcess, 10000)) return;
	LOG(_T(__FUNCTION__), _T("強制終了！"));
	TerminateProcess(mProcess, -1);
}

/* ----------------------------------------------------------------------------
 * コンストラクタ
 */
CPipeProcess::CPipeProcess(
	LPCTSTR str_commandline, 
	LPCTSTR str_wrkdir, 
	bool viewConsole)
{
	SCOPE_LOG(_T(__FUNCTION__));
	LOG(_T(__FUNCTION__), _T("   cmd=[%s]"), str_commandline);
	LOG(_T(__FUNCTION__), _T("wrkdir=[%s]"), str_wrkdir);

	// コマンドライン作成
	CString cmd(str_commandline);
	cmd.Replace( _T("%PIPE%"), mPipe.GetID() );

	// StartInfo作成
	memset(&mStartInfo,0,sizeof(mStartInfo));
	mStartInfo.cb = sizeof(mStartInfo);
	mStartInfo.dwFlags =
		STARTF_USEFILLATTRIBUTE |
		STARTF_USECOUNTCHARS |
		STARTF_USESHOWWINDOW |
		0;
	// コンソールの表示・非表示制御
   if (viewConsole) mStartInfo.wShowWindow = SW_SHOWDEFAULT;
   else             mStartInfo.wShowWindow = SW_HIDE;

	// プロセス起動
	{
		Pushd dir(str_wrkdir);
		BOOL rc = CreateProcess(
			NULL, cmd.GetBuffer(), NULL, NULL, FALSE, 0, NULL, 
			str_wrkdir, &mStartInfo, &mProcessInfo);
		if(rc != TRUE){
			DWORD dwError = ::GetLastError();
			LOG( _T("ERROR CreateProcess(%x)"), dwError);
			AtlThrowLastWin32();
		}
	}
	mProcess.Attach(mProcessInfo.hProcess);
}

/* ----------------------------------------------------------------------------
 * Getter
 */
const CString CPipeProcess::GetReqName() const { return mPipe.GetReqName(); }
const CString CPipeProcess::GetResName() const { return mPipe.GetResName(); }

/* ----------------------------------------------------------------------------
 * Pipeアクセス
 */
void CPipeProcess::Write(LPCSTR buf, DWORD length) { mPipe.Write(buf, length); }
void CPipeProcess::Read (LPSTR  buf, DWORD length) { mPipe.Read (buf, length); }

void CPipeProcess::Write(const CStringA& text)          { mPipe.Write(text); }
void CPipeProcess::WriteNetString(const CharArray& buf) { mPipe.WriteNetString(buf); }
void CPipeProcess::WriteNetString(const CStringA& text) { mPipe.WriteNetString(text); }

bool CPipeProcess::ReadNetString(CharArray& buf, LPSTR& pStart, int& length) { return mPipe.ReadNetString(buf, pStart, length); }
bool CPipeProcess::ReadNetString(CharArray& buf, CStringA& text)             { return mPipe.ReadNetString(buf, text); }

// ----------------------------------------------------------------------------
// EOF