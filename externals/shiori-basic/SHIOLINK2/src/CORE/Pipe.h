#pragma once
#include "Util.h"

class CPipe
{
protected:
	CString mID;
	CPath   mBaseName;
	CHandle mWrite;
	CHandle mRead;

public:
	CPipe(void);
	virtual ~CPipe(void);
	void Close(void);

public:
	LPCTSTR GetID(void) const;
	LPCTSTR GetBaseName(void) const;
	const CString GetReqName(void) const;
	const CString GetResName(void) const;

	void Write(LPCSTR buf, DWORD length);
	void Read (LPSTR  buf, DWORD length);

	void Write(const CStringA& text);
	void WriteNetString(const CharArray& buf);
	void WriteNetString(const CStringA& text);

	bool ReadNetString(CharArray& buf, LPSTR& pStart, int& length);
	bool ReadNetString(CharArray& buf, CStringA& text);

protected:
	void SetID(LPCTSTR id);
};

class CServerPipe : public CPipe
{
public:
	CServerPipe(void);
	virtual ~CServerPipe(void);

public:
	void Create(void);
	bool WaitForConnection(void);
private:
	bool TryCreate(void);
};

class CClientPipe : public CPipe
{
public:
	CClientPipe(LPCTSTR id);
	virtual ~CClientPipe(void);

public:
	void Create(LPCTSTR id);
};
