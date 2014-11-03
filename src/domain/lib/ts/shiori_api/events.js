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

        call_get_id_method(api, id, req);
        if (!send.hasResponse())
            return true;

        // TODO: [get]実装する
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: ID: OnMinuteChange
    events.prototype.OnMinuteChange = function (req) {
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: ID: OnSecondChange
    events.prototype.OnSecondChange = function (req) {
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: ID: OnSurfaceRestore
    events.prototype.OnSurfaceRestore = function (req) {
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: ID: OnTrayBalloonTimeout
    events.prototype.OnTrayBalloonTimeout = function (req) {
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: ID: OnMouseEnterAll
    events.prototype.OnMouseEnterAll = function (req) {
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: ID: OnMouseEnter
    events.prototype.OnMouseEnter = function (req) {
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: ID: OnMouseMove
    events.prototype.OnMouseMove = function (req) {
        return false;
    };

    // --------------------------------------------------------
    // SHIORI: ID: OnMouseHover
    events.prototype.OnMouseHover = function (req) {
        return false;
    };
    return events;
})();
exports.events = events;
//# sourceMappingURL=events.js.map
