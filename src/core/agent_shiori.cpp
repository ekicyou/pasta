// req_async.cpp : 非同期エージェントとしてSHIORI APIと通信します。
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "shiori_parse.h"
#include "shiori_response.h"
#include "util.h"
#include <regex>

//============================================================
// 初期化
//============================================================
shiori::Agent::Agent()
    :agent(), hasUnload(false)
{
}

shiori::Agent::Agent(concurrency::Scheduler& scheduler)
    : agent(scheduler), hasUnload(false)
{
}

shiori::Agent::Agent(concurrency::ScheduleGroup& group)
    : agent(group), hasUnload(false)
{
}

//============================================================
// 解放
//============================================================
shiori::Agent::~Agent()
{
    UnLoad();
}

//============================================================
// SHIORI API処理
//============================================================
void shiori::Agent::Load(const HINSTANCE hinst, const int cp, const std::wstring& dir)
{
    FUNC_START;

    this->hinst = hinst;
    this->cp = cp;
    this->loaddir = dir;
    this->hasUnload = true;
    start();
    asend(reqBuf, RequestItem(REQUEST_LOAD));
}

void shiori::Agent::UnLoad()
{
    if (!hasUnload)return;

    FUNC_START;

    asend(reqBuf, RequestItem(REQUEST_UNLOAD));
    wait(this);
    hasUnload = false;
}

const std::wstring shiori::Agent::Notify(const std::wstring& req)
{
    FUNC_START;

    asend(reqBuf, RequestItem(REQUEST_NOTIFY, req));
    return WSTR_RES_NO_CONTENT;
}

const std::wstring shiori::Agent::Get(const std::wstring& req)
{
    FUNC_START;

    asend(reqBuf, RequestItem(REQUEST_GET, req));
    auto res = receive(resBuf);
    if (res.resType == RESPONSE_ERROR){
        USES_CONVERSION;
        throw std::exception(W2CA_CP(res.value.c_str(), CP_UTF8));
    }
    return res.value;
}

void shiori::Agent::Response(const std::wstring& res)
{
    FUNC_START;

    asend(resBuf, ResponseItem(res, RESPONSE_NORMAL));
    hasResponse = false;
}

const std::wstring shiori::Agent::Request(const std::wstring& req)
{
    FUNC_START;
    try{
        // SHIORI REQUESTを解析
        auto text = req.c_str();
        auto match = matchShioriRequest(text);

        // 解析に失敗
        if (match.empty())      return WSTR_RES_BAT_REQUEST;
        if (match.size() < 2)   THROW_EX("matchShioriRequest INTERNAL ERROR");

        // GET
        std::wstring reqType(match[1].first, match[1].second);
        if (reqType == L"GET")      return Get(req);
        if (reqType == L"NOTIFY")   return Notify(req);

        THROW_EX("unmatch request type");
    }
    catch (const std::exception& ex){
        std::string mes(STR_RES_SERVER_ERROR);
        mes += "X-PASTA-Resion: ";
        mes += ex.what();
        mes += "\r\n\r\n";
        return ToWideStr(mes, CP_UTF8);
    }
    catch (...){
        std::string mes(STR_RES_SERVER_ERROR);
        mes += "X-PASTA-Resion: ";
        mes += "NOT std::exception fail";
        mes += "\r\n\r\n";
        return ToWideStr(mes, CP_UTF8);
    }
}

//============================================================
// SHIORI本体側の非同期メインループ
//============================================================
void shiori::Agent::run(){
    FUNC_START;

    try{
        // load処理
        try{
            DEBUG_MESSAGE(L"WAIT load");
            auto req = receive(reqBuf);
            DEBUG_MESSAGE(L"CALL LoadAction()");
            LoadAction();
            DEBUG_MESSAGE(L"END  LoadAction()");
        }
        catch (const std::exception& ex){ SetException(ex); }
        catch (...)                     { SetException(); }

        // メインループ
        while (true){
            DEBUG_MESSAGE("WAIT request");
            auto req = receive(reqBuf);
            switch (req.reqType)
            {
            case shiori::REQUEST_NOTIFY:
                try{
                    DEBUG_MESSAGE(L"CALL NotifyAction()");
                    NotifyAction(req.value);
                    DEBUG_MESSAGE(L"END  NotifyAction()");
                }
                catch (const std::exception& ex){ SetException(ex); }
                catch (...)                     { SetException(); }
                continue;

            case shiori::REQUEST_GET:
                try{
                    // GetAction内でSHIORIレスポンスを返すこと
                    hasResponse = true;
                    DEBUG_MESSAGE(L"CALL GetAction()");
                    GetAction(req.value);
                    DEBUG_MESSAGE(L"END  GetAction()");

                    // GetAction内でResponseが呼び出されていない場合は例外とする。
                    if (!hasResponse){
                        THROW_EX("script not response [GET]");
                    }
                }
                catch (const std::exception& ex){ SetException(ex); SendException(); }
                catch (...)                     { SetException();   SendException(); }
                continue;
            }
            break;
        }

        // unload処理
        try{
            DEBUG_MESSAGE(L"CALL UnLoadAction()");
            UnLoadAction();
            DEBUG_MESSAGE(L"END  UnLoadAction()");
        }
        catch (const std::exception& ex){ SetException(ex); }
        catch (...)                     { SetException(); }
    }
    catch (const std::exception& ex){ SetException(ex); }
    catch (...)                     { SetException(); }
    done();
}

//============================================================
// 例外処理
//============================================================

void shiori::Agent::SetException(const std::exception& ex){
    FUNC_START;

    DEBUG_MESSAGE(ex.what());
    last_error_what = ToWideStr(ex.what(), CP_UTF8);
}

void shiori::Agent::SetException(){
    FUNC_START;

    SetException(std::exception("(none)"));
}

void shiori::Agent::SendException(){
    FUNC_START;

    auto res = ResponseItem(last_error_what, RESPONSE_ERROR);
    asend(resBuf, res);
}

// EOF