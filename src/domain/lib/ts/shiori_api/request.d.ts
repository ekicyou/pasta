import IF = require('../interfaces');
export declare class request implements IF.shiori_request {
    constructor(text: string);
    static res: typeof res;
    private parse(text);
    public raw: string;
    public time: number;
    public match: string[];
    public method: string;
    public kvlist: IF.keyvalue[];
    public map: any;
    public ID: string;
}
