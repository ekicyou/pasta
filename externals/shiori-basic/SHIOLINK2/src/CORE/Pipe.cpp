#include "StdAfx.h"
#include "Pipe.h"

/* ----------------------------------------------------------------------------
 * [CPipe]�������E�J��
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
 * [CPipe]��������
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
 * [CPipe]�ǂݍ���
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
 * [CPipe]netString�̏�������
 */
void CPipe::WriteNetString(const CharArray& buf)
{
	SCOPE_LOG(_T(__FUNCTION__));
	// �t�H�[�}�b�g�̍쐬
	CStringA data;
	data.Format("%d:", buf.GetCount());
	data += CStringA((LPCSTR)buf.GetData(), buf.GetCount());
	data += ',';

	// ���M
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
 * [CPipe]netString�̏�������
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

	// �ŏ��̂R������ǂ�
	if(buf.GetCount()<3) buf.SetCount(3);
	Read(buf.GetData(), 3);

	// �P������
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

	// �Q�E�R������
	READONE(1);
	READONE(2);

	// ���̂S������ǂށi�ő��999999�܂Łj
	if(buf.GetCount()<7) buf.SetCount(7);
	Read(buf.GetData()+3, 4);
	readSize += 4;

	READONE(3);
	READONE(4);
	READONE(5);
	READONE(6);
	return false;

FIND_SPLIT:
	// �c���S�ēǂݍ���
	bufSize = numLength + 1 + len + 1;
	int remainSize = bufSize - readSize;
	if(buf.GetCount()<bufSize) buf.SetCount(bufSize);
	Read(buf.GetData()+readSize, remainSize);

CHECK_LAST_CHAR:
	// �Ō�̕����񂪐��������H
	if(buf[bufSize-1] != ',') return false;

	// ������
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
 * [CServerPipe]�������E�J��
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
 * [CServerPipe]�p�C�v�̍쐬
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

	// ����Ȃ烊�Z�b�g
	if(firstCreate){
		firstCreate = false;
		srand(GetTickCount());
	}

	// ���O�̍쐬
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

	// SECURITY_ATTRIBUTES �̐ݒ�(�p�C�v�����̂ɕK�v)
	SECURITY_ATTRIBUTES secAtt;
	memset(&secAtt,0,sizeof(secAtt));
	secAtt.nLength =sizeof(secAtt);
	secAtt.lpSecurityDescriptor = NULL;
	secAtt.bInheritHandle       = FALSE;
	const DWORD PIPE_MODE = PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT;

	// req�p(Write)
	HANDLE req = ::CreateNamedPipe(
		GetReqName(),
		PIPE_ACCESS_OUTBOUND,
		PIPE_MODE,
		1, 1024, 1024, 0, &secAtt);
	if (req == INVALID_HANDLE_VALUE) return false;
	mWrite.Attach(req);

	// res�p(Read)
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
 * [CServerPipe]�������݉\�ɂȂ�܂őҋ@���܂��B
 */
bool CServerPipe::WaitForConnection(void)
{
	SCOPE_LOG(_T(__FUNCTION__));
	BOOL rc = ::ConnectNamedPipe(mWrite, NULL);
	return rc == TRUE;
}

/* ----------------------------------------------------------------------------
 * [CClientPipe]�������E�J��
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
 * [CClientPipe]�p�C�v�̍쐬
 */
void CClientPipe::Create(LPCTSTR id)
{
	SCOPE_LOG(_T(__FUNCTION__));
	Close();

	// ���O�̍쐬
	SetID(id);

	// req�p(Read)
	{
		CString path(GetReqName());
		HANDLE handle = ::CreateFile( path, GENERIC_READ, 0,
			NULL, OPEN_EXISTING, 0, NULL);
		if (handle == INVALID_HANDLE_VALUE) AtlThrowLastWin32();
		mRead.Attach(handle);
	}

	// res�p(Write)
	{
		CString path(GetResName());
		HANDLE handle = ::CreateFile( path, GENERIC_WRITE , 0,
			NULL, OPEN_EXISTING, 0, NULL);
		if (handle == INVALID_HANDLE_VALUE) AtlThrowLastWin32();
		mWrite.Attach(handle);
	}

}

// EOF