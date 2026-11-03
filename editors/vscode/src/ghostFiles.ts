// GhostFiles - ゴーストのディスクの上のこと（vscode-mcp-client の設計の GhostFiles）
//
// vscode に触らない純粋なモジュール。ルート・descript の name を決める（2.1・2.3）。
// 書き込まない（読むだけ）。

import { promises as fs } from 'fs';
import * as path from 'path';

export interface GhostTarget {
  readonly root: string;        // ゴーストのルートの絶対パス（ghost_name に送る）
  readonly master: string;      // root/ghost/master
  readonly displayName: string; // descript の name か、ルートのフォルダ名
  readonly key: string;         // root を小文字にしたもの（覚えと重なりの鍵）
}

export const MAX_ASCENT = 3;

/**
 * masterPath は findGhostMaster(uri)?.fsPath。ファイルのフォルダから上へ MAX_ASCENT までの間にあり、
 * descript.txt（大文字・小文字を区別しない）を持つときだけ、その 2 つ上をルートにする（2.1）。外れたら undefined
 */
export async function resolveGhost(fileFsPath: string, masterPath: string | undefined): Promise<GhostTarget | undefined> {
  if (masterPath === undefined) {
    return undefined;
  }
  const rel = path.relative(masterPath, path.dirname(fileFsPath));
  const ups = rel === '' ? [] : rel.split(/[\\/]/);
  if (ups.length > MAX_ASCENT || ups[0] === '..' || path.isAbsolute(rel)) {
    return undefined;
  }
  let entries: string[];
  try {
    entries = await fs.readdir(masterPath);
  } catch {
    return undefined;
  }
  const descript = entries.find((e) => e.toLowerCase() === 'descript.txt');
  if (descript === undefined) {
    return undefined;
  }
  const root = path.dirname(path.dirname(masterPath));
  let name: string | undefined;
  try {
    name = decodeDescriptName(await fs.readFile(path.join(masterPath, descript)));
  } catch {
    name = undefined;
  }
  return { root, master: masterPath, displayName: name ?? path.basename(root), key: root.toLowerCase() };
}

/**
 * descript.txt の name の値。charset が UTF-8（大文字・小文字を問わない）なら UTF-8、ほかは Shift_JIS で読み、
 * BOM を外す。読めない・name が無い・空なら undefined（2.3）
 */
export function decodeDescriptName(bytes: Uint8Array): string | undefined {
  let b = bytes;
  if (b[0] === 0xef && b[1] === 0xbb && b[2] === 0xbf) {
    b = b.subarray(3);
  }
  // charset の行は ASCII なので、まず 1 バイト 1 文字で読んで探す
  const charset = keyValue(new TextDecoder('latin1').decode(b), 'charset');
  let text: string;
  try {
    const enc = charset?.toLowerCase() === 'utf-8' ? 'utf-8' : 'shift_jis';
    text = new TextDecoder(enc, { fatal: true }).decode(b);
  } catch {
    return undefined; // 読めない（Shift_JIS の無い ICU も含む）
  }
  return keyValue(text, 'name') || undefined;
}

/** `key,value` の行の value（最初のもの）。キーの大文字・小文字は区別しない */
function keyValue(text: string, key: string): string | undefined {
  for (const line of text.split(/\r?\n/)) {
    const comma = line.indexOf(',');
    if (comma >= 0 && line.slice(0, comma).trim().toLowerCase() === key) {
      return line.slice(comma + 1).trim();
    }
  }
  return undefined;
}
