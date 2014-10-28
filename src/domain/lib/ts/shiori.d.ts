declare module shiori {
    var loaddir: string;
    function load(dir: string): void;
    function unload(): void;
    function notify(req: string): void;
    function get(req: string): void;
}
