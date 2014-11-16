// req_async.cpp : �񓯊��G�[�W�F���g�Ƃ���SHIORI API�ƒʐM���܂��B
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "shiori_parse.h"
#include "shiori_response.h"
#include "util.h"
#include <regex>

//============================================================
// ������
//============================================================
shiori::Agent::Agent(const int cp, const HINSTANCE hinst)
    :agent(), hinst(hinst), cp(cp), hasUnload(false)
{
}

shiori::Agent::Agent(const int cp, const HINSTANCE hinst, concurrency::Scheduler& scheduler)
    : agent(scheduler), hinst(hinst), cp(cp), hasUnload(false)
{
}

shiori::Agent::Agent(const int cp, const HINSTANCE hinst, concurrency::ScheduleGroup& group)
    : agent(group), hinst(hinst), cp(cp), hasUnload(false)
{
}

//============================================================
// ���
//============================================================
shiori::Agent::~Agent()
{
    unload();
}

//============================================================
// SHIORI API����
//============================================================
void shiori::Agent::load(const std::wstring& dir)
{
    FUNC_START(cp);

    this->loaddir = dir;
    this->hasUnload = true;
    start();
    asend(reqBuf, RequestItem(REQUEST_LOAD));
}

void shiori::Agent::unload()
{
    if (!hasUnload)return;

    FUNC_START(cp);
    if (IsRunning()){
        asend(reqBuf, RequestItem(REQUEST_UNLOAD));
    }
    wait(this);
    hasUnload = false;
}

const std::string shiori::Agent::Notify(const std::string& req)
{
    FUNC_START(cp);

    asend(reqBuf, RequestItem(REQUEST_NOTIFY, req));
    return STR_RES_NO_CONTENT;
}

const std::string shiori::Agent::Get(const std::string& req)
{
    FUNC_START(cp);

    asend(reqBuf, RequestItem(REQUEST_GET, req));
    auto res = receive(resBuf);
    if (res.resType == RESPONSE_ERROR){
        throw std::exception(res.value.c_str());
    }
    return res.value;
}

void shiori::Agent::Response(const std::string& res)
{
    FUNC_START(cp);

    asend(resBuf, ResponseItem(res, RESPONSE_NORMAL));
    hasResponse = false;
}

const std::string shiori::Agent::Request(const std::string& req)
{
    FUNC_START(cp);
    try{
        // �G�[�W�F���g�����ɏI�����Ă���Ȃ��O�B
        if (!IsRunning()) THROW_EX("Agent not running");

        // SHIORI REQUEST�����
        auto text = req.c_str();
        auto match = matchShioriRequest(text);

        // ��͂Ɏ��s
        if (match.empty())      return STR_RES_BAT_REQUEST;
        if (match.size() < 2)   THROW_EX("matchShioriRequest INTERNAL ERROR");

        // GET
        std::string reqType(match[1].first, match[1].second);
        if (reqType == "GET")      return Get(req);
        if (reqType == "NOTIFY")   return Notify(req);
        THROW_EX("unmatch request type");
    }
    catch (const std::exception& ex){
        std::string mes(STR_RES_SERVER_ERROR);
        mes += "X-PASTA-Resion: ";
        mes += ex.what();
        mes += "\r\n\r\n";
        return mes;
    }
    catch (...){
        std::string mes(STR_RES_SERVER_ERROR);
        mes += "X-PASTA-Resion: ";
        mes += "NOT std::exception fail";
        mes += "\r\n\r\n";
        return mes;
    }
}

//============================================================
// SHIORI�{�̑��̔񓯊����C�����[�v
//============================================================
void shiori::Agent::run(){
    FUNC_START(cp);

    try{
        // load����
        try{
            DEBUG_MESSAGE(L"WAIT load");
            auto req = receive(reqBuf);
            DEBUG_MESSAGE(L"CALL LoadAction()");
            LoadAction();
            DEBUG_MESSAGE(L"END  LoadAction()");
        }
        catch (const std::exception& ex){ SetException(ex); }
        catch (...)                     { SetException(); }

        // ���C�����[�v
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
                    // GetAction����SHIORI���X�|���X��Ԃ�����
                    hasResponse = true;
                    DEBUG_MESSAGE(L"CALL GetAction()");
                    GetAction(req.value);
                    DEBUG_MESSAGE(L"END  GetAction()");

                    // GetAction����Response���Ăяo����Ă��Ȃ��ꍇ�͗�O�Ƃ���B
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

        // unload����
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

bool shiori::Agent::IsRunning(){
    switch (status()){
    case concurrency::agent_runnable:
    case concurrency::agent_started:
        return true;
    }
    return false;
}

//============================================================
// ��O����
//============================================================

void shiori::Agent::SetException(const std::exception& ex){
    FUNC_START(cp);

    DEBUG_MESSAGE(ex.what());
    last_error_what = ex.what();
}

void shiori::Agent::SetException(){
    FUNC_START(cp);

    SetException(std::exception("(none)"));
}

void shiori::Agent::SendException(){
    FUNC_START(cp);

    auto res = ResponseItem(last_error_what, RESPONSE_ERROR);
    asend(resBuf, res);
}

// EOF