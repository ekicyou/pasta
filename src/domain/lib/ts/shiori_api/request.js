// shiori request管理
// リクエストの解釈など
var logger = new Duktape.Logger();

/// regのエスケープ
function quote(str, delimiter) {
    if (typeof delimiter === "undefined") { delimiter = ''; }
    return (str + '').replace(new RegExp('[.\\\\+*?\\[\\^\\]$(){}=!<>|:\\' + (delimiter || '') + '-]', 'g'), '\\$&');
}

/// リクエスト分解
var IDENTIFIER = "([$a-zA-Z_][-0-9$a-zA-Z_]*)";
var CRLF = "\\r\\n";

var SHIORI_VER = quote("SHIORI/3.0");
var SHIORI_HEADER = IDENTIFIER + " " + SHIORI_VER + CRLF;
var SHIORI_VALUE = IDENTIFIER + quote(": ") + "(.*?)" + CRLF;

var SHIORI_REQUEST = "^" + SHIORI_HEADER + "(?:" + SHIORI_VALUE + ")*";

var re = new RegExp(SHIORI_REQUEST);

/// KeyValueアイテム
var keyvalue = (function () {
    function keyvalue(key, value) {
        this.key = key;
        this.value = value;
    }
    return keyvalue;
})();
exports.keyvalue = keyvalue;

/// リクエスト管理
var request = (function () {
    function request(text, res_func) {
        this.response = res_func;
        this.parse(text);
    }
    /// リクエスト分解
    request.prototype.parse = function (text) {
        this.raw = text;

        // メソッド取得まで
        var m = this.match = text.match(re);
        if (!m)
            return;
        this.method = m[1];

        // 値の取得
        var len = m.length;
        var list = [];

        var index = 2;
        while (index + 1 < len) {
            var kv = new keyvalue(m[index], m[index + 1]);
            list.push(kv);
            index += 2;
        }
        this.kvlist = list;

        // map化
        var map = {};
        list.forEach(function (item, i, a) {
            map[item.key] = item.value;
        });
        this.map = map;

        logger.trace(this);
    };
    return request;
})();
exports.request = request;
//# sourceMappingURL=request.js.map
