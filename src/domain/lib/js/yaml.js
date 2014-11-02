// yaml.js

(function (definition) {
    // CommonJS/RequireJS/<script>
    if (typeof exports === "object") module.exports = definition();
    else if (typeof define === "function" && define.amd) define(definition);
    else yaml = definition();
})(function () {
    // 実際の定義を行う関数
    'use strict';
    var mod = require("./yaml/Yaml");

    //---------------------------------------------------------
    // モジュールのエクスポート
    return mod;
});