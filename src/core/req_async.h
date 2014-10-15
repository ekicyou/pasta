#pragma once

#include <windows.h>
#include <agents.h>
#include <string>

namespace shiori{

	enum RequestType {
        REQUEST_NOTIFY,		// �ʒm�݂̂̃��N�G�X�g�A������҂����ɏ�����Ԃ��B
        REQUEST_GET,		// �l�擾���郊�N�G�X�g�A������҂B
        REQUEST_LOAD,		// Load���N�G�X�g      �A������҂����ɏ�����Ԃ��B
        REQUEST_UNLOAD,		// Unload���N�G�X�g    �A������҂B
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