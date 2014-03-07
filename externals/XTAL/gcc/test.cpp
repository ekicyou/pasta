#include "../src/xtal/xtal.h"
#include "../src/xtal/xtal_macro.h"

#include "../src/xtal/xtal_lib/xtal_pthread.h"
#include "../src/xtal/xtal_lib/xtal_cstdiostream.h"
#include "../src/xtal/xtal_lib/xtal_posixfilesystem.h"
#include "../src/xtal/xtal_lib/xtal_chcode.h"
#include "../src/xtal/xtal_lib/xtal_errormessage.h"

#include "time.h"

#include <iostream>

class TestGetterSetterBind{
public:
    float x, y;
    TestGetterSetterBind(): x(0), y(0) {}
};

XTAL_PREBIND(TestGetterSetterBind){
    it->def_ctor(xtal::ctor<TestGetterSetterBind>());
}

XTAL_BIND(TestGetterSetterBind){
   it->def_getter(Xid(x), &TestGetterSetterBind::x);
   it->def_setter(Xid(set_x), &TestGetterSetterBind::x);

   it->def_var(Xid(y), &TestGetterSetterBind::y);
}

struct MyData{
	int a;
};

struct MyDeleter{
	void destroy(MyData* p){
		delete p;
	}
};

XTAL_BIND(MyData){
	it->def_var(Xid(a), &MyData::a);
}

using namespace xtal;

void test(){
    lib()->def(Xid(TestGetterSetterBind), cpp_class<TestGetterSetterBind>());
	lib()->def(Xid(MyData), SmartPtr<MyData>(new MyData, MyDeleter()));
	if(CodePtr code = Xsrc((
		foo: lib::TestGetterSetterBind();
		foo.x = 0.5;
		foo.x.p;

		foo.y = 100.5;
		foo.y.p;

		mydata: lib::MyData;
		mydata.a = 10;
		mydata.a.p;
	))){
		code->call();
	}
} 

using namespace xtal;

int main2(int argc, char** argv){
	
	debug::enable_debug_compile();
	
	CodePtr code = compile_file("../test/exec.xtal");
	XTAL_CATCH_EXCEPT(e){
		stderr_stream()->println(e);
		return 1;
	}

	code->call();

	XTAL_CATCH_EXCEPT(e){
		stderr_stream()->println(e);
		return 1;
	}

	lib()->member("test")->send("run_dir", "../test");

	XTAL_CATCH_EXCEPT(e){
		stderr_stream()->println(e);
		return 1;
	}

	return 0;
}

int main(int argc, char** argv){
	CStdioStdStreamLib stream_lib;
	PThreadLib thread_lib;
	PosixFilesystemLib filesystem_lib;
	SJISChCodeLib ch_code_lib;

	Setting setting;
	setting.thread_lib = &thread_lib;
	setting.std_stream_lib = &stream_lib;
	setting.filesystem_lib = &filesystem_lib;
	setting.ch_code_lib = &ch_code_lib;

	initialize(setting);
	//bind_error_message();

	int ret = main2(argc, argv);

	vmachine()->print_info();
	uninitialize();

	return ret;
}

