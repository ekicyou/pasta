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
    mod.Logger = (function () {
        var cls = function () {
        }
        cls.prototype.trace = function () { console.trace.apply(this, arguments); };
        cls.prototype.debug = function () { console.debug.apply(this, arguments); };
        cls.prototype.info = function () { console.info.apply(this, arguments); };
        cls.prototype.warn = function () { console.warn.apply(this, arguments); };
        cls.prototype.error = function () { console.error.apply(this, arguments); };
        cls.prototype.fatal = function () { console.fatal.apply(this, arguments); };
        return cls;
    })();


    // モジュールのエクスポート
    return mod;
});
