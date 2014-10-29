// SHIORI API関係

import _events = require("./shiori_api/events");
import _request = require("./shiori_api/request");

export var events = _events.events;
export var request = _request.request;