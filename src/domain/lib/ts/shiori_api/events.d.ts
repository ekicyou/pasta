import IF = require('../interfaces');
export declare class events {
    constructor(ghost: IF.ghost);
    public ghost: IF.ghost;
    public loaddir: string;
    public user: any;
    public load(dir: string): void;
    public unload(): void;
    public notify(req: IF.shiori_request): void;
    public get(req: IF.shiori_request): void;
}
