// Duktapeモジュール（ダミー）

(function (definition) {
    // CommonJS/RequireJS/<script>
    if (typeof exports === "object") module.exports = definition();
    else if (typeof define === "function" && define.amd) define(definition);
    else Duktape = definition();
})(function () {
    // 実際の定義を行う関数
    'use strict';
    var mod = {};

    //
    mod.Logger = function () {
        return console;
    }

    // モジュールのエクスポート
    return mod;
});