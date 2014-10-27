#pragma once

#include <windows.h>
#include <string>
#include <filesystem>
#include <agents.h>

namespace shiori{
    // SHIORI REQUEST���
    enum RequestType {
        REQUEST_NOTIFY,		// �ʒm�݂̂̃��N�G�X�g�A�����Ȃ��B
        REQUEST_GET,		// �l�擾���郊�N�G�X�g�A��������B
        REQUEST_LOAD,		// Load���N�G�X�g      �A�����Ȃ��B
        REQUEST_UNLOAD,		// Unload���N�G�X�g    �A�����Ȃ��A�G�[�W�F���g�I����҂B
    };

    // SHIORI RESPONSE���
    enum ResponseType {
        RESPONSE_NORMAL,    // �ʏ탌�X�|���X�B���̂܂�SHIORI�ɕԂ��B
        RESPONSE_ERROR,		// ��O�B�����T�[�o�G���[�ɕϊ�����B
    };

    // SHIORI REQUEST
    class RequestItem{
    public:
        explicit RequestItem(const RequestType tp, const std::string& req)
            :reqType(tp), value(req){}
        explicit RequestItem(const RequestType tp)
            :reqType(tp){}
        const RequestType reqType;
        const std::string value;
    };

    // SHIORI RESPONSE
    class ResponseItem{
    public:
        explicit ResponseItem(const std::string& res, const ResponseType tp) :value(res), resType(tp){}
        const ResponseType resType;
        const std::string value;
    };

    // SHIORI �񓯊��G�[�W�F���g
    class Agent : public concurrency::agent
    {
    public:
        explicit Agent(const int cp, const HINSTANCE hinst);
        explicit Agent(const int cp, const HINSTANCE hinst, concurrency::Scheduler& scheduler);
        explicit Agent(const int cp, const HINSTANCE hinst, concurrency::ScheduleGroup& group);
        virtual  ~Agent();

    public:
        void Load(const std::wstring& dir);
        void UnLoad();

        const std::string Request(const std::string& req);
        const std::string Notify(const std::string& req);
        const std::string Get(const std::string& req);

        void Response(const std::string& res);

    protected:
        void run();

    public:
        virtual void LoadAction() = 0;
        virtual void UnLoadAction() = 0;
        virtual void NotifyAction(const std::string& req) = 0;
        virtual void GetAction(const std::string& req) = 0;

    public:
        const HINSTANCE hinst;          // SHIORI.DLL�̃C���X�^���X
        const int cp;                   // �R�[�h�y�[�W�iUTF-8�j
        std::tr2::sys::wpath loaddir;   // SHIORI.DLL�̃f�B���N�g��

    private:
        bool hasUnload;
        bool hasResponse = false;
        concurrency::unbounded_buffer<RequestItem> reqBuf;
        concurrency::unbounded_buffer<ResponseItem> resBuf;
        std::string last_error_what;
        bool IsRunning();

        void SetException(const std::exception& ex);
        void SetException();
        void SendException();
        const ResponseItem GetErrorResponse();
    };
}