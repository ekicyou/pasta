// pasta.cpp : DLL �A�v���P�[�V�����p�ɃG�N�X�|�[�g�����֐����`���܂��B
//

#include "stdafx.h"
#include "pasta.h"

pasta::App::App(const HINSTANCE hinst, const std::string& loaddir){
	this->hinst = hinst;

}
pasta::App::~App(void){

}
bool pasta::App::request(const std::string& request, std::string& response){

	return false;
}
