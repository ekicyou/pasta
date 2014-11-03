// SHIORI API関係

import _events = require("./shiori_api/events");
import _request = require("./shiori_api/request");
import _send = require("./shiori_api/send");

export var events = _events.events;
export var request = _request.request;
export var send = _send;

var _name = "pasta";
var _version = "0.07.01";
var _craftman = "dot-station";
var _craftmanwn = "どっとステーション";

export function version(): string {
    return _name + "-" + _version + "/Duktape" + Duktape.version;
}

export function name(): string { return _name; }
export function craftman(): string { return _craftman; }
export function craftmanw(): string { return _craftmanwn; }
export function useorigin1(): string { return "1"; }

