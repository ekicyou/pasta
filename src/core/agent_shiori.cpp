// req_async.cpp : �񓯊��G�[�W�F���g�Ƃ���SHIORI API�ƒʐM���܂��B
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "shiori_parse.h"
#include "util.h"
#include <regex>

//============================================================
// ������
//============================================================
shiori::Agent::Agent()
    :agent(), isUnload(false)
{
}

shiori::Agent::Agent(concurrency::Scheduler& scheduler)
    : agent(scheduler), isUnload(false)
{
}

shiori::Agent::Agent(concurrency::ScheduleGroup& group)
    : agent(group), isUnload(false)
{
}

//============================================================
// ���
//============================================================
shiori::Agent::~Agent()
{
    UnLoad();
}

//============================================================
// SHIORI API����
//============================================================
void shiori::Agent::Load(const HINSTANCE hinst, const int cp, const std::wstring& dir)
{
    OutputDebugString(L"[shiori::Agent::Load]START\n");
    this->hinst = hinst;
    this->cp = cp;
    this->loaddir = dir;
    asend(reqBuf, RequestItem(REQUEST_LOAD));
    OutputDebugString(L"[shiori::Agent::Load]END\n");
}

void shiori::Agent::UnLoad()
{
    if (isUnload)return;
    OutputDebugString(L"[shiori::Agent::UnLoad]START\n");
    asend(reqBuf, RequestItem(REQUEST_UNLOAD));
    wait(this);
    isUnload = true;
    OutputDebugString(L"[shiori::Agent::UnLoad]END\n");
}

void shiori::Agent::Notify(const std::wstring& req)
{
    OutputDebugString(L"[shiori::Agent::Notify]START\n");
    asend(reqBuf, RequestItem(REQUEST_NOTIFY, req));
    OutputDebugString(L"[shiori::Agent::Notify]END\n");
}

const std::wstring shiori::Agent::Get(const std::wstring& req)
{
    OutputDebugString(L"[shiori::Agent::Get]START\n");
    asend(reqBuf, RequestItem(REQUEST_GET, req));
    auto res = receive(resBuf);
    if (res.isError) throw  res.ex;
    else             return res.value;
    OutputDebugString(L"[shiori::Agent::Get]END\n");
}

void shiori::Agent::Response(const std::wstring& res)
{
    OutputDebugString(L"[shiori::Agent::Response]START\n");
    asend(resBuf, ResponseItem(res));
    hasResponse = false;
    OutputDebugString(L"[shiori::Agent::Response]END\n");
}

const std::wstring shiori::Agent::Request(const std::wstring& req)
{
    OutputDebugString(L"[shiori::Agent::Request]START\n");
    // SHIORI REQUEST�����
    auto text = req.c_str();
    auto match = matchShioriRequest(text);

    // ��͂Ɏ��s
    if (match.empty())      throw std::exception("NOT SHIORI/3.0 REQUEST");
    if (match.size() < 2)   throw std::exception("matchShioriRequest INTERNAL ERROR");

    // GET



    OutputDebugString(L"[shiori::Agent::Request]END\n");
}


//============================================================
// SHIORI�{�̑��̔񓯊����C�����[�v
//============================================================
void shiori::Agent::run(){
    try{
        OutputDebugString(L"[shiori::Agent::run]START\n");
        // load����
        try{
            OutputDebugString(L"[shiori::Agent::run]WAIT load\n");
            auto req = receive(reqBuf);
            OutputDebugString(L"[shiori::Agent::run]CALL LoadAction()\n");
            LoadAction();
            OutputDebugString(L"[shiori::Agent::run]END  LoadAction()\n");
        }
        catch (const std::exception& ex){ SetException(ex); }
        catch (...)                     { SetException(); }

        // ���C�����[�v
        while (true){
            OutputDebugString(L"[shiori::Agent::run]WAIT request\n");
            auto req = receive(reqBuf);
            switch (req.reqType)
            {

            case shiori::REQUEST_NOTIFY:
                try{
                    OutputDebugString(L"[shiori::Agent::run]CALL NotifyAction()\n");
                    NotifyAction(req.value);
                    OutputDebugString(L"[shiori::Agent::run]END  NotifyAction()\n");
                }
                catch (const std::exception& ex){ SetException(ex); }
                catch (...)                     { SetException(); }
                continue;

            case shiori::REQUEST_GET:
                try{
                    // GetAction����SHIORI���X�|���X��Ԃ�����
                    hasResponse = true;
                    OutputDebugString(L"[shiori::Agent::run]CALL GetAction()\n");
                    GetAction(req.value);
                    OutputDebugString(L"[shiori::Agent::run]END  GetAction()\n");

                    // GetAction����Response���Ăяo����Ă��Ȃ��ꍇ�͗�O�Ƃ���B
                    if (!hasResponse){
                        throw std::exception("NOT RESPONSE");
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
            OutputDebugString(L"[shiori::Agent::run]CALL UnLoadAction()\n");
            UnLoadAction();
            OutputDebugString(L"[shiori::Agent::run]END  UnLoadAction()\n");
        }
        catch (const std::exception& ex){ SetException(ex); }
        catch (...)                     { SetException(); }
    }
    catch (const std::exception& ex){ SetException(ex); }
    catch (...)                     { SetException(); }

    OutputDebugString(L"[shiori::Agent::run]END\n");
    done();
}


//============================================================
// ��O����
//============================================================


void shiori::Agent::SetException(const std::exception& ex){
#ifdef DEBUG
    {
        USES_CONVERSION;
        std::wstring mes(L"[shiori::Agent::SetException] ");
        mes.append(A2CW_UTF8(ex.what()));
        mes.append(L"\n");
        OutputDebugString(mes.c_str());
    }
#endif
    last_error = ex;
}

void shiori::Agent::SetException(){
    SetException(std::exception("(none)"));
}

void shiori::Agent::SendException(){
    OutputDebugString(L"[shiori::Agent::SendException]\n");
    auto res = ResponseItem(last_error);
    asend(resBuf, res);
}

