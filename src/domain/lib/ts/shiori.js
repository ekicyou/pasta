// shioriインターフェース
var pasta = require("./pasta");
var api = require("./shiori_api");

var logger = new Duktape.Logger();

//---------------------------------------------------------
// ゴースト/shiori event ハンドラ
var ghost = new pasta.ghost();
var event = new api.events(ghost);

//---------------------------------------------------------
// 公開変数
/// ロードディレクトリ
exports.loaddir;

//---------------------------------------------------------
// レスポンス処理関数
var hasResponse = false;

// レスポンス
var response = function (res) {
    if (!hasResponse) {
        logger.error("response(): multiple call");
        return;
    }
    hasResponse = false;
    libshiori.response(res);
};

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
        var req = new api.request(raw_request, response);
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
    hasResponse = true;
    try  {
        logger.debug("get: start");
        logger.debug(raw_request);

        var req = new api.request(raw_request, response);
        event.notify(req);

        // TODO: 正式応答を返すようになったら外す
        response("SHIORI/3.0 200 OK\r\n\r\n");
    } catch (e) {
        logger.error(e.stack || e);
    } finally {
        if (hasResponse) {
            // TODO: レスポンス漏れ
        }
        logger.debug("get: fin");
    }
}
exports.get = get;
;

logger.info("loaded");
//# sourceMappingURL=shiori.js.map
