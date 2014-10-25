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
            OutputDebugString(L"## 1\n");
            auto ghost = pasta::Agent();
            OutputDebugString(L"## 2\n");
            ghost.Load(NULL, CP_UTF8, loaddir.string());
            OutputDebugString(L"## 3\n");
        }

	};
}