// pasta.cpp : DLL アプリケーション用にエクスポートされる関数を定義します。
//

#include "stdafx.h"
#include "pasta.h"


static xtal::Environment* XTALinitialize(const xtal::Setting& setting){
	using namespace xtal;
#ifdef XTAL_DEBUG_ALLOC
	Setting setting2 = setting;
	setting2.allocator_lib = new DebugAllocatorLib;
	Environment* env = (Environment*)setting2.allocator_lib->malloc(sizeof(Environment));
	new(env)Environment();
	env->initialize(setting2);
#else
	Environment* env = (Environment*)setting.allocator_lib->malloc(sizeof(Environment));
	new(env)Environment();
	env->initialize(setting);
#endif
	return env;
}

static void XTALuninitialize(xtal::Environment* env){
	using namespace xtal;
	if (!env){
		return;
	}

	AllocatorLib* allocacator_lib = env->setting_.allocator_lib;
	env->uninitialize();
	env->~Environment();
	allocacator_lib->free(env, sizeof(Environment));
	env = 0;

#ifdef XTAL_DEBUG_ALLOC
	((DebugAllocatorLib*)allocacator_lib)->display_debug_memory();
	delete ((DebugAllocatorLib*)allocacator_lib);
#endif

}


pasta::App::App(const HINSTANCE hinst, const std::string& loaddir){
	this->hinst = hinst;
	// XTALの初期化
	using namespace xtal;
	setting.std_stream_lib = &std_stream_lib;
	setting.thread_lib = &thread_lib;
	setting.filesystem_lib = &filesystem_lib;
	setting.ch_code_lib = &ch_code_lib;
	env = XTALinitialize(setting);
}
pasta::App::~App(void){
	XTALuninitialize(env);
}


bool pasta::App::request(const std::string& request, std::string& response){

	return false;
}
