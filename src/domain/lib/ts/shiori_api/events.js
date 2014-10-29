// SHIORI EVENT を解釈し、ghostとのやり取りを行う
var logger = new Duktape.Logger();

var events = (function () {
    function events(ghost) {
        this.ghost = ghost;
    }
    // --------------------------------------------------------
    // SHIORI: load
    events.prototype.load = function (dir) {
    };

    // --------------------------------------------------------
    // SHIORI: unload
    events.prototype.unload = function () {
    };

    // --------------------------------------------------------
    // SHIORI: notify
    events.prototype.notify = function (req) {
    };

    // --------------------------------------------------------
    // SHIORI: get
    events.prototype.get = function (req) {
    };
    return events;
})();
exports.events = events;
//# sourceMappingURL=events.js.map
