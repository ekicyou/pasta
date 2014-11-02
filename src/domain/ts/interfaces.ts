// pasta interfase定義

export interface ghost {

    user: any;

}

export interface shiori_request {
    /// 生リクエスト
    raw: string;

    /// GETリクエストの場合、応答を返す。
    response: (res: string) => void;
}