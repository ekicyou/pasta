#pragma once

#include <windows.h>
#include <string>
#include <agents.h>

namespace shiori{
	
    // SHIORI REQUEST���
    enum RequestType {
		REQUEST_NOTIFY,		// �ʒm�݂̂̃��N�G�X�g�A�����Ȃ��B
		REQUEST_GET,		// �l�擾���郊�N�G�X�g�A��������B
		REQUEST_LOAD,		// Load���N�G�X�g      �A�����Ȃ��B
		REQUEST_UNLOAD,		// Unload���N�G�X�g    �A�����Ȃ��A�G�[�W�F���g�I����҂B
	};

	// SHIORI REQUEST
	class RequestItem{
	public:
        explicit RequestItem(const RequestType tp, const std::wstring& req)
            :reqType(tp), value(req){}
        explicit RequestItem(const RequestType tp)
            :reqType(tp){}
        const RequestType reqType;
		const std::wstring value;
	};

	
    // SHIORI RESPONSE
    class ResponseItem{
	public:
		explicit ResponseItem(const std::wstring& res) :value(res), isError(false){}
		explicit ResponseItem(const std::exception& ex) :ex(ex), isError(true){}
		const bool isError;
		const std::wstring value;
		const std::exception ex;
	};

	// SHIORI �񓯊��G�[�W�F���g
	class Agent : public concurrency::agent
	{
	public:
		explicit Agent();
		explicit Agent(concurrency::Scheduler& scheduler);
		explicit Agent(concurrency::ScheduleGroup& group);

		virtual  ~Agent();

        void Load(const HINSTANCE hinst, const int cp, const std::wstring& dir);
        void UnLoad();

        void Notify(const std::wstring& req);
		const std::wstring Get(const std::wstring& req);

	protected:
		void run();

    public:
        virtual void LoadAction() = 0;
		virtual void UnLoadAction() = 0;
		virtual void NotifyAction(const std::wstring& req) = 0;
		virtual const std::wstring GetAction(const std::wstring& req) = 0;
		virtual void GetAfterAction() = 0;

    public:
        HINSTANCE hinst;                // SHIORI.DLL�̃C���X�^���X
        int cp;                         // �R�[�h�y�[�W�iUTF-8�j
        std::tr2::sys::wpath loaddir;   // SHIORI.DLL�̃f�B���N�g��

	private:
		bool isUnload;
        concurrency::unbounded_buffer<RequestItem> reqBuf;
        concurrency::unbounded_buffer<ResponseItem> resBuf;
        std::exception last_error;

		void SetException(const std::exception& ex);
		void SetException();
		void SendException();
		const ResponseItem GetErrorResponse();
	};
}