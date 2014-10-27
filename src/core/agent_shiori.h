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
        explicit ResponseItem(const std::wstring& res, const ResponseType tp) :value(res), resType(tp){}
        const ResponseType resType;
        const std::wstring value;
    };

    // SHIORI 非同期エージェント
    class Agent : public concurrency::agent
    {
    public:
        explicit Agent();
        explicit Agent(concurrency::Scheduler& scheduler);
        explicit Agent(concurrency::ScheduleGroup& group);
        virtual  ~Agent();

    public:
        void Load(const HINSTANCE hinst, const int cp, const std::wstring& dir);
        void UnLoad();

        const std::wstring Request(const std::wstring& req);
        const std::wstring Notify(const std::wstring& req);
        const std::wstring Get(const std::wstring& req);

        void Response(const std::wstring& res);

    protected:
        void run();

    public:
        virtual void LoadAction() = 0;
        virtual void UnLoadAction() = 0;
        virtual void NotifyAction(const std::wstring& req) = 0;
        virtual void GetAction(const std::wstring& req) = 0;

    public:
        HINSTANCE hinst;                // SHIORI.DLLのインスタンス
        int cp;                         // コードページ（UTF-8）
        std::tr2::sys::wpath loaddir;   // SHIORI.DLLのディレクトリ

    private:
        bool hasUnload;
        bool hasResponse = false;
        concurrency::unbounded_buffer<RequestItem> reqBuf;
        concurrency::unbounded_buffer<ResponseItem> resBuf;
        std::wstring last_error_what;
        bool IsRunning();

        void SetException(const std::exception& ex);
        void SetException();
        void SendException();
        const ResponseItem GetErrorResponse();
    };
}