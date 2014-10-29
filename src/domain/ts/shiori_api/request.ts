// shiori request管理
// リクエストの解釈など

import IF = require('../interfaces');

var logger = new Duktape.Logger();


/// リクエスト分解


var IDENTIFIER = "([$a-zA-Z_][$0-9a-zA-Z_-]*)";
var CRLF = "\\r\\n";

var SHIORI_VER = "SHIORI/3.0";
var SHIORI_HEADER = IDENTIFIER + " " + SHIORI_VER + CRLF;
var SHIORI_VALUE = IDENTIFIER + ": (.*?)" + CRLF

var SHIORI_REQUEST = "^" + SHIORI_HEADER + "(" + SHIORI_VALUE + ")*" + CRLF + "$";

var re = new RegExp(SHIORI_REQUEST);
re.compile();



/// リクエスト管理
export class request implements IF.shiori_request {
    public constructor(text: string, res_func: (res: string) => void) {
        this.raw = text;
        this.response = res_func;
        this.match = re.exec(text);
    }

    /// 生リクエスト
    public raw: string;

    /// 正規表現分割オブジェクト
    public match: RegExpExecArray;

    /// GETリクエストの場合、応答を返す。
    public response: (res: string) => void;
}