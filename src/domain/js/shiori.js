// ルートプログラム
// 必要なスクリプトを読み込み、
//  [shiori.load(dir)]
//  [shiori.unload()]
//  [shiori.get(req)]
//  [shiori.notify(req)]
// の各関数をフックする。


(function (definition) {// 定義する関数を引数にとる
    // ロードされた文脈に応じてエクスポート方法を変える

    // CommonJS
    if (typeof exports === "object") {
        module.exports = definition();

        // RequireJS
    } else if (typeof define === "function" && define.amd) {
        define(definition);

        // <script>
    } else {
        shiori = definition();
    }

})(function () {// 実際の定義を行う関数
    'use strict';

    var mod = function() { };

    //---------------------------------------------------------
    // モジュール
    var logger = new Duktape.Logger();

    //---------------------------------------------------------
    // レスポンス処理関数
    var hasResponse = false;
    var response = function (res) {
        if (!hasResponse) {
            logger.error("response(): multiple call");
            return;
        }
        hasResponse = false;
        shiorilib.response(res);
    };

    //---------------------------------------------------------
    // SHIORI LOAD
    mod.load = function (dir) {
        try {
            logger.debug("load: start");
            logger.debug("loaddir=" + dir);
            mod.loaddir = dir;


        }
        catch (e) {
            logger.error(e);
        }
        finally {
            logger.debug("load: fin");
        }
    };

    //---------------------------------------------------------
    // SHIORI UNLOAD
    mod.unload = function () {
        try {
            logger.debug("unload: start");
            // TODO: シャットダウン処理の呼び出し


        }
        catch (e) {
            logger.error(e);
        }
        finally {
            logger.debug("unload: fin");
        }
    };

    //---------------------------------------------------------
    // SHIORI NOTIFY
    mod.notify = function (req) {
        try {
            logger.debug("notify: start");
            logger.debug(req);
            // TODO: NOTIFY処理


        }
        catch (e) {
            logger.error(e);
        }
        finally {
            logger.debug("notify: fin");
        }
    };

    //---------------------------------------------------------
    // SHIORI GET
    mod.get = function (req) {
        hasResponse = true;
        try {
            logger.debug("get: start");
            logger.debug(req);

            // TODO: GET処理
            response("SHIORI/3.0 200 OK\r\n\r\n");

        }
        catch (e) {
            logger.error(e);
        }
        finally {
            if (hasResponse) {
                // TODO: レスポンス漏れ

            }
            logger.debug("get: fin");
        }
    };

    logger.info("loaded");

    return mod;
});
