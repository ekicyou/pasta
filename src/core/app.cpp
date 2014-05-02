// app.cpp : SHIORI APIとVMとの橋渡しを行います。
//

#include "stdafx.h"
#include "app.h"


//ワイド文字列からマルチバイト文字列（ロケール依存）
static void narrow(const std::wstring &src, std::string &dest) {
	char *mbs = new char[src.length() * MB_CUR_MAX + 1];
	wcstombs(mbs, src.c_str(), src.length() * MB_CUR_MAX + 1);
	dest = mbs;
	delete[] mbs;
}

//マルチバイト文字列からワイド文字列（ロケール依存）
static void widen(const std::string &src, std::wstring &dest) {
	wchar_t *wcs = new wchar_t[src.length() + 1];
	mbstowcs(wcs, src.c_str(), src.length() + 1);
	dest = wcs;
	delete[] wcs;
}


pasta::App::App(const HINSTANCE hinst, const std::string& loaddir)
	:hinst(hinst)
{
	OutputDebugString(L"[pasta::App::App]開始！\n");
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
	OutputDebugString(L"[pasta::App::App]終了！\n");
}

pasta::App::~App(void){
	OutputDebugString(L"[pasta::App::~App]開始！\n");
	// VMの解放
	duk_destroy_heap(ctx);
	OutputDebugString(L"[pasta::App::~App]終了！\n");
}


bool pasta::App::request(const std::string& request, std::string& response){

	return false;
}
