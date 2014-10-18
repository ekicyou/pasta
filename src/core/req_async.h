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

	class RequestItem{
	public:
		explicit RequestItem(const RequestType tp, const std::wstring& req)
			:reqType(tp), value(req){}
		const RequestType reqType;
		const std::wstring value;
	};

	class ResponseItem{
	public:
		explicit ResponseItem(const std::wstring& res)
			:value(res){}
		const std::wstring value;
	};

	class Agent : public concurrency::agent
	{
	public:
		explicit Agent();
		explicit Agent(concurrency::Scheduler& scheduler);
		explicit Agent(concurrency::ScheduleGroup& group);

		virtual  ~Agent();

		void Notify(const std::wstring& req);
		const std::wstring Get(const std::wstring& req);
		void Load(const std::wstring& dir);
		void UnLoad();

	protected:
		concurrency::unbounded_buffer<RequestItem> reqBuf;
		concurrency::unbounded_buffer<ResponseItem> resBuf;

		void run();

		virtual void LoadAction(const std::wstring& dir) = 0;
		virtual void UnLoadAction() = 0;
		virtual void NotifyAction(const std::wstring& req) = 0;
		virtual const std::wstring GetAction(const std::wstring& req) = 0;
		virtual void GetAfterAction() = 0;

	private:
		bool isUnload;
		std::exception last_error;

		void SetException(const std::exception& ex);
		void SetException();
		const ResponseItem GetErrorResponse();
	};
}