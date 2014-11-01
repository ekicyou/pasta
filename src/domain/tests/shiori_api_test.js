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

    //---------------------------------------------------------
    test("shiori_api::test1", function () {


        equal(8, 8);
    });


    //---------------------------------------------------------
    // モジュールのエクスポート
    return mod;
});
