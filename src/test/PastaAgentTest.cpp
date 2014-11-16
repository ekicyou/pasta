#include "stdafx.h"
#include "CppUnitTest.h"
#include "agent_pasta.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

inline void AreFind(LPCSTR exp, std::string& actual){
    if (actual.find(exp) != std::string::npos)return;

    USES_CONVERSION;
    std::wstring mes;
    mes += L"expï∂éöóÒÇ™å©Ç¬Ç©ÇËÇ‹ÇπÇÒÅB\n<<exp>>\n";
    mes += A2CW_CP(exp, CP_UTF8);
    mes += L"\n\n<<actual>>\n";
    mes += A2CW_CP(actual.c_str(), CP_UTF8);
    mes += L"\n----------------";
    Assert::Fail(mes.c_str());
}

namespace test
{
    using namespace std::tr2::sys;

    TEST_CLASS(PastaAgentTest)
    {
    public:

        TEST_METHOD(BootTest)
        {
            auto loaddir = current_path<wpath>();
            auto ghost = pasta::Agent(NULL);
            ghost.load(loaddir.string());

            {
                auto req =
                    "GET Version SHIORI/2.6"    "\r\n"
                    "Charset: UTF-8"            "\r\n"
                    "Sender: SSP"               "\r\n"
                    "\r\n"
                    ;
                auto res = ghost.Request(req);
                AreFind("SHIORI/3.0 400 Bad Request", res);
            }
            {
                auto req =
                    "GET SHIORI/3.0\r\n"
                    "Charset: UTF-8\r\n"
                    "ID: version\r\n"
                    "SecurityLevel: local\r\n"
                    "Sender: SSP\r\n"
                    "\r\n"
                    ;
                auto res = ghost.Request(req);
                AreFind("SHIORI/3.0 200 OK", res);
            }
        }
    };
}