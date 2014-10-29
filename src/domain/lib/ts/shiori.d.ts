export declare module shiori {
    var loaddir: string;
    function load(dir: string): void;
    function unload(): void;
    function notify(raw_request: string): void;
    function get(raw_request: string): void;
}
