// ルートプログラム
// 必要なスクリプトを読み込み、
//  [Shiori.load(dir)]
//  [Shiori.unload()]
//  [Shiori.get(req)]
//  [Shiori.notify(req)]
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
        Shiori.response(res);
    };


    //---------------------------------------------------------
    // Cookieの準備（最終状態の保存用）
    Shiori.cookie = {};

    //---------------------------------------------------------
    // SHIORI LOAD
    Shiori.load = function (dir) {
        try{
            logger.debug("load: start");
            logger.debug("loaddir=" + dir);
            Shiori.loaddir = dir;

            // TODO: Shiori.cookie の読み込み

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
    Shiori.unload = function () {
        try {
            logger.debug("unload: start");
            // TODO: シャットダウン処理の呼び出し

            // TODO: Shiori.cookie の保存

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
    Shiori.notify = function (req) {
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
    Shiori.get = function (req) {
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