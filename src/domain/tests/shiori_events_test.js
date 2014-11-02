// shiori_events のテスト

(function (definition) {
    // CommonJS/RequireJS/<script>
    if (typeof exports === "object") module.exports = definition();
    else if (typeof define === "function" && define.amd) define(definition);
    else shiori_events_test = definition();
})(function () {
    // 実際の定義を行う関数
    'use strict';
    var mod = {};
    var api = require("shiori_api");
    var pasta = require("pasta");

    var CRLF = "\r\n";
    var paths = ["/user/init/"];

    // ライブラリ設定
    var libfs = {};
    libfs.readuser = function (fname) {
        var rc = require_simple.findFile(paths, fname);
        return rc;
    };
    libfs.writeuser = function (fname, buf) {
        console.trace("[writeuser]\n<<<< file=" + fname +" >>>>\n"+ buf +"\n<<<< ここまで >>>>");
    };

    //---------------------------------------------------------
    test("shiori_events::load save test", function () {
        window.libfs = libfs;

        // 実行
        var ghost = new pasta.ghost();
        var events = new api.events(ghost);
        events.load(null);
        console.trace(events.user);

        equal(true, ghost.user.firstload);

        events.unload();
    });

    //---------------------------------------------------------
    // モジュールのエクスポート
    return mod;
});