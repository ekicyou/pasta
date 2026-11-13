// NodeGhostIO - 共有の流れの入出力を Node で（pasta-recipe-dev-actions の設計の NodeGhostIO）
//
// vscode に触らない。HTTP の頼み（宛先は 127.0.0.1 の固定・送り直さない）・ファイルの読み取り・パスの結びと、
// ファイルから上ってゴーストを見つける検査（resolveGhost）を持つ。

import { promises as fs } from 'fs';
import * as http from 'http';
import * as path from 'path';
import type { FileStat, GhostIO, PostOutcome } from '../../shared/ghostIO.ts';
import { HOST, MCP_PATH } from '../../shared/mcpClient.ts';
import { type GhostTarget, ghostTarget } from '../../shared/ghostFiles.ts';

/** 127.0.0.1:{port}/api/mcp/v1 へ 1 回だけ POST する。投げない */
function post(port: number, body: string, timeoutMs: number): Promise<PostOutcome> {
  return new Promise((resolve) => {
    let done = false;
    const finish = (r: PostOutcome) => { if (!done) { done = true; resolve(r); } };
    let written = false;

    const req = http.request({
      host: HOST, port, path: MCP_PATH, method: 'POST',
      agent: false, // no connection reuse
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json, text/event-stream', // areka answers 406 without both
        'Content-Length': Buffer.byteLength(body),
      },
    }, (res) => {
      const chunks: Buffer[] = [];
      res.on('data', (c: Buffer) => chunks.push(c));
      res.on('error', (e) => finish({ kind: 'noResponse', detail: e.message }));
      res.on('end', () => finish({ kind: 'response', status: res.statusCode ?? 0, body: Buffer.concat(chunks).toString('utf8') }));
    });
    req.on('finish', () => { written = true; });
    req.setTimeout(timeoutMs, () => {
      finish({ kind: 'noResponse', detail: `no response within ${timeoutMs} ms` });
      req.destroy();
    });
    req.on('error', (e) => finish(written
      ? { kind: 'noResponse', detail: e.message }
      : { kind: 'unreachable', detail: e.message }));
    req.end(body);
  });
}

export const nodeGhostIO: GhostIO = {
  clientName: 'pasta-vscode',
  post,
  async list(dir) {
    try {
      return await fs.readdir(dir);
    } catch {
      return undefined;
    }
  },
  async stat(p): Promise<FileStat | undefined> {
    try {
      const st = await fs.stat(p);
      return { size: st.size, mtimeMs: st.mtimeMs, isFile: st.isFile() };
    } catch {
      return undefined;
    }
  },
  async read(p, start) {
    let handle;
    try {
      handle = await fs.open(p, 'r');
      const st = await handle.stat();
      if (!st.isFile()) {
        return undefined; // Windows ではフォルダも開けてしまう
      }
      const buf = Buffer.alloc(Math.max(0, st.size - start));
      const { bytesRead } = await handle.read(buf, 0, buf.length, start);
      return buf.subarray(0, bytesRead);
    } catch {
      return undefined;
    } finally {
      await handle?.close();
    }
  },
  join: (base, ...parts) => path.join(base, ...parts),
};

export const MAX_ASCENT = 3;

/**
 * masterPath は findGhostMaster(uri)?.fsPath。ファイルのフォルダから上へ MAX_ASCENT までの間にあり、
 * descript.txt（大文字・小文字を区別しない）を持つときだけ、その 2 つ上をルートにする（2.1）。外れたら undefined
 */
export async function resolveGhost(io: GhostIO, fileFsPath: string, masterPath: string | undefined): Promise<GhostTarget | undefined> {
  if (masterPath === undefined) {
    return undefined;
  }
  const rel = path.relative(masterPath, path.dirname(fileFsPath));
  const ups = rel === '' ? [] : rel.split(/[\\/]/);
  if (ups.length > MAX_ASCENT || ups[0] === '..' || path.isAbsolute(rel)) {
    return undefined;
  }
  const root = path.dirname(path.dirname(masterPath));
  return ghostTarget(io, root, masterPath, path.basename(root));
}
