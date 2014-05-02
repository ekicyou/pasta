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
			OutputDebugString(L"[AppLoad]�J�n�I\n");
			auto loaddir = current_path<path>();
			auto app = pasta::App(NULL, loaddir.string());
			OutputDebugString(L"[AppLoad]�I���I\n");
		}
	};
}