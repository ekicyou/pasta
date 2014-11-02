// 全テストの呼び出し

(function (definition) {
    // CommonJS/RequireJS/<script>
    if (typeof exports === "object") module.exports = definition();
    else if (typeof define === "function" && define.amd) define(definition);
    else hello = definition();
})(function () {
    // 実際の定義を行う関数
    'use strict';
    window.Duktape = require("duktape");

    // 試験モジュール
    var mod = {};
    mod.shiori_api_test = require("shiori_api_test");

    // モジュールのエクスポート
    return mod;
});