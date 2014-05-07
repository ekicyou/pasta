#include "stdafx.h"
#include "CppUnitTest.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace test
{
	using namespace std::tr2::sys;

	TEST_CLASS(LoadTest)
	{
	public:

		TEST_METHOD(AppLoad)
		{
			auto loaddir = current_path<path>();
			auto app = pasta::App(NULL, loaddir.string());
		}

		TEST_METHOD(EvalTest){
			USES_CONVERSION;

			auto loaddir = current_path<path>();
			auto app = pasta::App(NULL, loaddir.string());

			auto code =
				"(function() {\r\n"
				"var name, rc, _i, _len, _ref;\r\n"
				"print('*** All globals');\r\n"
				"rc = '';\r\n"
				"_ref = Object.getOwnPropertyNames(this);\r\n"
				"for (_i = 0, _len = _ref.length; _i < _len; _i++) {\r\n"
				"	name = _ref[_i];\r\n"
				"	rc += name + '\\r\\n';\r\n"
				"}\r\n"
				"return rc;\r\n"
				"}).call(this);\r\n"
				;
			auto rc = app.eval(code);
			OutputDebugString(L"[LoadTest::JsRunTest]globals...\n");
			OutputDebugString(A2CW_CP(rc.c_str(), CP_UTF8));
			OutputDebugString(L"[LoadTest::JsRunTest]end\n");
		}

	};

}