// shioriインターフェース

/// <reference path="shiorilib.d.ts">

import pasta = require("./pasta/ghost");
import api = require("./shiori/request");


export module shiori {
    //---------------------------------------------------------
    // ゴースト
    var ghost = new pasta.ghost();

    //---------------------------------------------------------
    // ロギング
    var logger = new Duktape.Logger();

    //---------------------------------------------------------
    // 公開変数

    /// ロードディレクトリ
    export var loaddir: string;

    //---------------------------------------------------------
    // レスポンス処理関数
    var hasResponse = false;

    // レスポンス
    var response = (res: string) => {
        if (!hasResponse) {
            logger.error("response(): multiple call");
            return;
        }
        hasResponse = false;
        shiorilib.response(res);
    }

    //---------------------------------------------------------
    // SHIORI LOAD
    export function load(dir: string): void {
        try {
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
    }

    //---------------------------------------------------------
    // SHIORI UNLOAD
    export function unload() {
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
    export function notify(raw_request: string) {
        try {
            logger.debug("notify: start");
            logger.debug(raw_request);
            // TODO: NOTIFY処理
            var req = new api.request(raw_request, response);


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
    export function get(raw_request: string) {
        hasResponse = true;
        try {
            logger.debug("get: start");
            logger.debug(raw_request);

            // TODO: GET処理
            var req = new api.request(raw_request, response);


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

}