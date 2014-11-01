﻿// shiori request管理
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
var SHIORI_VALUE = IDENTIFIER + quote(": ")+"(.*?)" + CRLF

var SHIORI_REQUEST = "^" + SHIORI_HEADER + "(?:" + SHIORI_VALUE + ")*";

var re = new RegExp(SHIORI_REQUEST);

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

        // メソッド取得まで
        var m = this.match = text.match(re);
        if (!m) return;
        this.method = m[1];

        // 値の取得
        var len = m.length;
        var list: keyvalue[] = [];

        var index = 2;
        while (index+1 < len) {
            var kv = new keyvalue(m[index], m[index + 1]);
            list.push(kv);
            index += 2;
        }
        this.kvlist = list;

        // map化
        var map: any = {};
        list.forEach((item, i, a) => {
            map[item.key] = item.value;
        });
        this.map = map;

        logger.trace(this);
    }

    /// GETリクエストの場合、応答を返すための関数。
    public response: (res: string) => void;

    /// 生リクエスト
    public raw: string;

    /// 正規表現分割オブジェクト
    public match: string[];

    /// 解決できた場合、SHIORI メソッド(GET/NOTIFY)
    public method: string;

    /// key/value リスト
    public kvlist: keyvalue[];

    /// key/value マップ
    public map: any;

    
}