#include "stdafx.h"
#include "CppUnitTest.h"
#include "agent_pasta.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

#define ARE_WFIND(exp, actual)      \
    Assert::AreNotEqual(            \
        std::wstring::npos,         \
        actual.find(exp),           \
        (std::wstring(L"expï∂éöóÒÇ™å©Ç¬Ç©ÇËÇ‹ÇπÇÒÅB\n<<exp>>\n" exp L"\n\n<<actual>>\n") + actual + L"\n----------------").c_str()  \
    )

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
            auto ghost = pasta::Agent(NULL);
            ghost.Load(loaddir.string());

            {
                auto req =
                    L"GET Version SHIORI/2.6" L"\r\n"
                    L"Charset: UTF-8" L"\r\n"
                    L"Sender: SSP" L"\r\n"
                    L"\r\n"
                    ;
                auto res = ghost.Request(req);
                ARE_WFIND(L"SHIORI/3.0 400 Bad Request", res);
            }
            {
                auto req =
                    L"GET SHIORI/3.0\r\n"
                    L"Charset: UTF-8\r\n"
                    L"ID: version\r\n"
                    L"SecurityLevel: local\r\n"
                    L"Sender: SSP\r\n"
                    L"\r\n"
                    ;
                auto res = ghost.Request(req);
                ARE_WFIND(L"SHIORI/3.0 200 OK", res);
            }
        }
    };
}