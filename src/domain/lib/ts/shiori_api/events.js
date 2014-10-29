// SHIORI EVENT を解釈し、ghostに渡す
var logger = new Duktape.Logger();

var events = (function () {
    function events(ghost) {
        this.ghost = ghost;
    }
    return events;
})();
exports.events = events;
//# sourceMappingURL=events.js.map
