// pasta.ghost ゴースト本体。AIを担当。

import IF = require('../interfaces');

var logger = new Duktape.Logger();

export class ghost implements IF.ghost {
    public constructor() {
        logger.info("ghost::constructor");
    }
}
