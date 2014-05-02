// app.cpp : SHIORI API��VM�Ƃ̋��n�����s���܂��B
//

#include "stdafx.h"
#include "app.h"


// std::string �� std::wstring�i���P�[���ˑ��j
inline std::wstring ToWideStr(const std::string &str)
{
	USES_CONVERSION;
	return A2CW(str.c_str());
}
// std::wstring �� std::string�i���P�[���ˑ��j
inline std::string ToMultStr(const std::wstring &wstr)
{
	USES_CONVERSION;
	return W2CA(wstr.c_str());
}


// std::string �� std::wstring�i�R�[�h�y�[�W�w��j
inline std::wstring ToWideStr(const std::string &str, int cp)
{
	USES_CONVERSION;
	return A2CW_CP(str.c_str(), cp);
}
// std::wstring �� std::string�i�R�[�h�y�[�W�w��j
inline std::string ToMultStr(const std::wstring &wstr, int cp)
{
	USES_CONVERSION;
	return W2CA_CP(wstr.c_str(), cp);
}


// �G���[�̃R�[���o�b�N�֐��ireturn���Ȃ�����O�ɕϊ����Ė߂��j
static void FatalFunc(duk_context *ctx, int code, const char *msg){
	USES_CONVERSION;
	std::wstring mes(L"duktape FATAL! code=(");
	mes += code;
	mes += L") ";
	mes += A2CW_CP(msg, CP_UTF8);

	OutputDebugString(L"[FatalFunc]");
	OutputDebugString(mes.c_str());
	OutputDebugString(L"\n");

	throw std::exception(W2CA(mes.c_str()));
}

#define duk_create_heap_pasta()  (duk_create_heap(NULL, NULL, NULL, NULL, FatalFunc))


// �f�X�g���N�^
pasta::App::~App(void){
	OutputDebugString(L"[pasta::App::~App]�J�n�I\n");
	// VM�̉��
	duk_destroy_heap(ctx);
	OutputDebugString(L"[pasta::App::~App]�I���I\n");
}


// �R���X�g���N�^
pasta::App::App(const HINSTANCE hinst, const std::string& loaddir)
	:hinst(hinst), loaddir(ToWideStr(loaddir)), cp(CP_UTF8)
{
	USES_CONVERSION;

#ifdef DEBUG
	{
		std::wstring mes(L"[pasta::App::App](");
		mes.append(this->loaddir);
		mes.append(L")\n");
		OutputDebugString(mes.c_str());
	}
#endif

	// VM�쐬
	ctx = duk_create_heap_pasta();
	if (!ctx) { throw std::exception("FAIL duk_create_heap_default"); }

	// ���C���X�N���v�g�̓ǂݍ���
	{
		std::tr2::sys::wpath p(this->loaddir);
		p /= L"js";
		p /= L"pasta.js";
#ifdef DEBUG
		{
			std::wstring mes(L"[pasta::App::App](");
			mes.append(p.string().c_str());
			mes.append(L")\n");
			OutputDebugString(mes.c_str());
		}
#endif
		auto utf8 = W2CA_CP(p.string().c_str(), cp);
		duk_eval_file(ctx, utf8);
		duk_pop(ctx);  /* pop eval result */
	}


	OutputDebugString(L"[pasta::App::App]�I���I\n");
}


// ���N�G�X�g����
bool pasta::App::request(const std::string& request, std::string& response){
	USES_CONVERSION;

	return false;
}
