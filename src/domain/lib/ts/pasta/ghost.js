// pasta.ghost ゴースト本体。AIを担当。
var logger = new Duktape.Logger();

var ghost = (function () {
    function ghost() {
        logger.info("ghost::constructor");
    }
    return ghost;
})();
exports.ghost = ghost;
//# sourceMappingURL=ghost.js.map
