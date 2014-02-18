#include "StdAfx.h"
#include "Pipe.h"

/* ----------------------------------------------------------------------------
 * [CPipe]初期化・開放
 */
CPipe::CPipe(void)
{
	SCOPE_LOG(_T(__FUNCTION__));
}

CPipe::~CPipe(void)
{
	SCOPE_LOG(_T(__FUNCTION__));
}

void CPipe::Close(void)
{
	if(mRead !=NULL) mRead .Close();
	if(mWrite!=NULL) mWrite.Close();
}

/* ----------------------------------------------------------------------------
 * [CPipe]Getter
 */
LPCTSTR CPipe::GetID()             const { return mID; }
LPCTSTR CPipe::GetBaseName()       const { return mBaseName; }
const CString CPipe::GetReqName()  const { return mBaseName + _T("req"); }
const CString CPipe::GetResName()  const { return mBaseName + _T("res"); }

/* ----------------------------------------------------------------------------
 * [CPipe]Setter
 */
void CPipe::SetID(LPCTSTR id)
{
	mID = id;
	CString buf;
	buf.Format(_T("\\\\.\\pipe\\sl2scgi%s"), mID);
	mBaseName = CPath(buf);
}

/* ----------------------------------------------------------------------------
 * [CPipe]書き込み
 */
void CPipe::Write(LPCSTR buf, DWORD length)
{
	SCOPE_LOG(_T(__FUNCTION__));
	LPBYTE pos = (LPBYTE)buf;
	while(length > 0){
		DWORD resultLen;
		if(!::WriteFile(mWrite, pos, length, &resultLen, NULL)) ::AtlThrowLastWin32();
		pos += resultLen;
		length -= resultLen;
	}
}
void CPipe::Write(const CStringA& text)
{
	SCOPE_LOG(_T(__FUNCTION__));
	Write((LPCSTR)text, text.GetLength());
}

/* ----------------------------------------------------------------------------
 * [CPipe]読み込み
 */
void CPipe::Read(LPSTR buf, DWORD length)
{
	SCOPE_LOG(_T(__FUNCTION__));
	LPSTR pos = buf;
	while(length > 0){
		DWORD resultLen;
		if(!::ReadFile(mRead, pos, length, &resultLen, NULL)) ::AtlThrowLastWin32();
		pos += resultLen;
		length -= resultLen;
	}
}


/* ----------------------------------------------------------------------------
 * [CPipe]netStringの書き込み
 */
void CPipe::WriteNetString(const CharArray& buf)
{
	SCOPE_LOG(_T(__FUNCTION__));
	// フォーマットの作成
	CStringA data;
	data.Format("%d:", buf.GetCount());
	data += CStringA((LPCSTR)buf.GetData(), buf.GetCount());
	data += ',';

	// 送信
	Write(data);
}
void CPipe::WriteNetString(const CStringA& text)
{
	SCOPE_LOG(_T(__FUNCTION__));
	CharArray buf;
	buf.SetCount(text.GetLength());
	memcpy(buf.GetData(), (LPCSTR)text, buf.GetCount());
	WriteNetString(buf);
}

/* ----------------------------------------------------------------------------
 * [CPipe]netStringの書き込み
 */
enum CharChass{
	NETSTRING_SPLIT = -1,
	NETSTRING_OTHER = -2,
};
const int CHECK_NETSTRING_SPLIT = (':' - '0');

static inline int CharCheck(CHAR a){
	if(a >= '0' && a <= '9') return a - '0';
	if(a == ':')             return NETSTRING_SPLIT;
	return NETSTRING_OTHER;
}

#define READONE(_pos_)                          \
{                                               \
	int a = CharCheck(buf[_pos_]);              \
	if(a==NETSTRING_OTHER) return false;        \
	if(a==NETSTRING_SPLIT) goto FIND_SPLIT;     \
	len = len*10 + a; numLength++;              \
}


bool CPipe::ReadNetString(CharArray& buf, LPSTR& pStart, int& length)
{
	SCOPE_LOG(_T(__FUNCTION__));
	size_t len;
	size_t numLength = 0;
	size_t bufSize = 3;
	size_t readSize = 3;

	// 最初の３文字を読む
	if(buf.GetCount()<3) buf.SetCount(3);
	Read(buf.GetData(), 3);

	// １文字目
	{
		int a = CharCheck(buf[0]);
		if(a<0 || a>9) return false;
		len = a;
		numLength++;
		if(len==0){
			if(buf[1] != ':') return false;
			goto CHECK_LAST_CHAR;
		}
	}

	// ２・３文字目
	READONE(1);
	READONE(2);

	// 次の４文字を読む（最大で999999まで）
	if(buf.GetCount()<7) buf.SetCount(7);
	Read(buf.GetData()+3, 4);
	readSize += 4;

	READONE(3);
	READONE(4);
	READONE(5);
	READONE(6);
	return false;

FIND_SPLIT:
	// 残りを全て読み込む
	bufSize = numLength + 1 + len + 1;
	int remainSize = bufSize - readSize;
	if(buf.GetCount()<bufSize) buf.SetCount(bufSize);
	Read(buf.GetData()+readSize, remainSize);

CHECK_LAST_CHAR:
	// 最後の文字列が正しいか？
	if(buf[bufSize-1] != ',') return false;

	// 正しい
	pStart = &buf[numLength + 1];
	length =(int) len;
	return true;
}
bool CPipe::ReadNetString(CharArray& buf, CStringA& text)
{
	SCOPE_LOG(_T(__FUNCTION__));
	LPSTR pStart;
	int length;
	if(! ReadNetString(buf, pStart, length)) return false;
	text = CStringA(pStart, length);
	return true;
}

/* ----------------------------------------------------------------------------
 * [CServerPipe]初期化・開放
 */
CServerPipe::CServerPipe(void)
	:CPipe()
{
	SCOPE_LOG(_T(__FUNCTION__));
}

CServerPipe::~CServerPipe(void)
{
	SCOPE_LOG(_T(__FUNCTION__));
}

/* ----------------------------------------------------------------------------
 * [CServerPipe]パイプの作成
 */
void CServerPipe::Create(void)
{
	SCOPE_LOG(_T(__FUNCTION__));
	for(int i=0; i<10; i++){
		if(TryCreate()){
			return;
		}
	}
	AtlThrow(ERROR_CANNOT_MAKE);
}

static bool firstCreate = true;
bool CServerPipe::TryCreate(void)
{
	SCOPE_LOG(_T(__FUNCTION__));
	Close();

	// 初回ならリセット
	if(firstCreate){
		firstCreate = false;
		srand(GetTickCount());
	}

	// 名前の作成
	for(UINT uUnique=rand() ;;uUnique++){
		if(uUnique==0) continue;
		CString id;
		id.Format(_T("%x"), uUnique);
		SetID(id);
		break;
	}
	LOG(_T(__FUNCTION__), _T("baseName = [%s]"), GetBaseName());
	LOG(_T(__FUNCTION__), _T(" reqName = [%s]"), GetReqName());
	LOG(_T(__FUNCTION__), _T(" resName = [%s]"), GetResName());

	// SECURITY_ATTRIBUTES の設定(パイプを作るのに必要)
	SECURITY_ATTRIBUTES secAtt;
	memset(&secAtt,0,sizeof(secAtt));
	secAtt.nLength =sizeof(secAtt);
	secAtt.lpSecurityDescriptor = NULL;
	secAtt.bInheritHandle       = FALSE;
	const DWORD PIPE_MODE = PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT;

	// req用(Write)
	HANDLE req = ::CreateNamedPipe(
		GetReqName(),
		PIPE_ACCESS_OUTBOUND,
		PIPE_MODE,
		1, 1024, 1024, 0, &secAtt);
	if (req == INVALID_HANDLE_VALUE) return false;
	mWrite.Attach(req);

	// res用(Read)
	HANDLE res = ::CreateNamedPipe(
		GetResName(),
		PIPE_ACCESS_INBOUND,
		PIPE_MODE,
		1, 1024, 1024, 0, &secAtt);
	if (res == INVALID_HANDLE_VALUE) return false;
	mRead.Attach(res);

	//
	return true;
}

/* ----------------------------------------------------------------------------
 * [CServerPipe]書き込み可能になるまで待機します。
 */
bool CServerPipe::WaitForConnection(void)
{
	SCOPE_LOG(_T(__FUNCTION__));
	BOOL rc = ::ConnectNamedPipe(mWrite, NULL);
	return rc == TRUE;
}

/* ----------------------------------------------------------------------------
 * [CClientPipe]初期化・開放
 */
CClientPipe::CClientPipe(LPCTSTR id)
	:CPipe()
{
	SCOPE_LOG(_T(__FUNCTION__));
	Create(id);
}

CClientPipe::~CClientPipe(void)
{
	SCOPE_LOG(_T(__FUNCTION__));
}

/* ----------------------------------------------------------------------------
 * [CClientPipe]パイプの作成
 */
void CClientPipe::Create(LPCTSTR id)
{
	SCOPE_LOG(_T(__FUNCTION__));
	Close();

	// 名前の作成
	SetID(id);

	// req用(Read)
	{
		CString path(GetReqName());
		HANDLE handle = ::CreateFile( path, GENERIC_READ, 0,
			NULL, OPEN_EXISTING, 0, NULL);
		if (handle == INVALID_HANDLE_VALUE) AtlThrowLastWin32();
		mRead.Attach(handle);
	}

	// res用(Write)
	{
		CString path(GetResName());
		HANDLE handle = ::CreateFile( path, GENERIC_WRITE , 0,
			NULL, OPEN_EXISTING, 0, NULL);
		if (handle == INVALID_HANDLE_VALUE) AtlThrowLastWin32();
		mWrite.Attach(handle);
	}

}

// EOF