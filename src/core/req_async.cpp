// req_async.cpp : 非同期エージェントとしてSHIORI APIと通信します。
//

#include "stdafx.h"
#include "req_async.h"
#include "app.h"


namespace shiori{

	ShioriAgent::ShioriAgent(){
	}

	ShioriAgent::ShioriAgent(concurrency::Scheduler& scheduler)
		:agent(scheduler)
	{
	}

	ShioriAgent::ShioriAgent(concurrency::ScheduleGroup& group)
		: agent(group)
	{
	}

	void ShioriAgent::Notify(const std::wstring& req)
	{
        asend(reqBuf, Request(REQUEST_NOTIFY, req));
	}

    std::wstring ShioriAgent::Get(const std::wstring& req)
    {
        asend(reqBuf, Request(REQUEST_GET, req));
        auto res = receive(resBuf);
        return res.value;
    }

    void ShioriAgent::Load(const std::wstring& req)
    {
        asend(reqBuf, Request(REQUEST_LOAD, req));
    }

    void ShioriAgent::UnLoad()
    {
        asend(reqBuf, Request(REQUEST_UNLOAD, std::wstring()));
        wait(this);
    }


}