// shioriインターフェース
var pasta = require("./pasta");
var api = require("./shiori_api");

var logger = new Duktape.Logger();

//---------------------------------------------------------
// ゴースト/shiori event ハンドラ
var ghost = new pasta.ghost();
var event = new api.events(ghost);
var send = api.send;

//---------------------------------------------------------
// 公開変数
/// ロードディレクトリ
exports.loaddir;

//---------------------------------------------------------
// SHIORI LOAD
function load(dir) {
    try  {
        logger.debug("load: start");
        logger.debug("loaddir=" + dir);
        exports.loaddir = dir;
        event.load(dir);
    } catch (e) {
        logger.error(e.stack || e);
    } finally {
        logger.debug("load: fin");
    }
}
exports.load = load;

//---------------------------------------------------------
// SHIORI UNLOAD
function unload() {
    try  {
        logger.debug("unload: start");
        event.unload();
    } catch (e) {
        logger.error(e.stack || e);
    } finally {
        logger.debug("unload: fin");
    }
}
exports.unload = unload;
;

//---------------------------------------------------------
// SHIORI NOTIFY
function notify(raw_request) {
    try  {
        logger.debug("notify: start");
        logger.debug(raw_request);
        var req = new api.request(raw_request);
        event.notify(req);
    } catch (e) {
        logger.error(e.stack || e);
    } finally {
        logger.debug("notify: fin");
    }
}
exports.notify = notify;
;

//---------------------------------------------------------
// SHIORI GET
function get(raw_request) {
    try  {
        logger.debug("get: start");
        logger.debug(raw_request);
        send.reset();
        var req = new api.request(raw_request);
        if (!req.method) {
            send.res400BadRequest();
            return;
        }
        event.get(req);
    } catch (e) {
        send.res500error(e);
        logger.error(e.stack || e);
    } finally {
        if (send.hasResponse()) {
            send.res204NoContent();
        }
        logger.debug("get: fin");
    }
}
exports.get = get;
;

logger.info("loaded");
//# sourceMappingURL=shiori.js.map
