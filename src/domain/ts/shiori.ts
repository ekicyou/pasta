// shioriインターフェース

import pasta = require("./pasta");
import api = require("./shiori_api");

var logger = new Duktape.Logger();

//---------------------------------------------------------
// ゴースト/shiori event ハンドラ
var ghost = new pasta.ghost();
var event = new api.events(ghost);
var send = api.send;

//---------------------------------------------------------
// 公開変数

/// ロードディレクトリ
export var loaddir: string;

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
        logger.error(e.stack || e);
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
        logger.error(e.stack || e);
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
        var req = new api.request(raw_request);
        event.notify(req);
    }
    catch (e) {
        logger.error(e.stack || e);
    }
    finally {
        logger.debug("notify: fin");
    }
};

//---------------------------------------------------------
// SHIORI GET
export function get(raw_request: string) {
    try {
        logger.debug("get: start");
        logger.debug(raw_request);
        send.reset();
        var req = new api.request(raw_request);
        if (!req.method) {
            send.res400BadRequest();
            return;
        }
        event.get(req);
    }
    catch (e) {
        send.res500error(e);
        logger.error(e.stack || e);
    }
    finally {
        if (send.hasResponse()) {
            send.res204NoContent();
        }
        logger.debug("get: fin");
    }
};

logger.info("loaded");