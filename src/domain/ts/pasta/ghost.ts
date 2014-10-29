// pasta.ghost ゴースト本体。AIを担当。

var logger = new Duktape.Logger();
logger.info("x1");

export class ghost {

    public constructor() {
        logger.info("ghost::constructor");
    }

}
logger.info("x2");
