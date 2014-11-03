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
    public OnMinuteChange(req: IF.shiori_request): boolean;
    public OnSecondChange(req: IF.shiori_request): boolean;
    public OnSurfaceRestore(req: IF.shiori_request): boolean;
    public OnTrayBalloonTimeout(req: IF.shiori_request): boolean;
    public OnMouseEnterAll(req: IF.shiori_request): boolean;
    public OnMouseEnter(req: IF.shiori_request): boolean;
    public OnMouseMove(req: IF.shiori_request): boolean;
    public OnMouseHover(req: IF.shiori_request): boolean;
}
