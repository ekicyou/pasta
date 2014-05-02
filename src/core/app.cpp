// app.cpp : SHIORI API‚ÆVM‚Æ‚Ì‹´“n‚µ‚ðs‚¢‚Ü‚·B
//

#include "stdafx.h"
#include "app.h"


pasta::App::App(const HINSTANCE hinst, const std::string& loaddir){
	this->hinst = hinst;
	charMode = MODE_UTF_8;
	ctx = duk_create_heap_default();
	if (!ctx) { throw std::exception("FAIL duk_create_heap_default"); }
}

pasta::App::~App(void){
	// VM‚Ì‰ð•ú
	duk_destroy_heap(ctx);
}


bool pasta::App::request(const std::string& request, std::string& response){

	return false;
}
