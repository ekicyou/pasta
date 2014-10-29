// shiori request管理
// リクエストの解釈など
var logger = new Duktape.Logger();

/// リクエスト分解
var IDENTIFIER = "([$a-zA-Z_][$0-9a-zA-Z_-]*)";
var CRLF = "\\r\\n";

var SHIORI_VER = "SHIORI/3.0";
var SHIORI_HEADER = IDENTIFIER + " " + SHIORI_VER + CRLF;
var SHIORI_VALUE = IDENTIFIER + ": (.*?)" + CRLF;

var SHIORI_REQUEST = "^" + SHIORI_HEADER + "(" + SHIORI_VALUE + ")*" + CRLF + "$";

var re = new RegExp(SHIORI_REQUEST);
re.compile();

/// リクエスト管理
var request = (function () {
    function request(text, res_func) {
        this.raw = text;
        this.response = res_func;
        this.match = re.exec(text);
    }
    return request;
})();
exports.request = request;
//# sourceMappingURL=request.js.map
