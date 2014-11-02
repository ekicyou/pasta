// shiori_api/event.ts
// ------------------------------------------------------------
// SHIORI EVENT を解釈し、ghostとのやり取りを行う
var logger = new Duktape.Logger();

var yaml = require('yaml');

var userfilename = "pasta.yaml";

function loaduser() {
    var str = libfs.readuser(userfilename);
    logger.info("<<loaduser:読み込みデータ>>\n" + str + "\n<<ここまで>>");
    var user = yaml.parse(str);
    logger.info("<<loaduser:1>>");
    logger.info(user);
    logger.info("<<loaduser:終了>>");
    return user;
}

function saveuser(user) {
    logger.info(user);
    var str = yaml.stringify(user);
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
