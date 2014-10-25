#include "stdafx.h"
#include "CppUnitTest.h"
#include "agent_pasta.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace test
{
    using namespace std::tr2::sys;
   
    TEST_CLASS(PastaAgentTest)
	{
	public:
		
		TEST_METHOD(BootTest)
		{
            USES_CONVERSION;
            auto loaddir = current_path<wpath>();
            auto ghost = pasta::Agent();
            ghost.Load(NULL, CP_UTF8, loaddir.string());

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
            auto rc = ghost.eval(code);
            OutputDebugString(L"[LoadTest::JsRunTest]globals...\n");
            OutputDebugString(A2CW_CP(rc.c_str(), CP_UTF8));
            OutputDebugString(L"[LoadTest::JsRunTest]end\n");

        }

	};
}