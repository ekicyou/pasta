import IF = require('../interfaces');
export declare class request implements IF.shiori_request {
    constructor(text: string, res_func: (res: string) => void);
    public raw: string;
    public match: RegExpExecArray;
    public response: (res: string) => void;
}
