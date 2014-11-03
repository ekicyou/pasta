// shiori_api/event.ts
// ------------------------------------------------------------
// SHIORI EVENT を解釈し、ghostとのやり取りを行う

var logger = new Duktape.Logger();

import pasta = require('../pasta');
import api = require('../shiori_api');
import IF = require('../interfaces');
import send = require('./send');

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

// 指定IDのreq処理メソッドがオブジェクトに存在すれば呼び出す。
function call_get_id_method(obj: any, id: string, req: IF.shiori_request): boolean {
    id = id.replace(/\./g, "$");
    var rc: any = null;
    var method = "obj." + id;
    var text = "if(typeof(" + method + ")==\"function\") rc = " + method + "(req);";
    logger.trace("[call_id_method] eval => " + text);
    eval(text);
    if (typeof (rc) == "string" && send.hasResponse()) send.resValue(rc);
    return send.hasResponse();
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
    public notify(req: IF.shiori_request): boolean {
        // TODO: [notify]実装する
        return false;
    }

    // --------------------------------------------------------
    // SHIORI: get
    public get(req: IF.shiori_request): boolean {
        var id = req.ID;
        if (!id) {
            send.res500message("ID not found");
            return false;
        }

        call_get_id_method(this.ghost, id, req);
        if (!send.hasResponse()) return true;

        call_get_id_method(this, id, req);
        if (!send.hasResponse()) return true;

        call_get_id_method(api, id, req);
        if (!send.hasResponse()) return true;


        // TODO: [get]実装する
        return false;
    }

    // --------------------------------------------------------
    // SHIORI: ID: OnMinuteChange
    public OnMinuteChange(req: IF.shiori_request): boolean {

        return false;
    }

    // --------------------------------------------------------
    // SHIORI: ID: OnSecondChange
    public OnSecondChange(req: IF.shiori_request): boolean {

        return false;
    }

    // --------------------------------------------------------
    // SHIORI: ID: OnSurfaceRestore
    public OnSurfaceRestore(req: IF.shiori_request): boolean {

        return false;
    }

    // --------------------------------------------------------
    // SHIORI: ID: OnTrayBalloonTimeout
    public OnTrayBalloonTimeout(req: IF.shiori_request): boolean {

        return false;
    }

    // --------------------------------------------------------
    // SHIORI: ID: OnMouseEnterAll
    public OnMouseEnterAll(req: IF.shiori_request): boolean {

        return false;
    }

    // --------------------------------------------------------
    // SHIORI: ID: OnMouseEnter
    public OnMouseEnter(req: IF.shiori_request): boolean {

        return false;
    }

    // --------------------------------------------------------
    // SHIORI: ID: OnMouseMove
    public OnMouseMove(req: IF.shiori_request): boolean {

        return false;
    }

    // --------------------------------------------------------
    // SHIORI: ID: OnMouseHover
    public OnMouseHover(req: IF.shiori_request): boolean {

        return false;
    }






}