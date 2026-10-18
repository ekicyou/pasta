// GhostValidator - ゴースト全体の検査の診断（Requirement 11）
//
// ghost/master の一覧と中身を workspace.fs で集め、選び出しと検査は WASM の核に任せる
// （TypeScript はパターンを見ない。11.8）。集め方は pasta_check の validate の殻
// （crates/pasta_check/src/validate_cmd.rs）と同じ: 最上位の profile に入らず、
// シンボリックリンクを含めず、中身はディスクのものを厳格な UTF-8 で読む（11.2・3.4）。

import * as vscode from 'vscode';
import type { FileText, GhostInput, GhostReport, TomlSource, WasmBridge } from './wasmBridge';

/** 核の文言（pasta_check::validate::msg::file_not_utf8）と同じ */
const NOT_UTF8_REASON = 'UTF-8 として読めません';

/** 入力のたびの構文の診断（DocumentSync）と重なる code（11.9） */
const SYNTAX_CODE = 'syntax';

/**
 * fileUri の祖先の ".../ghost/master"（いちばん近いもの）。無ければ undefined。
 * 大文字・小文字は区別しない（Windows の規則。3.1・3.5）。返す URI は届いた綴りのまま
 */
export function findGhostMaster(fileUri: vscode.Uri): vscode.Uri | undefined {
  const segments = fileUri.path.split('/');
  // 最後の要素はファイルそのものなので、親から上へ探す
  for (let i = segments.length - 2; i >= 1; i--) {
    if (segments[i].toLowerCase() === 'master' && segments[i - 1].toLowerCase() === 'ghost') {
      return fileUri.with({ path: segments.slice(0, i + 1).join('/') });
    }
  }
  return undefined;
}

/**
 * ゴーストの鍵（パスを小文字にした URI の文字列）。綴りの違う同じゴーストを 1 つにまとめる（3.6）。
 * 診断の URI と開いている文書の照らし合わせにも使う
 */
export function ghostKey(master: vscode.Uri): string {
  return master.with({ path: master.path.toLowerCase() }).toString();
}

/** 名前が suffix で終わるか（大文字・小文字を区別しない） */
function endsWithIgnoreCase(uri: vscode.Uri, suffix: string): boolean {
  return uri.path.toLowerCase().endsWith(suffix);
}

interface OpenChange {
  uri: vscode.Uri;
  open: boolean;
}

interface GhostState {
  master: vscode.Uri;
  report: GhostReport;
  /** 読めた中身（キーは相対パス）。列を UTF-16 に直すのに使う */
  texts: Map<string, string>;
  /** いま集合に出している URI（キーは toString） */
  shown: Map<string, vscode.Uri>;
}

/** ゴーストごとの束ねの状態 */
interface Schedule {
  timer?: ReturnType<typeof setTimeout>;
  running: boolean;
  /** 走行中に起動の条件が来た */
  again: boolean;
  /** 最後に起動した URI の綴り */
  master: vscode.Uri;
}

export class GhostValidator implements vscode.Disposable {
  private readonly collection = vscode.languages.createDiagnosticCollection('pasta-ghost');
  private readonly ghosts = new Map<string, GhostState>();
  private readonly schedules = new Map<string, Schedule>();
  private readonly subscriptions: vscode.Disposable[] = [];
  private disposed = false;

  constructor(
    private readonly bridge: Pick<WasmBridge, 'selectGhostFiles' | 'validateGhost'>,
    private readonly output: vscode.OutputChannel,
    private readonly delayMs = 300
  ) {}

  /**
   * 保存（.pasta・master の直下の pasta.toml）と配下の .pasta・.lua の作成・削除で検査を起動する
   * （11.3・11.6。名前の変更は削除と作成として届く）。ghost/master の外なら何もしない（11.10）。
   * 大文字・小文字はどれも区別しない（3.1〜3.4）。見張りはすべてのファイルに張って関数で絞る:
   * 1.106 までの VSCode はパターンを大文字・小文字を区別して照らすため（OQ-6）。
   * 開く・閉じるは syntax の出し分けをやり直すだけ（11.9）。起動直後の全体検査はしない
   */
  register(): void {
    const watcher = vscode.workspace.createFileSystemWatcher('**/*');
    const onFileChange = (uri: vscode.Uri) => {
      if (!endsWithIgnoreCase(uri, '.pasta') && !endsWithIgnoreCase(uri, '.lua')) {
        return;
      }
      const master = findGhostMaster(uri);
      if (master) {
        this.requestValidation(master);
      }
    };
    // フォルダを丸ごと消すと、中の削除は届かないことがある。前回読んだファイルか診断を
    // 出していた URI がその下にあれば検査をやり直す（残った診断を消し、辞書の変化を拾う）
    const onDelete = (uri: vscode.Uri) => {
      onFileChange(uri);
      const master = findGhostMaster(uri);
      const state = master && this.ghosts.get(ghostKey(master));
      if (!master || !state) {
        return;
      }
      const prefix = `${ghostKey(uri)}/`;
      const under = (u: vscode.Uri) => ghostKey(u).startsWith(prefix);
      if (
        [...state.shown.values()].some(under) ||
        [...state.texts.keys()].some((rel) => under(vscode.Uri.joinPath(state.master, ...rel.split('/'))))
      ) {
        this.requestValidation(master);
      }
    };
    this.subscriptions.push(
      watcher,
      watcher.onDidCreate(onFileChange),
      watcher.onDidDelete(onDelete),
      vscode.workspace.onDidSaveTextDocument((doc) => {
        const master = findGhostMaster(doc.uri);
        if (!master) {
          return;
        }
        if (
          endsWithIgnoreCase(doc.uri, '.pasta') ||
          doc.uri.path.toLowerCase() === `${master.path.toLowerCase()}/pasta.toml`
        ) {
          this.requestValidation(master);
        }
      }),
      vscode.workspace.onDidOpenTextDocument((doc) => this.renderAll({ uri: doc.uri, open: true })),
      vscode.workspace.onDidCloseTextDocument((doc) => this.renderAll({ uri: doc.uri, open: false }))
    );
  }

  /**
   * ゴーストごと（鍵は ghostKey）に delayMs で束ねて検査する。走行中に来たら終わった後に
   * 1 回だけ走らせる。検査に使う master は最後に起動した URI の綴り
   */
  requestValidation(master: vscode.Uri): void {
    if (this.disposed) {
      return;
    }
    const key = ghostKey(master);
    let s = this.schedules.get(key);
    if (!s) {
      s = { running: false, again: false, master };
      this.schedules.set(key, s);
    }
    s.master = master;
    if (s.running) {
      s.again = true;
      return;
    }
    clearTimeout(s.timer);
    const schedule = s;
    s.timer = setTimeout(() => void this.run(schedule), this.delayMs);
  }

  private async run(s: Schedule): Promise<void> {
    s.timer = undefined;
    s.running = true;
    try {
      await this.validate(s.master);
    } finally {
      s.running = false;
    }
    if (s.again && !this.disposed) {
      s.again = false;
      await this.run(s);
    }
  }

  /** master のゴーストを集めて検査し、そのゴーストの診断を置き換える */
  async validate(master: vscode.Uri): Promise<void> {
    const key = ghostKey(master);
    try {
      const entries: string[] = [];
      const dirs: string[] = [];
      await walk(master, '', entries, dirs);
      const input: GhostInput = { pastaToml: await readToml(master), entries, dirs };
      const listed = new Set(entries);
      const files: Record<string, FileText> = {};
      for (const rel of this.bridge.selectGhostFiles(input)) {
        files[rel] = listed.has(rel)
          ? await readText(vscode.Uri.joinPath(master, ...rel.split('/')))
          : { kind: 'unreadable', reason: '一覧にありません' };
      }
      const report = this.bridge.validateGhost(input, files);
      if (this.disposed) {
        return;
      }
      const texts = new Map<string, string>();
      for (const [rel, f] of Object.entries(files)) {
        if (f.kind === 'text') {
          texts.set(rel, f.text);
        }
      }
      if (input.pastaToml.kind === 'text') {
        texts.set('pasta.toml', input.pastaToml.text);
      }
      const state = this.ghosts.get(key) ?? { master, report, texts, shown: new Map<string, vscode.Uri>() };
      // 診断の URI はその回の綴りで作る（前の綴りの URI は render が消す）
      state.master = master;
      state.report = report;
      state.texts = texts;
      this.ghosts.set(key, state);
      this.render(state);
    } catch (error) {
      if (this.disposed) {
        return;
      }
      const state = this.ghosts.get(key);
      if (state) {
        for (const uri of state.shown.values()) {
          this.collection.delete(uri);
        }
        this.ghosts.delete(key);
      }
      this.output.appendLine(`ゴースト全体の検査に失敗しました（${master.fsPath}）: ${error}`);
    }
  }

  dispose(): void {
    this.disposed = true;
    for (const s of this.schedules.values()) {
      clearTimeout(s.timer);
    }
    this.schedules.clear();
    for (const s of this.subscriptions) {
      s.dispose();
    }
    this.subscriptions.length = 0;
    this.ghosts.clear();
    this.collection.dispose();
  }

  private renderAll(change: OpenChange): void {
    for (const state of this.ghosts.values()) {
      this.render(state, change);
    }
  }

  /**
   * 前回の報告から診断を出し直す。change は開いた・閉じたばかりの文書で、
   * textDocuments の更新と通知の順に頼らずその状態として扱う。
   * 作った URI と大文字・小文字だけ違う文書が開いていれば、その文書の URI に付ける（3.6）
   */
  private render(state: GhostState, change?: OpenChange): void {
    // キーは ghostKey、値は開いている文書の URI
    const open = new Map(vscode.workspace.textDocuments.map((d) => [ghostKey(d.uri), d.uri] as const));
    if (change) {
      if (change.open) {
        open.set(ghostKey(change.uri), change.uri);
      } else {
        open.delete(ghostKey(change.uri));
      }
    }
    const byUri = new Map<string, { uri: vscode.Uri; diagnostics: vscode.Diagnostic[] }>();
    for (const f of state.report.findings) {
      const built = vscode.Uri.joinPath(state.master, ...f.path.split('/'));
      const opened = open.get(ghostKey(built));
      if (f.code === SYNTAX_CODE && opened) {
        continue;
      }
      const uri = opened ?? built;
      const key = uri.toString();
      const line = Math.max((f.line ?? 1) - 1, 0);
      const pos = new vscode.Position(line, utf16Column(state.texts.get(f.path), line, Math.max((f.column ?? 1) - 1, 0)));
      const d = new vscode.Diagnostic(
        new vscode.Range(pos, pos),
        f.message,
        f.severity === 'error' ? vscode.DiagnosticSeverity.Error : vscode.DiagnosticSeverity.Warning
      );
      d.source = 'pasta validate';
      d.code = f.code;
      const entry = byUri.get(key) ?? { uri, diagnostics: [] };
      entry.diagnostics.push(d);
      byUri.set(key, entry);
    }
    for (const [key, uri] of state.shown) {
      if (!byUri.has(key)) {
        this.collection.delete(uri);
      }
    }
    state.shown = new Map();
    for (const [key, { uri, diagnostics }] of byUri) {
      this.collection.set(uri, diagnostics);
      state.shown.set(key, uri);
    }
  }
}

/**
 * 核の列（0 始まりのコードポイント）を VSCode の列（UTF-16）に直す。
 * その行の中身が無ければコードポイントのまま返す
 */
function utf16Column(text: string | undefined, line: number, column: number): number {
  const lineText = text?.split(/\r?\n/)[line];
  if (lineText === undefined) {
    return column;
  }
  let utf16 = 0;
  let n = 0;
  for (const ch of lineText) {
    if (n++ >= column) {
      break;
    }
    utf16 += ch.length;
  }
  return utf16;
}

/**
 * 再帰で '/' 区切りの相対パスを集める。ファイルは out に、入ったフォルダ（末尾の '/' なし）は dirs に。
 * 最上位の profile に入らず（名前は区別して比べる）、リンクは含めない（CLI の walk と同じ）
 */
async function walk(dir: vscode.Uri, prefix: string, out: string[], dirs: string[]): Promise<void> {
  for (const [name, type] of await vscode.workspace.fs.readDirectory(dir)) {
    if ((type & vscode.FileType.SymbolicLink) !== 0) {
      continue;
    }
    if (type === vscode.FileType.Directory) {
      if (prefix === '' && name === 'profile') {
        continue;
      }
      await walk(vscode.Uri.joinPath(dir, name), `${prefix}${name}/`, out, dirs);
      dirs.push(`${prefix}${name}`);
    } else if (type === vscode.FileType.File) {
      out.push(`${prefix}${name}`);
    }
  }
}

function decodeStrict(bytes: Uint8Array): string | undefined {
  try {
    return new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(bytes);
  } catch {
    return undefined;
  }
}

async function readText(uri: vscode.Uri): Promise<FileText> {
  let bytes: Uint8Array;
  try {
    bytes = await vscode.workspace.fs.readFile(uri);
  } catch (error) {
    return { kind: 'unreadable', reason: reasonOf(error) };
  }
  const text = decodeStrict(bytes);
  return text === undefined ? { kind: 'notUtf8' } : { kind: 'text', text };
}

async function readToml(master: vscode.Uri): Promise<TomlSource> {
  let bytes: Uint8Array;
  try {
    bytes = await vscode.workspace.fs.readFile(vscode.Uri.joinPath(master, 'pasta.toml'));
  } catch (error) {
    if ((error as { code?: unknown }).code === 'FileNotFound') {
      return { kind: 'missing' };
    }
    return { kind: 'unreadable', reason: reasonOf(error) };
  }
  const text = decodeStrict(bytes);
  return text === undefined ? { kind: 'unreadable', reason: NOT_UTF8_REASON } : { kind: 'text', text };
}

function reasonOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
