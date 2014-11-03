// pasta.ghost ゴースト本体。AIを担当。
var logger = new Duktape.Logger();

var ghost = (function () {
    function ghost() {
        logger.info("ghost::constructor");
    }
    // --------------------------------------------------------
    // ユーザー名
    ghost.prototype.username = function () {
        return "<<todo.username>>";
    };

    // --------------------------------------------------------
    // メインキャラ：ポータルサイト
    ghost.prototype.sakura$portalsites = function () {
        return "<<todo.sakura$portalsites>>";
    };

    // --------------------------------------------------------
    // メインキャラ：おすすめサイト
    ghost.prototype.sakura$recommendsites = function () {
        return "<<todo.sakura$recommendsites>>";
    };

    // --------------------------------------------------------
    // メインキャラ：ポータルサイト
    ghost.prototype.kero$portalsites = function () {
        return "<<todo.kero$portalsites>>";
    };

    // --------------------------------------------------------
    // メインキャラ：おすすめサイト
    ghost.prototype.kero$recommendsites = function () {
        return "<<todo.kero$recommendsites>>";
    };
    return ghost;
})();
exports.ghost = ghost;
//# sourceMappingURL=ghost.js.map
