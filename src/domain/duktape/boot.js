// 起動処理
//  [shiori.load(dir)]
//  [shiori.unload()]
//  [shiori.get(req)]
//  [shiori.notify(req)]
// の各関数をフックする。
shiori = (function () {
    var logger = new Duktape.Logger();
    logger.debug("shiori init start");

    try {
        var mod = require("shiori");
        logger.debug("shiori init end");
        return mod;
    } catch (e) {
        logger.error(e.stack || e);
    }
})();