// スクリプトローダ
// duktapeのrequire()解決用のロジック
// pasta.dllの起動時に必ず読み込まれる。

(function (global) {
    // スクリプトローダ
    Duktape.modSearch = function (id, require, exports, module) {
        var name;
        var src;
        var found = false;

        print('loading module:', id);

        /* Ecmascript check. */
        jsname = id + '.js';
        src = FileIO.readtext(jsname);
        if (typeof src === 'string') {
            print('loaded Ecmascript:', name);
            found = true;
        }

        /* Must find either a DLL or an Ecmascript file (or both) */
        if (!found) {
            throw new Error('module not found: ' + id);
        }

        /* For pure C modules, 'src' may be undefined which is OK. */
        return src;
    }

    // デバッグログの出力先変更
    Duktape.Logger.prototype.raw = function (buf) {
        FileIO.debugstring(buf);
    }

    var logger = new Duktape.Logger();
    logger.info("loaded");
})(this);