#pragma once

#include <windows.h>
#include <string>
#include <filesystem>
#include <agents.h>

namespace shiori{
    // SHIORI REQUEST種別
    enum RequestType {
        REQUEST_NOTIFY,		// 通知のみのリクエスト、応答なし。
        REQUEST_GET,		// 値取得するリクエスト、応答あり。
        REQUEST_LOAD,		// Loadリクエスト      、応答なし。
        REQUEST_UNLOAD,		// Unloadリクエスト    、応答なし、エージェント終了を待つ。
    };

    // SHIORI RESPONSE種別
    enum ResponseType {
        RESPONSE_NORMAL,    // 通常レスポンス。そのままSHIORIに返す。
        RESPONSE_ERROR,		// 例外。内部サーバエラーに変換する。
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

    // SHIORI 非同期エージェント
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
        const HINSTANCE hinst;          // SHIORI.DLLのインスタンス
        const int cp;                   // コードページ（UTF-8）
        std::tr2::sys::wpath loaddir;   // SHIORI.DLLのディレクトリ

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