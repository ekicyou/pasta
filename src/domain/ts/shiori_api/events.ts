// SHIORI EVENT を解釈し、ghostとのやり取りを行う 

var logger = new Duktape.Logger();

import pasta = require('../pasta');
import api = require('../shiori_api');
import IF = require('../interfaces');


export class events {

    public constructor(ghost: IF.ghost) {
        this.ghost = ghost;
    }

    /// ゴーストのインターフェース
    public ghost: IF.ghost;

    // --------------------------------------------------------
    // SHIORI: load
    public load(dir: string): void {

    }

    // --------------------------------------------------------
    // SHIORI: unload
    public unload(): void {

    }

    // --------------------------------------------------------
    // SHIORI: notify
    public notify(req: IF.shiori_request): void {
    }

    // --------------------------------------------------------
    // SHIORI: get
    public get(req: IF.shiori_request): void {
    }

}