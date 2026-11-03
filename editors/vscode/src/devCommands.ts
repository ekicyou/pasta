// DevCommands - 3 つのコマンドの vscode の糊（vscode-mcp-client の設計の DevCommands）
//
// 対象の文書（2.5）・ゴースト（2.4）・送る相手の選び方と覚え（3.2〜3.7）・出力と通知（7.1〜7.3）・
// 進み具合（7.4）・重なりの止め（7.5）を受け持つ。流れの中身は reloadShiori・runSceneAtCursor・dumpVariables。

import * as path from 'path';
import * as vscode from 'vscode';
import { CANDIDATE_PORTS, CallResult, Endpoint, HOST, findBasewares, identify } from './mcpClient';
import { GhostTarget, resolveGhost } from './ghostFiles';
import { findGhostMaster } from './ghostValidator';
import { DEFAULT_TIMING, FlowContext, Timing, formatReload, reloadAndWait } from './reloadShiori';
import { formatScene, runScene } from './runSceneAtCursor';
import { dumpVariables, formatDump } from './dumpVariables';

export interface DevCommandDeps {
  readonly ports: readonly number[];   // 既定 CANDIDATE_PORTS（試験だけが差し替える）
  readonly timing: Timing;             // 既定 DEFAULT_TIMING（同上）
  readonly sleep: (ms: number) => Promise<void>;
}

export const OUTPUT_CHANNEL = 'Pasta 開発';

const BAD_DOCUMENT = 'ゴーストを特定できません（ディスク上の .pasta・.lua・pasta.toml を開いてください）';
const NOT_IN_GHOST = 'ゴーストのフォルダの中のファイルではありません';
const BUSY = 'このゴーストへのコマンドが進んでいます';
const NOT_FOUND = 'このゴーストが動いている SSP・areka が見つかりません';
const NOT_FOUND_CHECKS = '確かめること: ベースウェアでこのゴーストを起動しているか・SSP の MCP の窓口が有効か';

/** 誤りの通知で出す失敗（7.3。それ以外の止まり方は警告） */
const ERROR_KINDS: ReadonlySet<CallResult['kind']> = new Set(['unreachable', 'badShape', 'rpcError', 'ng']);

type Report = { lines: string[]; notice?: string; failure?: CallResult };

interface Command {
  readonly id: string;
  readonly title: string;
  readonly accepts: (doc: vscode.TextDocument) => boolean;
  readonly run: (ctx: FlowContext, editor: vscode.TextEditor) => Promise<Report>;
}

const isPasta = (doc: vscode.TextDocument) => doc.languageId === 'pasta';
const isGhostSource = (doc: vscode.TextDocument) =>
  isPasta(doc) || doc.languageId === 'lua' || path.basename(doc.uri.fsPath).toLowerCase() === 'pasta.toml';

const COMMANDS: readonly Command[] = [
  {
    id: 'pasta.runSceneAtCursor', title: '▶ シーンを実行', accepts: isPasta,
    run: async (ctx, editor) => {
      const o = await runScene(ctx, editor.document.uri.fsPath, editor.selection.active.line + 1);
      const failure = o.kind === 'failed' ? o.result : o.reload.kind === 'refused' ? o.reload.result : undefined;
      return { ...formatScene(o), failure };
    },
  },
  {
    id: 'pasta.reloadShiori', title: 'SHIORI をリロード', accepts: isGhostSource,
    run: async (ctx) => {
      const o = await reloadAndWait(ctx);
      return { ...formatReload(o), failure: o.kind === 'refused' ? o.result : undefined };
    },
  },
  {
    id: 'pasta.showSaveVariables', title: '保存変数を表示', accepts: isGhostSource,
    run: async (ctx) => {
      const o = await dumpVariables(ctx);
      return { ...formatDump(o), failure: o.kind === 'failed' ? o.result : undefined };
    },
  },
];

const pad = (n: number) => String(n).padStart(2, '0');
const clock = (d = new Date()) => `[${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}]`;

export function registerDevCommands(context: vscode.ExtensionContext, deps: Partial<DevCommandDeps> = {}): void {
  const ports = deps.ports ?? CANDIDATE_PORTS;
  const timing = deps.timing ?? DEFAULT_TIMING;
  const sleep = deps.sleep ?? ((ms: number) => new Promise<void>((r) => setTimeout(r, ms)));
  // この窓の間だけ（保存しない。3.5）
  const remembered = new Map<string, Endpoint>();
  const busy = new Set<string>();
  const output = vscode.window.createOutputChannel(OUTPUT_CHANNEL);
  context.subscriptions.push(output);

  const write = (lines: readonly string[]) => {
    for (const l of lines) {
      output.appendLine(l);
    }
    output.show(true); // フォーカスを奪わない（7.1）
  };
  const stop = (lines: readonly string[], notice: string, asError = false) => {
    write(lines);
    void (asError ? vscode.window.showErrorMessage(notice) : vscode.window.showWarningMessage(notice));
  };

  /** 覚えた相手を 1 回で確かめ、外れたら探し直す（3.6）。0 個・閉じたら undefined */
  async function chooseEndpoint(cmd: Command, ghost: GhostTarget): Promise<Endpoint | undefined> {
    const head = `${clock()} ${cmd.title} — ${ghost.displayName}（${ghost.root}）`;
    const progress = { location: vscode.ProgressLocation.Window, title: `Pasta: ${cmd.title}` };
    const found = await vscode.window.withProgress(progress, async () => {
      const known = remembered.get(ghost.key);
      if (known) {
        const now = await identify(known.port, ghost.root);
        if (now?.basewareName === known.basewareName) {
          return [known];
        }
        remembered.delete(ghost.key);
      }
      return findBasewares(ghost.root, ports);
    });
    if (found.length === 0) {
      stop([head, `${NOT_FOUND}（${ghost.displayName}・${ghost.root}）`, NOT_FOUND_CHECKS],
        `${NOT_FOUND}: ${ghost.displayName}（${ghost.root}）。${NOT_FOUND_CHECKS}`);
      return undefined;
    }
    let chosen = found[0];
    if (found.length > 1) {
      const items = found.map((e) => ({ label: `${e.basewareName}（${HOST}:${e.port}）`, endpoint: e }));
      const pick = await vscode.window.showQuickPick(items, { placeHolder: `${ghost.displayName} へ送る相手を選んでください` });
      if (!pick) {
        return undefined; // 閉じたら何も送らない（3.4）
      }
      chosen = pick.endpoint;
    }
    remembered.set(ghost.key, chosen);
    return chosen;
  }

  async function execute(cmd: Command): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    const doc = editor?.document;
    if (!editor || !doc || doc.uri.scheme !== 'file' || doc.isUntitled || !cmd.accepts(doc)) {
      stop([`${clock()} ${cmd.title} — ${BAD_DOCUMENT}`], BAD_DOCUMENT);
      return;
    }
    const ghost = await resolveGhost(doc.uri.fsPath, findGhostMaster(doc.uri)?.fsPath);
    if (!ghost) {
      stop([`${clock()} ${cmd.title} — ${NOT_IN_GHOST}: ${doc.uri.fsPath}`], `${NOT_IN_GHOST}: ${doc.uri.fsPath}`);
      return;
    }
    // 確かめと足し込みの間に await を挟まないので、同じゴーストの 2 本目はここで止まる（7.5）
    if (busy.has(ghost.key)) {
      stop([`${clock()} ${cmd.title} — ${BUSY}（${ghost.displayName}）`], BUSY);
      return;
    }
    busy.add(ghost.key);
    try {
      const endpoint = await chooseEndpoint(cmd, ghost);
      if (!endpoint) {
        return;
      }
      write([`${clock()} ${cmd.title} — ${ghost.displayName}（${ghost.root}）→ ${endpoint.basewareName} ${HOST}:${endpoint.port}`]);
      const progress = { location: vscode.ProgressLocation.Window, title: `Pasta: ${cmd.title}` };
      const report = await vscode.window.withProgress(progress, () => cmd.run({ endpoint, ghost, timing, sleep }, editor));
      if (report.notice) {
        stop(report.lines, report.notice, report.failure !== undefined && ERROR_KINDS.has(report.failure.kind));
      } else {
        write(report.lines); // 成功は通知しない（7.2）
      }
    } finally {
      busy.delete(ghost.key);
    }
  }

  for (const cmd of COMMANDS) {
    context.subscriptions.push(vscode.commands.registerCommand(cmd.id, () => execute(cmd)));
  }
}
