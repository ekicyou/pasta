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
var SHIORI_VALUE = IDENTIFIER + quote(": ") + "(.*)" + "(" + CRLF + ")?";

var reHeader = new RegExp("^" + SHIORI_HEADER);
var reValue = new RegExp("^" + SHIORI_VALUE);

//var re = re_base.compile();
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
        this.time = Date.now();

        // メソッド取得まで
        var m = this.match = text.match(reHeader);
        if (!m)
            return;
        this.method = m[1];

        // 値の取得
        var lines = text.split("\r\n");
        var len = lines.length;
        var list = [];

        for (var index = 1; index < len; index++) {
            var m = lines[index].match(reValue);
            if (!m)
                break;
            var kv = new keyvalue(m[1], m[2]);
            list.push(kv);
        }
        this.kvlist = list;

        // map化
        var map = {};
        list.forEach(function (item, i, a) {
            return map[item.key] = item.value;
        });
        this.map = map;
    };
    return request;
})();
exports.request = request;
//# sourceMappingURL=request.js.map
