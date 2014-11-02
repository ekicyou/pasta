// pasta.dll組み込み関数[libfs]
declare module libfs {

    // ファイルをバッファとして読み込みます。
    function readfile(fname: string): string;

    // ファイルをテキストとして読み込みます。
    function readtext(fname: string): string;

    // ユーザーファイルをテキストとして読み込みます。
    function readuser(fname: string): string;

    // ユーザーファイルを保存します。
    function writeuser(fname: string, buf: string): string;

}