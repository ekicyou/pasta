// Duktape組み込みAPI：Logger

declare module Duktape {
    /**
     * ロガー
     */
    class Logger {
        trace(...args: any[]): void;
        debug(...args: any[]): void;
        info(...args: any[]): void;
        warn(...args: any[]): void;
        error(...args: any[]): void;
        fatal(...args: any[]): void;
    }

    function enc(fmt: string, obj: any, replacer: any, space: number): string;
    function enc(fmt: string, obj: any, replacer: any): string;
    function enc(fmt: string, obj: any): string;

    function dec(fmt: string, text: string): any;
}