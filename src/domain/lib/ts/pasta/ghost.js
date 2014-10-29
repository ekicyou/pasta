// pasta.ghost ゴースト本体。AIを担当。
var logger = new Duktape.Logger();
logger.info("x1");

var ghost = (function () {
    function ghost() {
        logger.info("ghost::constructor");
    }
    return ghost;
})();
exports.ghost = ghost;
logger.info("x2");
//# sourceMappingURL=ghost.js.map
