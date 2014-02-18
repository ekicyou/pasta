#include "StdAfx.h"
#include "IniFile.h"
#include "..\\ini\\ParseRequest.h"
#include "Util.h"

// コンストラクタ
CIniFile::CIniFile(void)
{
}

CIniFile::~CIniFile(void)
{
}

void CIniFile::Load(const CString& path, const UINT codePage)
{
	SECTION sec(SECTION_NONE);
	DESC_KEY key(DESC_KEY_NONE);
	CString text = Util::File::ReadAllText(path, codePage);
	RequestScanner sc(text);
	while(true){
		RequestScanner::Token t=sc.Scan();
		switch(t.GetToken()){
			case RequestScanner::TOKEN_EOT:
				goto FIN;

			case RequestScanner::TOKEN_SECTION_NAME:
				sec = t.GetText() == _T("SHIOLINK2")
					? SECTION_SHIOLINK2 : SECTION_NONE;
				break;

			case RequestScanner::TOKEN_ITEM_KEYWORD:
				if(sec != SECTION_SHIOLINK2) break;
				if     (t.GetText() == _T("Charset"))			key = DESC_KEY_SHIOLINK2_CHARSET;
				else if(t.GetText() == _T("LinkType"))			key = DESC_KEY_SHIOLINK2_LINK_TYPE;
				else if(t.GetText() == _T("ActiveCodeGUID"))	key = DESC_KEY_SHIOLINK2_ACTIVE_CODE_GUID;
				else											key = DESC_KEY_NONE;
				break;

			case RequestScanner::TOKEN_ITEM_SPLIT:
				break;

			case RequestScanner::TOKEN_ITEM_VALUE:
				SetValue(key, t.GetText(), false);
				break;

			case RequestScanner::TOKEN_ITEM_VALUE_QUOT:
				SetValue(key, t.GetText(), true);
				break;

			default:
				sec = SECTION_NONE;
				key = DESC_KEY_NONE;
		}
	}
FIN:
	return;
}


void CIniFile::SetValue(const DESC_KEY key, const CString& value, const bool isQuot)
{
	CString v;
	if(isQuot)	v = Util::Text::UnQuot(value);
	else		v = value;
	switch(key){
		case DESC_KEY_SHIOLINK2_CHARSET:			SetCharSet(v); return;
		case DESC_KEY_SHIOLINK2_LINK_TYPE:			SetLinkType(v); return;
		case DESC_KEY_SHIOLINK2_ACTIVE_CODE_GUID:	SetActiveCodeGUID(v); return;
	}
}

void CIniFile::SetCharSet(const CString& value)
{
	mCodePage = Util::Text::GetCP(value);
}

void CIniFile::SetLinkType(const CString& value)
{
	CString lo(value);
	lo.MakeLower();
	if		(lo == _T("process"))		mLinkType = LINKTYPE_PROCESS;
	else if	(lo == _T("activecode"))	mLinkType = LINKTYPE_ACTIVE_CODE;
}

void CIniFile::SetActiveCodeGUID(const CString& value)
{
	mActiveCodeGUID = value;
}

//EOF