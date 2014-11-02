import IF = require('../interfaces');
export declare class keyvalue {
    constructor(key: string, value: string);
    public key: string;
    public value: string;
}
export declare class request implements IF.shiori_request {
    constructor(text: string, res_func: (res: string) => void);
    private parse(text);
    public response: (res: string) => void;
    public raw: string;
    public time: number;
    public match: string[];
    public method: string;
    public kvlist: keyvalue[];
    public map: any;
}
