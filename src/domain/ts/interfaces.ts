// pasta interfase定義

export interface ghost {
    user: any;
}

export class keyvalue {
    public constructor(key: string, value: string) {
        this.key = key;
        this.value = value;
    }
    public key: string;
    public value: string;
}

export interface shiori_request {
    /// 生リクエスト
    raw: string;

    /// 時刻
    time: number;

    /// 正規表現分割オブジェクト
    match: string[];

    /// 解決できた場合、SHIORI メソッド(GET/NOTIFY)
    method: string;

    /// <key/value> list
    kvlist: keyvalue[];

    /// <key/value> map
    map: any;

    /// ID:
    ID: string;
}