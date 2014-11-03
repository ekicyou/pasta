// SHIORI API関係
var _events = require("./shiori_api/events");
var _request = require("./shiori_api/request");
var send = require("./shiori_api/send");
exports.send = send;

exports.events = _events.events;
exports.request = _request.request;

var _name = "pasta";
var _version = "0.07.01";

function version() {
    return _name + "-" + _version + "/Duktape" + Duktape.version;
}
exports.version = version;

function name() {
    return _name;
}
exports.name = name;
//# sourceMappingURL=shiori_api.js.map
