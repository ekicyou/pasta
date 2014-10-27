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
            logger.debug(dir);
            Shiori.loaddir = dir;

            // TODO: Shiori.cookie の読み込み
        }
        catch (e) {
            logger.error(e);
        }
        finally {

        }
    };

    //---------------------------------------------------------
    // SHIORI UNLOAD
    Shiori.unload = function () {
        try {
            // TODO: シャットダウン処理の呼び出し

            // TODO: Shiori.cookie の保存

        }
        catch (e) {
            logger.error(e);
        }
        finally {

        }
    };

    //---------------------------------------------------------
    // SHIORI NOTIFY
    Shiori.notify = function (req) {
        try {
            logger.debug(req);
            // TODO: NOTIFY処理


        }
        catch (e) {
            logger.error(e);
        }
        finally {

        }
    };

    //---------------------------------------------------------
    // SHIORI GET
    Shiori.get = function (req) {
        hasResponse = true;
        try {
            logger.debug(req);
            // TODO: GET処理


        }
        catch (e) {
            logger.error(e);
        }
        finally {
            if (hasResponse) {
                // TODO: レスポンス漏れ

            }
        }
    };

    logger.info("loaded");

})(this);