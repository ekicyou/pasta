// スクリプトローダ
// duktapeのrequire()解決用のロジック
// pasta.dllの起動時に必ず読み込まれる。
'use strict';

(function () {
    // ロギング設定
    Duktape.Logger.prototype.raw = function (buf) {
        shiorilib.debugstring(buf);
    }
    var logger = new Duktape.Logger();
    logger.debug("start");

    // スクリプトローダ
    Duktape.modSearch = function (id, require, exports, module) {
        var name, src;
        var found = false;

        logger.info('[modSearch] loading module:', id);

        /* Ecmascript check. */
        name = id + '.js';
        logger.trace('loading ecmascript:', name);
        src = _raw_fs.readtext(name);
        logger.trace('loaded ecmascript');
        if (typeof src === 'string') {
            logger.trace('loaded Ecmascript:', name);
            found = true;
        }

        /* Must find either a DLL or an Ecmascript file (or both) */
        if (!found) {
            var mes = 'module not found: ' + id;
            logger.error(mes);
            throw new Error(mes);
        }

        /* For pure C modules, 'src' may be undefined which is OK. */
        return src;
    }

    logger.debug("loaded");
})();