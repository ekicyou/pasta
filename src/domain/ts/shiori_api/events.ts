// shiori_api/event.ts
// ------------------------------------------------------------
// SHIORI EVENT を解釈し、ghostとのやり取りを行う

var logger = new Duktape.Logger();

import pasta = require('../pasta');
import api = require('../shiori_api');
import IF = require('../interfaces');

var userfilename = "pasta.json";

function enc(obj: any): string { return Duktape.enc("jx", obj, null, 2);}
function dec(str: string): any { return Duktape.dec("jx", str); }

function loaduser(): any {
    var str = libfs.readuser(userfilename);
    var user = dec(str);
    return user;
}

function saveuser(user: any): void {
    var str = enc(user);
    libfs.writeuser(userfilename, str);
}

export class events {
    public constructor(ghost: IF.ghost) {
        this.ghost = ghost;
    }

    /// ゴーストのインターフェース
    public ghost: IF.ghost;

    /// ベースディレクトリ
    public loaddir: string;

    /// ユーザデータ
    public user: any;

    // --------------------------------------------------------
    // SHIORI: load
    public load(dir: string): void {
        this.loaddir = dir;

        // TODO: レジストリの読み込み
        var user = loaduser();
        this.user = user
        this.ghost.user = user;

        // TODO: [load]実装する
    }

    // --------------------------------------------------------
    // SHIORI: unload
    public unload(): void {
        // TODO: レジストリの保存
        saveuser(this.user);

        // TODO: [unload]実装する
    }

    // --------------------------------------------------------
    // SHIORI: notify
    public notify(req: IF.shiori_request): void {
        // TODO: [notify]実装する
    }

    // --------------------------------------------------------
    // SHIORI: get
    public get(req: IF.shiori_request): void {
        // TODO: [get]実装する
    }
}