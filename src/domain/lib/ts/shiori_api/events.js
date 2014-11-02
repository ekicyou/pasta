// shiori_api/event.ts
// ------------------------------------------------------------
// SHIORI EVENT を解釈し、ghostとのやり取りを行う
var logger = new Duktape.Logger();

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
    };

    // --------------------------------------------------------
    // SHIORI: get
    events.prototype.get = function (req) {
        // TODO: [get]実装する
    };
    return events;
})();
exports.events = events;
//# sourceMappingURL=events.js.map
