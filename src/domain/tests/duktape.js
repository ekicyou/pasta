// Duktape / pasta.dll組み込みモジュール（ダミー）

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
    };

    mod.enc = function (fmt, obj, replacer, space) {
        try {
            return JSON.stringify(obj, replacer, space);
        }
        catch (e) {
            console.error(e);
            throw e;
        }
    };

    mod.dec = function (fmt, str) {
        try {
            return JSON.parse(str);
        }
        catch (e) {
            console.error(e);
            console.error(str);
            throw e;
        }
    };

    // モジュールのエクスポート
    return mod;
});