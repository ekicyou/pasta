// SHIORI API関係
var _events = require("./shiori_api/events");
var _request = require("./shiori_api/request");
var _send = require("./shiori_api/send");

exports.events = _events.events;
exports.request = _request.request;
exports.send = _send;

var _name = "pasta";
var _version = "0.07.01";
var _craftman = "dot-station";
var _craftmanwn = "どっとステーション";

function version() {
    return _name + "-" + _version + "/Duktape" + Duktape.version;
}
exports.version = version;

function name() {
    return _name;
}
exports.name = name;
function craftman() {
    return _craftman;
}
exports.craftman = craftman;
function craftmanw() {
    return _craftmanwn;
}
exports.craftmanw = craftmanw;
function useorigin1() {
    return "1";
}
exports.useorigin1 = useorigin1;
//# sourceMappingURL=shiori_api.js.map
