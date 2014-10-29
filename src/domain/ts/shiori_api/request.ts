// shiori request管理
// リクエストの解釈など

import IF = require('../interfaces');

var logger = new Duktape.Logger();

export class request implements IF.shiori_request{
    public constructor(text: string, res_func: (res: string) => void) {
        this.raw = text;
        this.response = res_func;
    }

    /// 生リクエスト
    public raw: string;

    /// GETリクエストの場合、応答を返す。
    public response: (res: string) => void;
}