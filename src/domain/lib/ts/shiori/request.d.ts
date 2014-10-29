export declare class request {
    constructor(text: string, res_func: (res: string) => void);
    public raw: string;
    public response: (res: string) => void;
}
