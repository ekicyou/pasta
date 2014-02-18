#include "stdafx.h"
#include "Util.h"

	// 指定したcodepageのテキストとしてファイルを読み込みます
CString Util::File::ReadAllText(const CString& path, const UINT codePage)
{
	CString rc;
	CAtlFile file;
	if(file.Create(path, GENERIC_READ, FILE_SHARE_READ, OPEN_EXISTING) != S_OK){
		rc.Format(_T("ERROR: File Not Found[path=%s]\n"), (LPCTSTR)path);
		return rc;
	}
	ULONGLONG size;
	if(file.GetSize(size) != S_OK){
		rc.Format(_T("ERROR: File GetSize[%s]\n"), (LPCTSTR)path);
		return rc;
	}
	CAtlArray<char> buf;
	buf.SetCount((size_t)size+1);
	if(file.Read(buf.GetData(), (DWORD)size) != S_OK){
		rc.Format(_T("ERROR: File Read[%s]\n"), (LPCTSTR)path);
		return rc;
	}
	buf[(size_t)size] = 0;
	rc = CA2CT(buf.GetData(), codePage);
	return rc;
}

// 文字列から[""] を除去します。
CString Util::Text::UnQuot(const CString& text)
{
	CString rc;
	int index = 0;
	int st = 0;
	int len = 0;
	int quotMark = 0;
	for(;text[index] != 0; index++){
		len++;
		if( text[index] != _T('\"')){
			quotMark = 0;
			continue;
		}
		quotMark++;
		if(quotMark != 2) continue;
		rc += text.Mid(st, len-2);
		quotMark = 0;
		st = index+1;
		len = 0;
	}
	if(len > 0) rc += text.Mid(st, len);
	return rc;
}

// コードページを取得します。
UINT Util::Text::GetCP(const CString& text)
{
	CString lo(text);
	lo.MakeLower();
	if		(lo == _T("utf-8"))		return CP_UTF8;
	else if	(lo == _T("shift_jis"))	return 932;
	else if	(lo == _T("ansi"))		return CP_ACP;
	else if	(lo == _T("utf8"))		return CP_UTF8;
	else if	(lo == _T("utf_8"))		return CP_UTF8;
	else if	(lo == _T("shiftjis"))	return 932;
	else if	(lo == _T("shift-jis"))	return 932;
	else if	(lo == _T("sjis"))		return 932;

	return CP_ACP;
}

//EOF