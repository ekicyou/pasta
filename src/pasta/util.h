// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#pragma once

void CreateBatRequestResponse(std::string& response, const char* reason);
void CreateBatRequestResponse(std::string& response, const char* reason, const int cp);

class Pushd
{
private:
	CString mOldDir;

public:
	Pushd(LPCTSTR newdir);
	~Pushd();
};

template <class S>
S TrimRight(const S& str)
{
	int pos = int(str.size()) - 1;
	while (pos >= 0) {
		if (str[pos] <= 0x20) break;
		if (str[pos] == 0x7f) break;
		--pos;
	}
	return S(str, 0, pos + 1);
};

class CriticalSection
{
private:
	LPCRITICAL_SECTION pce;

public:
	inline CriticalSection(LPCRITICAL_SECTION pce){
		this->pce = pce;
		EnterCriticalSection(pce);
	}
	inline ~CriticalSection(void){
		LeaveCriticalSection(pce);
	}
};

// EOF