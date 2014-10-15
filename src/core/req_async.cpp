// req_async.cpp : 非同期エージェントとしてSHIORI APIと通信します。
//

#include "stdafx.h"
#include "req_async.h"
#include "app.h"


namespace shiori{

    Agent::Agent()
        :agent(), isUnload(false)
    {
    }

    Agent::Agent(concurrency::Scheduler& scheduler)
        : agent(scheduler), isUnload(false)
    {
    }

    Agent::Agent(concurrency::ScheduleGroup& group)
        : agent(group), isUnload(false)
    {
    }


    Agent::~Agent()
    {
        UnLoad();
    }




    void Agent::Notify(const std::wstring& req)
    {
        asend(reqBuf, RequestItem(REQUEST_NOTIFY, req));
    }

    const std::wstring Agent::Get(const std::wstring& req)
    {
        asend(reqBuf, RequestItem(REQUEST_GET, req));
        auto res = receive(resBuf);
        return res.value;
    }

    void Agent::Load(const std::wstring& dir)
    {
        asend(reqBuf, RequestItem(REQUEST_LOAD, dir));
    }

    void Agent::UnLoad()
    {
        if (isUnload)return;
        asend(reqBuf, RequestItem(REQUEST_UNLOAD, std::wstring()));
        wait(this);
        isUnload = true;
    }


    void Agent::run(){

        try{
            // load処理
            try{
                auto loaddir = receive(reqBuf).value;
                LoadAction(loaddir);
            }
            catch (const std::exception& ex){
                SetException(ex);
            }
            catch (...){
                SetException();
            }

            // メインループ
            while (true){
                auto req = receive(reqBuf);
                switch (req.reqType)
                {
                case shiori::REQUEST_NOTIFY:
                    try{
                        NotifyAction(req.value);
                    }
                    catch (const std::exception& ex){
                        SetException(ex);
                    }
                    catch (...){
                        SetException();
                    }
                    break;
                }

                    case shiori::REQUEST_GET:
                        try{
                            auto value = GetAction(req.value);
                            auto res = ResponseItem(value);
                            asend(resBuf, res);
                        }
                        catch (const std::exception& ex){
                            SetException(ex);
                            asend(resBuf, GetErrorResponse());
                        }
                        catch (...){
                            SetException();
                            asend(resBuf, GetErrorResponse());
                        }

                        try{
                            GetAfterAction();
                        }
                        catch (const std::exception& ex){
                            SetException(ex);
                        }
                        catch (...){
                            SetException();
                        }
                        break;

                    default:
                        break;
            }
        }
    }
   

}
