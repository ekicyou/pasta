// 試験の偽の入出力: メモリの上のファイルと、筋書きどおりに答える POST。
// 区切りはアプリと同じ \ に固定する（/ で渡したパスも \ に直して持つ）。

import type { FileStat, GhostIO, PostOutcome } from './ghostIO.ts';

export interface PostRequest {
  readonly port: number;
  readonly body: string;
  readonly timeoutMs: number;
}

/** 筋書きの 1 歩。関数なら頼みを見て答えを返す（その中でファイルを増やしてもよい） */
export type PostStep = PostOutcome | ((req: PostRequest) => PostOutcome | Promise<PostOutcome>);

export interface FakeGhostIO extends GhostIO {
  /** 送られた頼みを順に */
  readonly posts: PostRequest[];
  /** 置く・置き換える。mtimeMs を省くと時計を 1 進めた時刻 */
  setFile(path: string, content: string | Uint8Array, mtimeMs?: number): void;
  /** 終わりに足す（無ければ作る）。更新時刻も進む */
  appendFile(path: string, content: string): void;
  deleteFile(path: string): void;
  /** 次の POST から順に使う答えを足す。尽きたら post は投げる（筋書きの書き漏れ） */
  queuePost(...steps: PostStep[]): void;
}

const norm = (p: string) => p.replace(/\//g, '\\').replace(/\\+$/, '');
const enc = new TextEncoder();

export function createFakeGhostIO(clientName = 'pasta-test'): FakeGhostIO {
  const files = new Map<string, { bytes: Uint8Array; mtimeMs: number }>();
  const steps: PostStep[] = [];
  const posts: PostRequest[] = [];
  let clock = 1_000_000;
  const tick = (t?: number) => {
    if (t === undefined) return ++clock;
    clock = Math.max(clock, t); // 明示の時刻より後に足したものが必ず新しくなるように
    return t;
  };
  const under = (dir: string) => {
    const prefix = norm(dir) + '\\';
    return [...files.keys()].filter((k) => k.startsWith(prefix)).map((k) => k.slice(prefix.length));
  };

  return {
    clientName,
    posts,
    async post(port, body, timeoutMs) {
      const req = { port, body, timeoutMs };
      posts.push(req);
      const step = steps.shift();
      if (step === undefined) throw new Error(`偽の POST の筋書きが尽きました: ${body}`);
      return typeof step === 'function' ? step(req) : step;
    },
    async list(dir) {
      const names = under(dir).map((rest) => rest.split('\\')[0]);
      return names.length === 0 ? undefined : [...new Set(names)];
    },
    async stat(path): Promise<FileStat | undefined> {
      const f = files.get(norm(path));
      if (f) return { size: f.bytes.length, mtimeMs: f.mtimeMs, isFile: true };
      return under(path).length > 0 ? { size: 0, mtimeMs: 0, isFile: false } : undefined;
    },
    async read(path, start) {
      return files.get(norm(path))?.bytes.slice(start);
    },
    join(base, ...parts) {
      return [base, ...parts].map(norm).join('\\');
    },
    setFile(path, content, mtimeMs) {
      files.set(norm(path), { bytes: typeof content === 'string' ? enc.encode(content) : content, mtimeMs: tick(mtimeMs) });
    },
    appendFile(path, content) {
      const old = files.get(norm(path))?.bytes ?? new Uint8Array();
      const add = enc.encode(content);
      const bytes = new Uint8Array(old.length + add.length);
      bytes.set(old);
      bytes.set(add, old.length);
      files.set(norm(path), { bytes, mtimeMs: tick() });
    },
    deleteFile(path) {
      files.delete(norm(path));
    },
    queuePost(...more) {
      steps.push(...more);
    },
  };
}
