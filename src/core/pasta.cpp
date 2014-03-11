// pasta.cpp : DLL �A�v���P�[�V�����p�ɃG�N�X�|�[�g�����֐����`���܂��B
//

#include "stdafx.h"
#include "pasta.h"


pasta::App::App(const HINSTANCE hinst, const std::string& loaddir){
	this->hinst = hinst;
	// XTAL�̏�����
	using namespace xtal;
	setting.std_stream_lib = &std_stream_lib;
	setting.thread_lib = &thread_lib;
	setting.filesystem_lib = &filesystem_lib;
	setting.ch_code_lib = &ch_code_lib;
	initialize(setting);
	bind_error_message();
}

pasta::App::~App(void){
	using namespace xtal;
	uninitialize();
}


bool pasta::App::request(const std::string& request, std::string& response){

	return false;
}
