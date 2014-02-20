// pasta.cpp : DLL アプリケーション用にエクスポートされる関数を定義します。
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
