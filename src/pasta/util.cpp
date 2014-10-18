// ----------------------------------------------------------------------------
// �ʃv���Z�X�ʐMSHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"
#include "util.h"

/**----------------------------------------------------------------------------
 * SHIORI 3.0 RESPONSE
 */
// RESPOSE 400: Bad Request
void CreateBatRequestResponse(std::string& response, const char* reason)
{
	response =
		"SHIORI/3.0 400 Bad Request\r\n"
		"Charset: UTF-8\r\n"
		"Sender: PASTA\r\n"
		"X-PASTA-Reason: ";
	response += reason;
	response += "\r\n\r\n";
}

void CreateBatRequestResponse(std::string& response, const char* reason, const int cp){
	USES_CONVERSION;
	auto message = W2A(A2W_CP(reason, cp));
	CreateBatRequestResponse(response, reason);
}

/**----------------------------------------------------------------------------
 * �J�����g�f�B���N�g���ړ������A
 */

Pushd::Pushd(LPCTSTR newdir)
	:mOldDir()
{
	TCHAR buf[_MAX_PATH + 1];
	GetCurrentDirectory(sizeof(buf), buf);
	mOldDir = buf;
	BOOL rc = SetCurrentDirectory(newdir);
	if (!rc) AtlThrow(FAILED(ERROR_CURRENT_DIRECTORY));
}

Pushd::~Pushd()
{
	if (mOldDir.IsEmpty()) return;
	SetCurrentDirectory(mOldDir);
}

// EOF