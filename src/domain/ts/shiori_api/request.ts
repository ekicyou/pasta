// shiori request管理
// リクエストの解釈など

import IF = require('../interfaces');
import res = require('./send');

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

/// リクエスト管理
export class request implements IF.shiori_request {
    public constructor(text: string) {
        this.parse(text);
    }

    public static res = res;

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
        var list: IF.keyvalue[] = [];

        for (var index = 1; index < len; index++) {
            var m = lines[index].match(reValue);
            if (!m) break;
            var kv = new IF.keyvalue(m[1], m[2]);
            list.push(kv);
        }
        this.kvlist = list;

        // map化
        var map: any = {};
        list.forEach((item, i, a) => map[item.key] = item.value);
        this.map = map;

        // 固有パラメータ
        this.ID = map.ID;
    }

    /// 生リクエスト
    public raw: string;

    /// 時刻
    public time: number;

    /// 正規表現分割オブジェクト
    public match: string[];

    /// 解決できた場合、SHIORI メソッド(GET/NOTIFY)
    public method: string;

    /// <key/value> list
    public kvlist: IF.keyvalue[];

    /// <key/value> map
    public map: any;

    /// ID:
    public ID: string;
}