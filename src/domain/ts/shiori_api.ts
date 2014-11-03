// SHIORI API関係

import _events = require("./shiori_api/events");
import _request = require("./shiori_api/request");
export import send = require("./shiori_api/send");

export var events = _events.events;
export var request = _request.request;

var _name = "pasta";
var _version = "0.07.01";

export function version(): string {
    return _name + "-" + _version + "/Duktape" + Duktape.version;
}

export function name(): string {
    return _name;
}