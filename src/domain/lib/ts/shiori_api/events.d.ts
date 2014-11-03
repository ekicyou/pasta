import IF = require('../interfaces');
export declare class events {
    constructor(ghost: IF.ghost);
    public ghost: IF.ghost;
    public loaddir: string;
    public user: any;
    public load(dir: string): void;
    public unload(): void;
    public notify(req: IF.shiori_request): boolean;
    public get(req: IF.shiori_request): boolean;
    private version(req);
    private name(req);
}
