// shiori response

export var Charset = "UTF-8";
export var Sender = "pasta";
export var last_response: string = null;

var CRLF = "\r\n";

// レスポンスが必要な状態ならtrue。
export function hasResponse(): boolean {
    return last_response == null;
}

// レスポンス情報をリセットし、レスポンスが必要な状態とする。
export function reset(): void {
    last_response = null;
}

function res(code: string, values: string): void {
    var res =
        "SHIORI/3.0 " + code + CRLF +
        "Charset: " + Charset + CRLF +
        "Sender: " + Sender + CRLF;
    if (values) res += values + CRLF;
    res += CRLF;

    if (!hasResponse()) throw new Error("responseは既に送信済みです。このレスポンスは破棄されます。\n" + res);
    libshiori.response(res);
    last_response = res;
}

export function res400BadRequest(): void {
    res("400 Bad Request", "");
}

export function res204NoContent(): void {
    res("204 No Content", "");
}

export function res200Ok(values: string[]): void {
    var text = values.join(CRLF);
    res("200 OK", text);
}

export function resValue(value: string): void {
    var text = "Value: " + value;
    res("200 OK", text);
}


export function res500message(mes: string): void {
    var text = "X-PASTA-ERROR: " + mes;
    res("500 Internal Server Error", text);
}

export function res500error(e: Error): void {
    res500message(e.message);
}

