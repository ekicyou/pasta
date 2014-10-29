// pasta.ghost ゴースト本体。AIを担当。

import IF = require('../interfaces');

var logger = new Duktape.Logger();
logger.info("x1");

export class ghost implements IF.ghost {

    public constructor() {
        logger.info("ghost::constructor");
    }

}
logger.info("x2");
