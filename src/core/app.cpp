// app.cpp : SHIORI API��VM�Ƃ̋��n�����s���܂��B
//

#include "stdafx.h"
#include "app.h"


//���C�h�����񂩂�}���`�o�C�g������i���P�[���ˑ��j
static void narrow(const std::wstring &src, std::string &dest) {
	char *mbs = new char[src.length() * MB_CUR_MAX + 1];
	wcstombs(mbs, src.c_str(), src.length() * MB_CUR_MAX + 1);
	dest = mbs;
	delete[] mbs;
}

//�}���`�o�C�g�����񂩂烏�C�h������i���P�[���ˑ��j
static void widen(const std::string &src, std::wstring &dest) {
	wchar_t *wcs = new wchar_t[src.length() + 1];
	mbstowcs(wcs, src.c_str(), src.length() + 1);
	dest = wcs;
	delete[] wcs;
}


pasta::App::App(const HINSTANCE hinst, const std::string& loaddir)
	:hinst(hinst)
{
	OutputDebugString(L"[pasta::App::App]�J�n�I\n");
	widen(loaddir, this->loaddir);
#if DEBUG
	{
		std::wstring mes(L"[pasta::App::App]");
		mes.append(this->loaddir);
		mes.append(L"\n");
		OutputDebugString(mes.c_str());
}
#endif

	charMode = MODE_UTF_8;
	ctx = duk_create_heap_default();
	if (!ctx) { throw std::exception("FAIL duk_create_heap_default"); }
	OutputDebugString(L"[pasta::App::App]�I���I\n");
}

pasta::App::~App(void){
	OutputDebugString(L"[pasta::App::~App]�J�n�I\n");
	// VM�̉��
	duk_destroy_heap(ctx);
	OutputDebugString(L"[pasta::App::~App]�I���I\n");
}


bool pasta::App::request(const std::string& request, std::string& response){

	return false;
}
