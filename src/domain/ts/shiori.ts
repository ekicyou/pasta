// shioriインターフェース

import pasta = require("./pasta");
import api = require("./shiori_api");

var logger = new Duktape.Logger();
logger.info("\n<<import:pasta>>\n", Duktape.enc("jx", pasta, null, 4));
logger.info("\n<<import:api>>\n", Duktape.enc("jx", api, null, 4));
logger.info("import fin...");


//---------------------------------------------------------
// ゴースト
var ghost = new pasta.ghost();
logger.info("x:1");

//---------------------------------------------------------
// 公開変数

/// ロードディレクトリ
export var loaddir: string;
logger.info("x:2");

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
    libshiori.response(res);
}
logger.info("x:3");

//---------------------------------------------------------
// SHIORI LOAD
export function load(dir: string): void {
    try {
        logger.debug("load: start");
        logger.debug("loaddir=" + dir);
        loaddir = dir;
    }
    catch (e) {
        logger.error(e);
    }
    finally {
        logger.debug("load: fin");
    }
}
logger.info("x:4");

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
logger.info("x:5");

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
logger.info("x:6");

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
