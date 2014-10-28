// ルートプログラム
// 必要なスクリプトを読み込み、
//  [shiori.load(dir)]
//  [shiori.unload()]
//  [shiori.get(req)]
//  [shiori.notify(req)]
// の各関数をフックする。

'use strict';
(function (global) {

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
        shiori.response(res);
    };

    //---------------------------------------------------------
    // SHIORI LOAD
    shiori.load = function (dir) {
        try{
            logger.debug("load: start");
            logger.debug("loaddir=" + dir);
            shiori.loaddir = dir;


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
    shiori.unload = function () {
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
    shiori.notify = function (req) {
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
    shiori.get = function (req) {
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

})(this);