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
    // ユーザー名
    public username(): string { return "<<todo.username>>"; }

    // --------------------------------------------------------
    // メインキャラ：ポータルサイト
    public sakura$portalsites(): string { return "<<todo.sakura$portalsites>>"; }

    // --------------------------------------------------------
    // メインキャラ：おすすめサイト
    public sakura$recommendsites(): string { return "<<todo.sakura$recommendsites>>"; }





    // --------------------------------------------------------
    // メインキャラ：ポータルサイト
    public kero$portalsites(): string { return "<<todo.kero$portalsites>>"; }

    // --------------------------------------------------------
    // メインキャラ：おすすめサイト
    public kero$recommendsites(): string { return "<<todo.kero$recommendsites>>"; }
}