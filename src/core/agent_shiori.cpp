// req_async.cpp : �񓯊��G�[�W�F���g�Ƃ���SHIORI API�ƒʐM���܂��B
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "util.h"

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
    this->hinst = hinst;
    this->cp = cp;
    this->loaddir = dir;
    asend(reqBuf, RequestItem(REQUEST_LOAD));
}

void shiori::Agent::UnLoad()
{
    if (isUnload)return;
    asend(reqBuf, RequestItem(REQUEST_UNLOAD));
    wait(this);
    isUnload = true;
}

void shiori::Agent::Notify(const std::wstring& req)
{
    asend(reqBuf, RequestItem(REQUEST_NOTIFY, req));
}

const std::wstring shiori::Agent::Get(const std::wstring& req)
{
    asend(reqBuf, RequestItem(REQUEST_GET, req));
    auto res = receive(resBuf);
    if (res.isError) throw  res.ex;
    else             return res.value;
}

void shiori::Agent::Response(const std::wstring& res)
{
    asend(resBuf, ResponseItem(res));
    hasResponse = false;
}



//============================================================
// SHIORI�{�̑��̔񓯊����C�����[�v
//============================================================
void shiori::Agent::run(){
    try{
        // load����
        try{
            auto req = receive(reqBuf);
            LoadAction();
        }
        catch (const std::exception& ex){ SetException(ex); }
        catch (...)                     { SetException(); }

        // ���C�����[�v
        while (true){
            auto req = receive(reqBuf);
            switch (req.reqType)
            {

            case shiori::REQUEST_NOTIFY:
                try{
                    NotifyAction(req.value);
                }
                catch (const std::exception& ex){ SetException(ex); }
                catch (...)                     { SetException(); }
                continue;

            case shiori::REQUEST_GET:
                try{
                    // GetAction����SHIORI���X�|���X��Ԃ�����
                    hasResponse = true;
                    GetAction(req.value);

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
            UnLoadAction();
        }
        catch (const std::exception& ex){ SetException(ex); }
        catch (...)                     { SetException(); }
    }
    catch (const std::exception& ex){ SetException(ex); }
    catch (...)                     { SetException(); }
    done();
}


//============================================================
// ��O����
//============================================================


void shiori::Agent::SetException(const std::exception& ex){
    last_error = ex;
}

void shiori::Agent::SetException(){
    SetException(std::exception("(none)"));
}

void shiori::Agent::SendException(){
    auto res = ResponseItem(last_error);
    asend(resBuf, res);
}

