#include "StdAfx.h"
#include "Ini.h"

/* ----------------------------------------------------------------------------
 * デストラクタ
 */
CIni::~CIni(void)
{
}

/* ----------------------------------------------------------------------------
 * コンストラクタ
 */
CIni::CIni(const HINSTANCE hinst)
{
	TCHAR buf[ _MAX_PATH + 1 ];
	::GetModuleFileName( hinst, buf, _MAX_PATH );
	Parse(buf);
}

CIni::CIni(LPCTSTR path)
{
	Parse(path);
}

/* ----------------------------------------------------------------------------
 * Iniファイルの解析
 */
static inline CString PPSTR(LPCTSTR path, LPCTSTR section, LPCTSTR field, LPCTSTR defvalue)
{
	TCHAR buf[ _MAX_PATH + 1 ];
	GetPrivateProfileString(section, field, defvalue, buf, sizeof(buf), path);
	return buf;
}
static inline bool PPBOOL(LPCTSTR path, LPCTSTR section, LPCTSTR field, const bool defvalue)
{
	LPTSTR def = _T("T");
	if(! defvalue) def = _T("F");
	CString value = PPSTR(path, section, field, def);
	value.Trim();
	if(value.IsEmpty()) return false;
	CString lower = value.MakeLower();
	switch(lower[0]){
		case _T('1'):
		case _T('t'):
			return true;
		case _T('o'):
			return lower == _T("on");
	}
	return false;
}


void CIni::Parse(LPCTSTR path)
{
	mIniFilePath = path;
	LPCTSTR sec = _T("SHIOLINK2");
	mCommndLine  = PPSTR (path, sec, _T("COMMANDLINE"), _T("foo.exe -pipe:%PIPE%"));
	mViewConsole = PPBOOL(path, sec, _T("VIEW_CONSOLE"), false);
}


