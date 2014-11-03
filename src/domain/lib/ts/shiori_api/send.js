// shiori response
exports.Charset = "UTF-8";
exports.Sender = "pasta";
exports.last_response = null;

var CRLF = "\r\n";

// レスポンスが必要な状態ならtrue。
function hasResponse() {
    return exports.last_response == null;
}
exports.hasResponse = hasResponse;

// レスポンス情報をリセットし、レスポンスが必要な状態とする。
function reset() {
    exports.last_response = null;
}
exports.reset = reset;

function res(code, values) {
    var res = "SHIORI/3.0 " + code + CRLF + "Charset: " + exports.Charset + CRLF + "Sender: " + exports.Sender + CRLF;
    if (values)
        res += values + CRLF;
    res += CRLF;

    if (!exports.hasResponse())
        throw new Error("responseは既に送信済みです。このレスポンスは破棄されます。\n" + res);
    libshiori.response(res);
    exports.last_response = res;
}

function res400BadRequest() {
    res("400 Bad Request", "");
}
exports.res400BadRequest = res400BadRequest;

function res204NoContent() {
    res("204 No Content", "");
}
exports.res204NoContent = res204NoContent;

function res200Ok(values) {
    var text = values.join(CRLF);
    res("200 OK", text);
}
exports.res200Ok = res200Ok;

function resValue(value) {
    var text = "Value: " + value;
    res("200 OK", text);
}
exports.resValue = resValue;

function res500message(mes) {
    var text = "X-PASTA-ERROR: " + mes;
    res("500 Internal Server Error", text);
}
exports.res500message = res500message;

function res500error(e) {
    exports.res500message(e.message);
}
exports.res500error = res500error;
//# sourceMappingURL=send.js.map
