#pragma once

#include <windows.h>
#include <string>

//-------------------------------------------------------------
// ユーティリティ関数：ユニーク変数名
//-------------------------------------------------------------
#define CAT_IMPL(s1, s2) s1##s2
#define CAT(s1, s2) CAT_IMPL(s1, s2)

#ifdef __COUNTER__
#define GEN_ID(str) CAT(str, __COUNTER__)
#else
#define GEN_ID(str) CAT(str, __LINE__)
#endif

//-------------------------------------------------------------
// ユーティリティ関数：文字変換
//-------------------------------------------------------------

// std::string → std::wstring（ロケール依存）
inline std::wstring ToWideStr(const std::string &str)
{
    USES_CONVERSION;
    return A2CW(str.c_str());
}
// std::wstring → std::string（ロケール依存）
inline std::string ToMultStr(const std::wstring &wstr)
{
    USES_CONVERSION;
    return W2CA(wstr.c_str());
}

// std::string → std::wstring（コードページ指定）
inline std::wstring ToWideStr(const std::string &str, int cp)
{
    USES_CONVERSION;
    return A2CW_CP(str.c_str(), cp);
}
// std::wstring → std::string（コードページ指定）
inline std::string ToMultStr(const std::wstring &wstr, int cp)
{
    USES_CONVERSION;
    return W2CA_CP(wstr.c_str(), cp);
}

#define A2CW_UTF8(_str_) (A2CW_CP(_str_,CP_UTF8))

//-------------------------------------------------------------
// ユーティリティ関数：例外出力
//-------------------------------------------------------------

// メソッド名付きでstd::exceptionを発行します。
void ThrowStdException(LPCSTR funcname, LPCSTR  what);
void ThrowStdException(LPCSTR funcname, LPCWSTR what);

// メソッド名付きでstd::exceptionを発行します。
#define THROW_EX(what)  ThrowStdException(__FUNCTION__,what)

// 未実装
#define NOT_IMPLMENT    THROW_EX("not implment")

//-------------------------------------------------------------
// スコープが外れたときに実行する関数
//-------------------------------------------------------------

class DisposeLambda{
public:
    DisposeLambda(const std::tr1::function<void(void)> func)
        :dispose(func){}
    ~DisposeLambda(){ dispose(); }

private:
    const std::tr1::function<void(void)> dispose;
};

#define DISPOSE_LAMBDA(lambda)                              \
    DisposeLambda GEN_ID(_dispose_lambda_)(lambda)

#define AUTO_CLOSE(file)                                    \
    DISPOSE_LAMBDA( [file](){if (file) fclose(file); } )

//-------------------------------------------------------------
// ユーティリティ関数：ログ出力関係
//-------------------------------------------------------------

class FunctionInOutDebugLog{
public:
    FunctionInOutDebugLog(const int cp, LPCSTR funcname);
    ~FunctionInOutDebugLog();

    void OutputLog(LPCSTR message);
    void OutputLog(LPCWSTR message);

    void OutputRaw(LPCSTR message);
    void OutputRaw(LPCWSTR message);

private:
    std::wstring funcName;
    const int cp;
};

#ifdef DEBUG
#define FUNC_START(cp)  FunctionInOutDebugLog __func_start_debuglog__(cp,__FUNCTION__);
#else
#define FUNC_START(cp)  ;
#endif

#ifdef DEBUG
#define DEBUG_MESSAGE(mes)  __func_start_debuglog__.OutputLog(mes);
#else
#define DEBUG_MESSAGE(mes)  ;
#endif

#ifdef DEBUG
#define DEBUG_RAW_MESSAGE(mes)  __func_start_debuglog__.OutputRaw(mes);
#else
#define DEBUG_RAW_MESSAGE(mes)  ;
#endif
