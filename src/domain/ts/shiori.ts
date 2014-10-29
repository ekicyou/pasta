// shioriインターフェース

import pasta = require("./pasta");
import api = require("./shiori_api");

var logger = new Duktape.Logger();

//---------------------------------------------------------
// ゴースト/shiori event ハンドラ
var ghost = new pasta.ghost();
var event = new api.events(ghost);

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
    libshiori.response(res);
}

//---------------------------------------------------------
// SHIORI LOAD
export function load(dir: string): void {
    try {
        logger.debug("load: start");
        logger.debug("loaddir=" + dir);
        loaddir = dir;
        event.load(dir);
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
        event.unload();
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
        var req = new api.request(raw_request, response);
        event.notify(req);
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

        var req = new api.request(raw_request, response);
        event.notify(req);

        // TODO: 正式応答を返すようになったら外す
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