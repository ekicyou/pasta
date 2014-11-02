// テスト

(function (definition) {
    console.info("start test1");
    // CommonJS/RequireJS/<script>
    if (typeof exports === "object") module.exports = definition();
    else if (typeof define === "function" && define.amd) define(definition);
    else hello = definition();
})(function () {
    // 実際の定義を行う関数
    'use strict';
    console.info("start test3");
    var mod = {};
    mod.hello = function () {
    };

    window.Duktape = require("./duktape");
    var api = require("shiori_api");

    test("test1", function () {
        var version = 8;
        equal(version, 8);
    });

    // モジュールのエクスポート
    return mod;
});

console.info("start test2");