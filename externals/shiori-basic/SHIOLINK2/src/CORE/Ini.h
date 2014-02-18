#pragma once

class CIni
{
private:
	CString mIniFilePath;

	CString mCommndLine;
	bool    mViewConsole;

public:
	CIni(const HINSTANCE hinst);
	CIni(LPCTSTR path);
	virtual ~CIni(void);

private:
	void Parse(LPCTSTR path);
};
