/// <reference path="shiori_test_data.js" />
// shiori_events のテスト

(function (definition) {
    // CommonJS/RequireJS/<script>
    if (typeof exports === "object") module.exports = definition();
    else if (typeof define === "function" && define.amd) define(definition);
    else shiori_events_test = definition();
})(function () {
    // 実際の定義を行う関数
    'use strict';
    var testData;
    var mod = {};
    var api = require("shiori_api");
    var shiori = require("shiori");
    var testData = require("shiori_test_data");

    var CRLF = "\r\n";
    var paths = ["/user/init/"];

    // ライブラリ設定[libfs]
    var libfs = {};
    libfs.readuser = function (fname) {
        var rc = require_simple.findFile(paths, fname);
        return rc;
    };
    libfs.writeuser = function (fname, buf) {
        console.trace("[writeuser]\n<<<< file=" + fname + " >>>>\n" + buf + "\n<<<< ここまで >>>>");
    };

    // ライブラリ設定[libshiori]
    var lastRes;
    var libshiori = {};
    libshiori.response = function (res) {
        lastRes = res;
    };

    // ライブラリ設定[Duktape]
    var Duktape = {};
    Duktape.version = 10000;


    //---------------------------------------------------------
    test("shiori_events::load save test", function () {
        window.libfs = libfs;
        window.libshiori = libshiori;
        window.Duktape = Duktape;

        // 実行
        shiori.load(null);

        var data = testData.data;
        data.forEach(function (item, index, array) {
            var req = item.req;
            var res = item.res;
            lastRes = null;
            if (0 === req.indexOf("GET")) {
                shiori.get(req);
                equal(lastRes, res, req);
            } else {
                shiori.notify(req);
            }
        });

        shiori.unload();
    });
    //---------------------------------------------------------
    // テストデータ

    //---------------------------------------------------------
    // モジュールのエクスポート
    return mod;
});