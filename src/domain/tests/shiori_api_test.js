// shiori_api のテスト

(function (definition) {
    // CommonJS/RequireJS/<script>
    if (typeof exports === "object") module.exports = definition();
    else if (typeof define === "function" && define.amd) define(definition);
    else hello = definition();
})(function () {
    // 実際の定義を行う関数
    'use strict';
    var mod = {};
    var api = require("shiori_api");

    var CRLF = "\r\n";

    //---------------------------------------------------------
    test("shiori_api::test1", function () {
        var text = "";
        text += "GET SHIORI/123" + CRLF;
        text += CRLF;
        var req = new api.request(text);
        equal(req.method, undefined);
    });

    //---------------------------------------------------------
    test("shiori_api::test2", function () {
        var text = "";
        text += "GET SHIORI/3.0" + CRLF;
        text += "Charset: UTF-8" + CRLF;
        text += "ID: version" + CRLF;
        text += "SecurityLevel: local" + CRLF;
        text += "Sender: SSP" + CRLF;
        text += CRLF;
        var req = new api.request(text);
        equal(req.method, "GET");
    });

    //---------------------------------------------------------
    // モジュールのエクスポート
    return mod;
});