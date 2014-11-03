// shiori_api/event.ts
// ------------------------------------------------------------
// SHIORI EVENT を解釈し、ghostとのやり取りを行う
var logger = new Duktape.Logger();

var api = require('../shiori_api');

var send = require('./send');

var userfilename = "pasta.json";

function enc(obj) {
    return Duktape.enc("jx", obj, null, 2);
}
function dec(str) {
    return Duktape.dec("jx", str);
}

function loaduser() {
    var str = libfs.readuser(userfilename);
    var user = dec(str);
    return user;
}

function saveuser(user) {
    var str = enc(user);
    libfs.writeuser(userfilename, str);
}

// 指定IDのreq処理メソッドがオブジェクトに存在すれば呼び出す。
function call_get_id_method(obj, id, req) {
    id = id.replace(/\./g, "$");
    var rc = null;
    var method = "obj." + id;
    var text = "if(typeof(" + method + ")==\"function\") rc = " + method + "(req);";
    logger.trace("[call_id_method] eval => " + text);
    eval(text);
    if (typeof (rc) == "string" && send.hasResponse())
        send.resValue(rc);
    return send.hasResponse();
}

var events = (function () {
    function events(ghost) {
        this.ghost = ghost;
    }
    // --------------------------------------------------------
    // SHIORI: load
    events.prototype.load = function (dir) {
        this.loaddir = dir;

        // TODO: レジストリの読み込み
        var user = loaduser();
        this.user = user;
        this.ghost.user = user;
        // TODO: [load]実装する
    };

    // --------------------------------------------------------
    // SHIORI: unload
    events.prototype.unload = function () {
        // TODO: レジストリの保存
        saveuser(this.user);
        // TODO: [unload]実装する
    };

    // --------------------------------------------------------
    // SHIORI: notify
    events.prototype.notify = function (req) {
        // TODO: [notify]実装する
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: get
    events.prototype.get = function (req) {
        var id = req.ID;
        if (!id) {
            send.res500message("ID not found");
            return false;
        }

        call_get_id_method(this.ghost, id, req);
        if (!send.hasResponse())
            return true;

        call_get_id_method(this, id, req);
        if (!send.hasResponse())
            return true;

        // TODO: [get]実装する
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: ID: version
    events.prototype.version = function (req) {
        send.resValue(api.version());
        return true;
    };

    // --------------------------------------------------------
    // SHIORI: ID: name
    events.prototype.name = function (req) {
        send.resValue(api.name());
        return true;
    };
    return events;
})();
exports.events = events;
//# sourceMappingURL=events.js.map
