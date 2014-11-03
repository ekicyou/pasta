// pasta.ghost ゴースト本体。AIを担当。

import IF = require('../interfaces');

var logger = new Duktape.Logger();

export class ghost implements IF.ghost {
    public constructor() {
        logger.info("ghost::constructor");
    }

    // 保管されるユーザ情報
    public user: any;

    // --------------------------------------------------------
    // 製作者（ascii表記）
    public craftman(): string { return "<<todo.craftman>>"; }

    // --------------------------------------------------------
    // 製作者（ascii表記）
    public craftmanw(): string { return "<<todo.craftmanw>>"; }
}