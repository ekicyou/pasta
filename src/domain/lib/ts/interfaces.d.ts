export interface ghost {
    user: any;
}
export declare class keyvalue {
    constructor(key: string, value: string);
    public key: string;
    public value: string;
}
export interface shiori_request {
    raw: string;
    time: number;
    match: string[];
    method: string;
    kvlist: keyvalue[];
    map: any;
    ID: string;
}
