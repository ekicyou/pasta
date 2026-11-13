// 共有の流れが外へ出る口の全部（拡張は Node、アプリは Tauri のコマンドで実装する）。
// 失敗で投げない: 読めない・無い・門で断られたは undefined。

export type PostOutcome =
  | { readonly kind: 'response'; readonly status: number; readonly body: string } // 本文は UTF-8 で読んだもの
  | { readonly kind: 'unreachable'; readonly detail: string } // 頼みを書く前に切れた（接続できない）
  | { readonly kind: 'noResponse'; readonly detail: string }; // 書いた後に切れた・時間切れ

export interface FileStat {
  readonly size: number;
  readonly mtimeMs: number;
  readonly isFile: boolean;
}

export interface GhostIO {
  /** initialize の clientInfo.name（拡張 'pasta-vscode'・アプリ 'pasta-recipe'） */
  readonly clientName: string;
  /** 127.0.0.1:{port}/api/mcp/v1 へ 1 回だけ POST する。宛先・パス・頭は実装が固定し、Origin は付けない */
  post(port: number, body: string, timeoutMs: number): Promise<PostOutcome>;
  /** フォルダの中の名前。無い・読めない・断られたら undefined */
  list(dir: string): Promise<readonly string[] | undefined>;
  /** 無い・読めない・断られたら undefined */
  stat(path: string): Promise<FileStat | undefined>;
  /** start から終わりまでのバイト列。ファイルでない・無い・読めない・断られたら undefined */
  read(path: string, start: number): Promise<Uint8Array | undefined>;
  /** 区切りを足して結ぶ（拡張は path.join、アプリは \） */
  join(base: string, ...parts: string[]): string;
}
