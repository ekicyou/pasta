#pragma once

#include <windows.h>
#include <agents.h>
#include <string>

namespace shiori{

	enum RequestType {
        REQUEST_NOTIFY,		// 通知のみのリクエスト、応答を待たずに処理を返す。
        REQUEST_GET,		// 値取得するリクエスト、応答を待つ。
        REQUEST_LOAD,		// Loadリクエスト      、応答を待たずに処理を返す。
        REQUEST_UNLOAD,		// Unloadリクエスト    、応答を待つ。
    };


	class Request{
	public:
		explicit Request(const RequestType tp, const std::wstring& req)
			:reqType(tp), value(req){}
		const RequestType reqType;
		const std::wstring value;
	};

	class Response{
    public:
        explicit Response(const std::wstring& res)
			:value(res){}
		const std::wstring value;
	};


	class ShioriAgent : public concurrency::agent
	{
	public:
		explicit ShioriAgent();
		explicit ShioriAgent(concurrency::Scheduler& scheduler);
		explicit ShioriAgent(concurrency::ScheduleGroup& group);

        void Notify(const std::wstring& req);
        std::wstring Get(const std::wstring& req);
        void Load(const std::wstring& dir);
        void UnLoad();

	private:
		concurrency::unbounded_buffer<Request> reqBuf;
		concurrency::unbounded_buffer<Response> resBuf;
	};


}