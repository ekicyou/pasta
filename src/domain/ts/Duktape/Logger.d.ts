// Duktape組み込みAPI：Logger

declare module Duktape {
    /**
     * ロガー 
     */
    class Logger {
        trace(fmt: string, ...args: any[]): void;
        debug(fmt: string, ...args: any[]): void;
        info(fmt: string, ...args: any[]): void;
        warn(fmt: string, ...args: any[]): void;
        error(fmt: string, ...args: any[]): void;
        fatal(fmt: string, ...args: any[]): void;
    }
}