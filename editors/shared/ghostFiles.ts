// GhostFiles - ゴーストのディスクの上のこと（vscode-mcp-client の設計の GhostFiles）
//
// 拡張とアプリの共有。descript の name（2.1・2.3）と pasta.log の場所（5.6）を決め、
// pasta.log の増えた分（4.3・5.7）と variables.toml（6.2・6.3）を読む。ディスクには io を通してだけ触る。
// 書き込まない（読むだけ）。

import type { GhostIO } from './ghostIO.ts';

export interface GhostTarget {
  readonly root: string;        // ゴーストのルートの絶対パス（ghost_name に送る）
  readonly master: string;      // root/ghost/master
  readonly displayName: string; // descript の name か、ルートのフォルダ名
  readonly key: string;         // root を小文字にしたもの（覚えと重なりの鍵）
}

/** master に descript.txt（大文字・小文字を問わない）があればゴースト。名前は descript の name か folderName */
export async function ghostTarget(io: GhostIO, root: string, master: string, folderName: string): Promise<GhostTarget | undefined> {
  const descript = (await io.list(master))?.find((e) => e.toLowerCase() === 'descript.txt');
  if (descript === undefined) {
    return undefined;
  }
  const bytes = await io.read(io.join(master, descript), 0);
  const name = bytes === undefined ? undefined : decodeDescriptName(bytes);
  return { root, master, displayName: name ?? folderName, key: root.toLowerCase() };
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
/** デバッグを有効にする案内の通知（RunScene の 4.10・DumpVariables の 6.3 が同じ文を使う） */
export const DEBUG_HINT = 'pasta.toml に [debug] enabled = true を書いてリロードしてください';
/**
 * pasta.log の場所（5.6）。pasta.toml の [logging] file_path を 3 つの書き方で読み、pasta の logger.rs と同じ条件
 * （最初の部品が profile で後に部品がある・`..` 無し・絶対パスでない）に合えば使う。書かれていなければ既定。書かれているのに読めない・外れるときは既定と note
 */
export function pastaLogPath(io: GhostIO, master: string, pastaTomlText: string | undefined): PastaLogLocation {
  const fallback = { path: io.join(master, DEFAULT_PASTA_LOG) };
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
  return { path: io.join(master, ...parts) };
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

export interface LogCursor { readonly path: string; readonly offset: number } // offset は読む前の大きさ（無ければ 0）

/** 読む前の大きさを覚える。無い・読めないときは 0（後からできたファイルを頭から読む） */
export async function openLogCursor(io: GhostIO, p: string): Promise<LogCursor> {
  return { path: p, offset: (await io.stat(p))?.size ?? 0 };
}

/** offset から増えた分の完全な行（最後の改行より後は次の回に残す）。読めなければ undefined（5.7） */
export async function readNewLines(io: GhostIO, cursor: LogCursor): Promise<{ lines: string[]; next: LogCursor } | undefined> {
  const st = await io.stat(cursor.path);
  if (st === undefined || !st.isFile) {
    return undefined; // フォルダも読めない扱い
  }
  const start = st.size < cursor.offset ? 0 : cursor.offset; // 縮んだ（作り直された）ら頭から
  const buf = await io.read(cursor.path, start);
  if (buf === undefined) {
    return undefined;
  }
  const end = buf.lastIndexOf(0x0a) + 1; // 改行の後ろまで。無ければ 0
  const lines = end === 0 ? [] : new TextDecoder('utf-8').decode(buf.subarray(0, end - 1)).split(/\r?\n/);
  return { lines, next: { path: cursor.path, offset: start + end } };
}

export type PastaLogLevel = 'ERROR' | 'WARN' | 'INFO' | 'DEBUG' | 'TRACE';
export interface PastaLogLine { readonly level: PastaLogLevel | undefined; readonly text: string }

/** tracing の fmt の行 `<時刻>  <LEVEL> <target>: <本文>` のレベル。形が違う行（続きの行など）は undefined */
export function parsePastaLogLine(line: string): PastaLogLine {
  const m = /^\S+\s+(ERROR|WARN|INFO|DEBUG|TRACE)\s/.exec(line);
  return { level: m ? (m[1] as PastaLogLevel) : undefined, text: line };
}

/** `SHIORI load entry completed` の行なら loaded の値（true・false だけ）、ほかは undefined */
export function loadCompletion(line: string): { loaded: boolean } | undefined {
  if (!line.includes('SHIORI load entry completed')) {
    return undefined;
  }
  const m = /(?:^|\s)loaded=(true|false)(?=\s|$)/.exec(line);
  return m ? { loaded: m[1] === 'true' } : undefined;
}

export function variablesPath(io: GhostIO, master: string): string {
  return io.join(master, 'profile', 'pasta', 'debug', 'variables.toml');
}

/** 更新時刻（ms）。無い・読めないときは undefined */
export async function mtimeOf(io: GhostIO, p: string): Promise<number | undefined> {
  return (await io.stat(p))?.mtimeMs;
}

/** UTF-8 の中身（先頭の BOM は外す）。無い・読めないときは undefined */
export async function readText(io: GhostIO, p: string): Promise<string | undefined> {
  const bytes = await io.read(p, 0);
  return bytes === undefined ? undefined : new TextDecoder('utf-8').decode(bytes);
}
