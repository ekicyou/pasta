#include "stdafx.h"
#include "CppUnitTest.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace test
{
	using namespace biscuit;

	TEST_CLASS(PEGTests)
	{
	public:

		TEST_METHOD(HelloBiscuit)
		{
			struct c_comment : seq < str<'/', '*'>, star_until< any, str<'*', '/'> > > {};
			Assert::IsTrue((match<c_comment>(std::string("/* hello, biscuit */"))));
		}

		TEST_METHOD(WideHelloBiscuit)
		{
			struct c_comment : seq < wstr<L'/', L'*'>, star_until< any, wstr<L'*', L'/'> > > {};
			Assert::IsTrue((match<c_comment>(std::wstring(L"/* こんにちわ、ビスケット */"))));
		}

		TEST_METHOD(WideHelloBiscuit2)
		{
			struct c_comment : seq < wstr<L'開', L'始'>, star_until< any, wstr<L'終', L'了'> > > {};
			Assert::IsTrue((match<c_comment>(std::wstring(L"開始　こんにちは、ビスケット！ 終了"))));
		}
	};
}