// pasta.ghost ゴースト本体。AIを担当。
var logger = new Duktape.Logger();

var ghost = (function () {
    function ghost() {
        logger.info("ghost::constructor");
    }
    // --------------------------------------------------------
    // 製作者（ascii表記）
    ghost.prototype.craftman = function () {
        return "<<todo.craftman>>";
    };

    // --------------------------------------------------------
    // 製作者（ascii表記）
    ghost.prototype.craftmanw = function () {
        return "<<todo.craftmanw>>";
    };
    return ghost;
})();
exports.ghost = ghost;
//# sourceMappingURL=ghost.js.map
