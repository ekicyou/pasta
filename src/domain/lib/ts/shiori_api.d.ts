import _events = require("./shiori_api/events");
import _request = require("./shiori_api/request");
export import send = require("./shiori_api/send");
export declare var events: typeof _events.events;
export declare var request: typeof _request.request;
export declare function version(): string;
export declare function name(): string;
