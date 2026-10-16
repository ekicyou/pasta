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

/** fileUri の祖先の ".../ghost/master"（いちばん近いもの）。無ければ undefined */
export function findGhostMaster(fileUri: vscode.Uri): vscode.Uri | undefined {
  const segments = fileUri.path.split('/');
  // 最後の要素はファイルそのものなので、親から上へ探す
  for (let i = segments.length - 2; i >= 1; i--) {
    if (segments[i] === 'master' && segments[i - 1] === 'ghost') {
      return fileUri.with({ path: segments.slice(0, i + 1).join('/') });
    }
  }
  return undefined;
}

interface OpenChange {
  uri: string;
  open: boolean;
}

interface GhostState {
  master: vscode.Uri;
  report: GhostReport;
  /** いま集合に出している URI（キーは toString） */
  shown: Map<string, vscode.Uri>;
}

/** ゴーストごとの束ねの状態 */
interface Schedule {
  timer?: ReturnType<typeof setTimeout>;
  running: boolean;
  /** 走行中に起動の条件が来た */
  again: boolean;
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
   * 開く・閉じるは syntax の出し分けをやり直すだけ（11.9）。起動直後の全体検査はしない
   */
  register(): void {
    const watcher = vscode.workspace.createFileSystemWatcher('**/ghost/master/**/*.{pasta,lua}');
    const onFileChange = (uri: vscode.Uri) => {
      const master = findGhostMaster(uri);
      if (master) {
        this.requestValidation(master);
      }
    };
    this.subscriptions.push(
      watcher,
      watcher.onDidCreate(onFileChange),
      watcher.onDidDelete(onFileChange),
      vscode.workspace.onDidSaveTextDocument((doc) => {
        const master = findGhostMaster(doc.uri);
        if (!master) {
          return;
        }
        if (doc.uri.path.endsWith('.pasta') || doc.uri.path === `${master.path}/pasta.toml`) {
          this.requestValidation(master);
        }
      }),
      vscode.workspace.onDidOpenTextDocument((doc) => this.renderAll({ uri: doc.uri.toString(), open: true })),
      vscode.workspace.onDidCloseTextDocument((doc) => this.renderAll({ uri: doc.uri.toString(), open: false }))
    );
  }

  /** ゴーストごとに delayMs で束ねて検査する。走行中に来たら終わった後に 1 回だけ走らせる */
  requestValidation(master: vscode.Uri): void {
    if (this.disposed) {
      return;
    }
    const key = master.toString();
    let s = this.schedules.get(key);
    if (!s) {
      s = { running: false, again: false };
      this.schedules.set(key, s);
    }
    if (s.running) {
      s.again = true;
      return;
    }
    clearTimeout(s.timer);
    const schedule = s;
    s.timer = setTimeout(() => void this.run(master, schedule), this.delayMs);
  }

  private async run(master: vscode.Uri, s: Schedule): Promise<void> {
    s.timer = undefined;
    s.running = true;
    try {
      await this.validate(master);
    } finally {
      s.running = false;
    }
    if (s.again && !this.disposed) {
      s.again = false;
      await this.run(master, s);
    }
  }

  /** master のゴーストを集めて検査し、そのゴーストの診断を置き換える */
  async validate(master: vscode.Uri): Promise<void> {
    const key = master.toString();
    try {
      const entries: string[] = [];
      await walk(master, '', entries);
      const input: GhostInput = { pastaToml: await readToml(master), entries };
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
      const state = this.ghosts.get(key) ?? { master, report, shown: new Map<string, vscode.Uri>() };
      state.report = report;
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
   * textDocuments の更新と通知の順に頼らずその状態として扱う
   */
  private render(state: GhostState, change?: OpenChange): void {
    const open = new Set(vscode.workspace.textDocuments.map((d) => d.uri.toString()));
    if (change) {
      if (change.open) {
        open.add(change.uri);
      } else {
        open.delete(change.uri);
      }
    }
    const byUri = new Map<string, { uri: vscode.Uri; diagnostics: vscode.Diagnostic[] }>();
    for (const f of state.report.findings) {
      const uri = vscode.Uri.joinPath(state.master, ...f.path.split('/'));
      const key = uri.toString();
      if (f.code === SYNTAX_CODE && open.has(key)) {
        continue;
      }
      const pos = new vscode.Position(Math.max((f.line ?? 1) - 1, 0), Math.max((f.column ?? 1) - 1, 0));
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

/** 再帰で '/' 区切りの相対パスを集める。最上位の profile に入らず、リンクは含めない */
async function walk(dir: vscode.Uri, prefix: string, out: string[]): Promise<void> {
  for (const [name, type] of await vscode.workspace.fs.readDirectory(dir)) {
    if ((type & vscode.FileType.SymbolicLink) !== 0) {
      continue;
    }
    if (type === vscode.FileType.Directory) {
      if (prefix === '' && name === 'profile') {
        continue;
      }
      await walk(vscode.Uri.joinPath(dir, name), `${prefix}${name}/`, out);
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
