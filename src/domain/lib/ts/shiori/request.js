// shiori request管理
// リクエストの解釈など
var request = (function () {
    function request(text, res_func) {
        this.raw = text;
        this.response = res_func;
    }
    return request;
})();
exports.request = request;
//# sourceMappingURL=request.js.map
