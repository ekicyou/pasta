// shioriインターフェース
/// <reference path="shiorilib.d.ts">
var pasta = require("./pasta/ghost");
var api = require("./shiori/request");

//---------------------------------------------------------
// ゴースト
var ghost = new pasta.ghost();

//---------------------------------------------------------
// ロギング
var logger = new Duktape.Logger();

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
    shiorilib.response(res);
};

//---------------------------------------------------------
// SHIORI LOAD
function load(dir) {
    try  {
        logger.debug("load: start");
        logger.debug("loaddir=" + dir);
        exports.loaddir = dir;
    } catch (e) {
        logger.error(e);
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
        // TODO: シャットダウン処理の呼び出し
    } catch (e) {
        logger.error(e);
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

        // TODO: NOTIFY処理
        var req = new api.request(raw_request, response);
    } catch (e) {
        logger.error(e);
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

        // TODO: GET処理
        var req = new api.request(raw_request, response);

        response("SHIORI/3.0 200 OK\r\n\r\n");
    } catch (e) {
        logger.error(e);
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
