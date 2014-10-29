// pastaモジュールインポート。

import _events = require("./pasta/events");
import _ghost = require("./pasta/ghost");
import _user_persistence = require("./pasta/user_persistence");

export var events = _events.events;
export var ghost = _ghost.ghost;
export var user_persistence = _user_persistence.user_persistence;