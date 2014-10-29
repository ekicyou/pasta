// shiori request管理
// リクエストの解釈など

import IF = require('../interfaces');

var logger = new Duktape.Logger();


/// リクエスト分解

/*
#define IDENTIFIER      T("([$a-zA-Z_][$0-9a-zA-Z_-]*)")
#define CRLF            T("\\r\\n")

#define SHIORI_VER      T("SHIORI/3.0")
#define SHIORI_HEADER   IDENTIFIER T(" ") SHIORI_VER CRLF
#define SHIORI_VALUE    IDENTIFIER T(": (.*?)") CRLF

#define SHIORI_REQUEST  T("^") SHIORI_HEADER T("(") SHIORI_VALUE T(")*") CRLF T("$")


*/




/// リクエスト管理
export class request implements IF.shiori_request{
    public constructor(text: string, res_func: (res: string) => void) {
        this.raw = text;
        this.response = res_func;
    }

    /// 生リクエスト
    public raw: string;

    /// GETリクエストの場合、応答を返す。
    public response: (res: string) => void;
}