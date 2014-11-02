// shiori request管理
// リクエストの解釈など

import IF = require('../interfaces');

var logger = new Duktape.Logger();

/// regのエスケープ
function quote(str: string, delimiter: string= '') {
    return (str + '').replace(new RegExp('[.\\\\+*?\\[\\^\\]$(){}=!<>|:\\' + (delimiter || '') + '-]', 'g'), '\\$&');
}

/// リクエスト分解
var IDENTIFIER = "([$a-zA-Z_][-0-9$a-zA-Z_]*)";
var CRLF = "\\r\\n";

var SHIORI_VER = quote("SHIORI/3.0");
var SHIORI_HEADER = IDENTIFIER + " " + SHIORI_VER + CRLF;
var SHIORI_VALUE = IDENTIFIER + quote(": ") + "(.*)" + "(" + CRLF + ")?";

var reHeader = new RegExp("^" +SHIORI_HEADER);
var reValue = new RegExp("^" + SHIORI_VALUE);
//var re = re_base.compile();

/// KeyValueアイテム
export class keyvalue {
    public constructor(key: string, value: string) {
        this.key = key;
        this.value = value;
    }
    public key: string;
    public value: string;
}


/// リクエスト管理
export class request implements IF.shiori_request {
    public constructor(text: string, res_func: (res: string) => void) {
        this.response = res_func;
        this.parse(text);
    }

    /// リクエスト分解
    private parse(text: string): void {
        this.raw = text;
        this.time = Date.now();

        // メソッド取得まで
        var m = this.match = text.match(reHeader);
        if (!m) return;
        this.method = m[1];

        // 値の取得
        var lines = text.split("\r\n");
        var len = lines.length;
        var list: keyvalue[] = [];

        for (var index = 1; index < len; index++) {
            var m = lines[index].match(reValue);
            if (!m) break;
            var kv = new keyvalue(m[1], m[2]);
            list.push(kv);
        }
        this.kvlist = list;

        // map化
        var map: any = {};
        list.forEach((item, i, a) => map[item.key] = item.value);
        this.map = map;
    }

    /// GETリクエストの場合、応答を返すための関数。
    public response: (res: string) => void;

    /// 生リクエスト
    public raw: string;

    /// 時刻
    public time: number;

    /// 正規表現分割オブジェクト
    public match: string[];

    /// 解決できた場合、SHIORI メソッド(GET/NOTIFY)
    public method: string;

    /// <key/value> list
    public kvlist: keyvalue[];

    /// <key/value> map
    public map: any;

    
}