// shioriインターフェース

module shiori {

    //---------------------------------------------------------
    // ロギング
    var logger = new Duktape.Logger();

    //---------------------------------------------------------
    // 公開変数

    // ロードディレクトリ
    export var loaddir: string;



    //---------------------------------------------------------
    // レスポンス処理関数
    var hasResponse = false;

    // レスポンス
    function response(res: string): void {
        if (!hasResponse) {
            logger.error("response(): multiple call");
            return;
        }
        hasResponse = false;
        shiori.response(res);
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
    export function unload  () {
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
    export function notify (req:string) {
        try {
            logger.debug("notify: start");
            logger.debug(req);
            // TODO: NOTIFY処理


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
    export function get(req: string) {
        hasResponse = true;
        try {
            logger.debug("get: start");
            logger.debug(req);

            // TODO: GET処理
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