// shioriインターフェース
var pasta = require("./pasta");
var api = require("./shiori_api");

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
exports.loaddir;
logger.info("x:2");

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
logger.info("x:3");

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
logger.info("x:4");

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
logger.info("x:5");

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
logger.info("x:6");

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
