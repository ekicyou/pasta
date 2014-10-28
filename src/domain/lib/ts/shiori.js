// shioriインターフェース
var shiori;
(function (shiori) {
    //---------------------------------------------------------
    // ロギング
    var logger = new Duktape.Logger();

    //---------------------------------------------------------
    // 公開変数
    // ロードディレクトリ
    shiori.loaddir;

    //---------------------------------------------------------
    // レスポンス処理関数
    var hasResponse = false;

    // レスポンス
    function response(res) {
        if (!hasResponse) {
            logger.error("response(): multiple call");
            return;
        }
        hasResponse = false;
        shiori.response(res);
    }

    //---------------------------------------------------------
    // SHIORI LOAD
    function load(dir) {
        try  {
            logger.debug("load: start");
            logger.debug("loaddir=" + dir);
            shiori.loaddir = dir;
        } catch (e) {
            logger.error(e);
        } finally {
            logger.debug("load: fin");
        }
    }
    shiori.load = load;

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
    shiori.unload = unload;
    ;

    //---------------------------------------------------------
    // SHIORI NOTIFY
    function notify(req) {
        try  {
            logger.debug("notify: start");
            logger.debug(req);
            // TODO: NOTIFY処理
        } catch (e) {
            logger.error(e);
        } finally {
            logger.debug("notify: fin");
        }
    }
    shiori.notify = notify;
    ;

    //---------------------------------------------------------
    // SHIORI GET
    function get(req) {
        hasResponse = true;
        try  {
            logger.debug("get: start");
            logger.debug(req);

            // TODO: GET処理
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
    shiori.get = get;
    ;

    logger.info("loaded");
})(shiori || (shiori = {}));
//# sourceMappingURL=shiori.js.map
