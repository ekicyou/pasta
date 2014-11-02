// shiori_api/event.ts
// ------------------------------------------------------------
// SHIORI EVENT を解釈し、ghostとのやり取りを行う
var logger = new Duktape.Logger();

var events = (function () {
    function events(ghost) {
        this.ghost = ghost;
    }
    // --------------------------------------------------------
    // SHIORI: load
    events.prototype.load = function (dir) {
        this.loaddir = dir;
        // TODO: レジストリの読み込み
        // TODO: [load]実装する
    };

    // --------------------------------------------------------
    // SHIORI: unload
    events.prototype.unload = function () {
        // TODO: レジストリの保存
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
