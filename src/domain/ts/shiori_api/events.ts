// SHIORI EVENT を解釈し、ghostに渡す 

var logger = new Duktape.Logger();

import pasta = require('../pasta');


export class events {

    public constructor(ghost: pasta.ghost) {
        this.ghost = ghost;
    }

    /// 生リクエスト
    public ghost: pasta.ghost;
}
