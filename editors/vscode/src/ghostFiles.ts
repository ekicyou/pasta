// GhostFiles - ゴーストのディスクの上のこと（vscode-mcp-client の設計の GhostFiles）
//
// vscode に触らない純粋なモジュール。ルート・descript の name（2.1・2.3）と pasta.log の場所（5.6）を決める。
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

export interface PastaLogLocation {
  readonly path: string;
  /** file_path を読めず既定へ倒したときの添え書き（出力に出す）。読めた・書かれていないときは undefined */
  readonly note?: string;
}

export const DEFAULT_PASTA_LOG = 'profile/pasta/logs/pasta.log';
/**
 * pasta.log の場所（5.6）。pasta.toml の [logging] file_path を 3 つの書き方で読み、pasta の logger.rs と同じ条件
 * （最初の部品が profile で後に部品がある・`..` 無し・絶対パスでない）に合えば使う。書かれていなければ既定。書かれているのに読めない・外れるときは既定と note
 */
export function pastaLogPath(master: string, pastaTomlText: string | undefined): PastaLogLocation {
  const fallback = { path: path.join(master, DEFAULT_PASTA_LOG) };
  const found = pastaTomlText === undefined ? undefined : findLoggingFilePath(pastaTomlText);
  if (found === undefined) {
    return fallback;
  }
  const note = (why: string) => ({ ...fallback, note: `pasta.toml の [logging] file_path ${why}ため、既定の ${DEFAULT_PASTA_LOG} を読みます` });
  if (found === null) {
    return note('を読めなかった（1 行の文字列で書いてください）');
  }
  // logger.rs と同じくパスの部品で判じる（本番の Windows では \ も区切り。空と . の部品は消える）
  const rel = found.replace(/\\/g, '/');
  const parts = rel.split('/').filter((p) => p !== '' && p !== '.');
  if (rel.startsWith('/') || parts[0] !== 'profile' || parts.length < 2 || rel.includes('..')) {
    return note(`（${found}）が profile/ の下のファイルでない`);
  }
  return { path: path.join(master, ...parts) };
}

/**
 * file_path の値。読めた値・書かれているのに読めない（null）・書かれていない（undefined）。
 * shortcut: 読むのは 3 つの書き方の 1 行だけ（引用符付きのキーは書かれていない扱い）。要望が出たら TOML の読み手を考える
 */
function findLoggingFilePath(text: string): string | null | undefined {
  let table = '';
  let inMultiLineInline = false;
  let mentioned = false;
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line === '' || line.startsWith('#')) {
      continue;
    }
    if (inMultiLineInline) {
      mentioned ||= /^file_path\s*=|[{,]\s*file_path\s*=/.test(line);
      inMultiLineInline = !line.includes('}');
      continue;
    }
    if (line.startsWith('[')) {
      const header = /^\[([^[\]]*)\]/.exec(line); // [[配列の表]] は合わない
      table = header ? header[1].replace(/\s/g, '') : '[[';
      continue;
    }
    let rest: string | undefined;
    if (table === 'logging') {
      rest = after(line, /^file_path\s*=/);
    } else if (table === '') {
      rest = after(line, /^logging\s*\.\s*file_path\s*=/);
      const inline = rest === undefined ? after(line, /^logging\s*=/) : undefined;
      if (inline?.startsWith('{')) {
        if (!inline.includes('}')) {
          inMultiLineInline = true;
        }
        rest = after(inline, /[{,]\s*file_path\s*=/);
      }
    }
    if (rest !== undefined) {
      return readString(rest);
    }
  }
  return mentioned ? null : undefined;
}

function after(s: string, re: RegExp): string | undefined {
  const m = re.exec(s);
  return m ? s.slice(m.index + m[0].length).trimStart() : undefined;
}

/** 先頭の 1 行の文字列。基本の文字列は \\ と \" だけを戻す。ほか（複数行の文字列・数など）は null */
function readString(s: string): string | null {
  if (s.startsWith('"""') || s.startsWith("'''")) {
    return null;
  }
  if (s.startsWith("'")) {
    const end = s.indexOf("'", 1);
    return end < 0 ? null : s.slice(1, end);
  }
  if (!s.startsWith('"')) {
    return null;
  }
  let out = '';
  for (let i = 1; i < s.length; i++) {
    const c = s[i];
    if (c === '"') {
      return out;
    }
    if (c === '\\' && (s[i + 1] === '\\' || s[i + 1] === '"')) {
      out += s[++i];
    } else {
      out += c;
    }
  }
  return null;
}
