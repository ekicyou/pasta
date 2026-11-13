// DevCommands - 3 つのコマンドの vscode の糊（vscode-mcp-client の設計の DevCommands）
//
// 対象の文書（2.5）・ゴースト（2.4）を決め、画面（出力・通知・選ぶ窓・進み具合）と保存を DevHost として DevSession へ渡す。
// 相手の選び方と覚え・重なりの止め・見出し・保存の手順・通知の分けは共有の devSession。流れの中身は reloadShiori・runSceneAtCursor・dumpVariables。

import * as path from 'path';
import * as vscode from 'vscode';
import { CANDIDATE_PORTS } from '../../shared/mcpClient.ts';
import { nodeGhostIO as io, resolveGhost } from './nodeGhostIO';
import { findGhostMaster } from './ghostValidator';
import { DEFAULT_TIMING, type FlowContext, type Timing, formatReload, reloadAndWait } from '../../shared/reloadShiori.ts';
import { formatScene, runScene } from '../../shared/runSceneAtCursor.ts';
import { dumpVariables, formatDump } from '../../shared/dumpVariables.ts';
import { type DevHost, DevSession, type Report, clock } from '../../shared/devSession.ts';

export interface DevCommandDeps {
  readonly ports: readonly number[];   // 既定 CANDIDATE_PORTS（試験だけが差し替える）
  readonly timing: Timing;             // 既定 DEFAULT_TIMING（同上）
  readonly sleep: (ms: number) => Promise<void>;
}

export const OUTPUT_CHANNEL = 'Pasta 開発';

const BAD_DOCUMENT = 'ゴーストを特定できません（ディスク上の .pasta・.lua・pasta.toml を開いてください）';
const NOT_IN_GHOST = 'ゴーストのフォルダの中のファイルではありません';

/** コマンドの対象（文書と、1 から数えた行） */
interface Target {
  readonly document: vscode.TextDocument;
  readonly line1: number;
}

interface Command {
  readonly id: string;
  readonly title: string;
  readonly accepts: (doc: vscode.TextDocument) => boolean;
  readonly saves: boolean; // 読み込み直す 2 つだけが保存する（4.2・5.2・6.4）
  readonly run: (ctx: FlowContext, target: Target) => Promise<Report>;
}

/** 引数から対象を決める。(Uri, number) なら指定の文書と行、それ以外（右クリックの Uri だけを含む）はアクティブなエディタとカーソル */
async function resolveTarget(args: readonly unknown[]): Promise<Target | undefined> {
  if (args[0] instanceof vscode.Uri && typeof args[1] === 'number') {
    const line0 = args[1];
    return vscode.workspace.openTextDocument(args[0]).then((document) => ({ document, line1: line0 + 1 }), () => undefined);
  }
  const editor = vscode.window.activeTextEditor;
  return editor && { document: editor.document, line1: editor.selection.active.line + 1 };
}

const isPasta = (doc: vscode.TextDocument) => doc.languageId === 'pasta';
const isGhostSource = (doc: vscode.TextDocument) =>
  isPasta(doc) || doc.languageId === 'lua' || path.basename(doc.uri.fsPath).toLowerCase() === 'pasta.toml';

// 拡張はローカル変数も書き換えも送らない（3 つのコマンドだけ）
const COMMANDS: readonly Command[] = [
  {
    id: 'pasta.runSceneAtCursor', title: '▶ シーンを実行', accepts: isPasta, saves: true,
    run: async (ctx, target) => {
      const o = await runScene(ctx, target.document.uri.fsPath, target.line1);
      const failure = o.kind === 'failed' ? o.result : o.reload.kind === 'refused' ? o.reload.result : undefined;
      return { ...formatScene(o), failure };
    },
  },
  {
    id: 'pasta.reloadShiori', title: 'SHIORI をリロード', accepts: isGhostSource, saves: true,
    run: async (ctx) => {
      const o = await reloadAndWait(ctx);
      return { ...formatReload(o), failure: o.kind === 'refused' ? o.result : undefined };
    },
  },
  {
    id: 'pasta.showSaveVariables', title: '保存変数を表示', accepts: isGhostSource, saves: false,
    run: async (ctx) => {
      const o = await dumpVariables(ctx);
      return { ...formatDump(o), failure: o.kind === 'failed' ? o.result : undefined };
    },
  },
];

/** ルートの下（大文字・小文字を問わない）の未保存の .pasta・.lua・pasta.toml を全部保存し、保存できなかったパスを返す */
async function saveGhostDocuments(root: string): Promise<string[]> {
  const base = path.resolve(root).toLowerCase() + path.sep;
  const targets = vscode.workspace.textDocuments.filter((d) => {
    const file = d.uri.fsPath;
    const ext = path.extname(file).toLowerCase();
    return d.isDirty && d.uri.scheme === 'file' && path.resolve(file).toLowerCase().startsWith(base)
      && (ext === '.pasta' || ext === '.lua' || path.basename(file).toLowerCase() === 'pasta.toml');
  });
  const failed: string[] = [];
  for (const d of targets) {
    if (!(await d.save().then((ok) => ok, () => false))) {
      failed.push(d.uri.fsPath);
    }
  }
  return failed;
}

export function registerDevCommands(context: vscode.ExtensionContext, deps: Partial<DevCommandDeps> = {}): void {
  const output = vscode.window.createOutputChannel(OUTPUT_CHANNEL);
  context.subscriptions.push(output);

  const host: DevHost = {
    io,
    ports: deps.ports ?? CANDIDATE_PORTS,
    timing: deps.timing ?? DEFAULT_TIMING,
    sleep: deps.sleep ?? ((ms: number) => new Promise<void>((r) => setTimeout(r, ms))),
    write: (lines) => {
      for (const l of lines) {
        output.appendLine(l);
      }
      output.show(true); // フォーカスを奪わない（7.1）
    },
    notify: (level, text) => {
      void (level === 'error' ? vscode.window.showErrorMessage(text) : vscode.window.showWarningMessage(text));
    },
    pick: async (_ghost, items, placeHolder) => (await vscode.window.showQuickPick([...items], { placeHolder }))?.endpoint,
    progress: async (title, work) => vscode.window.withProgress({ location: vscode.ProgressLocation.Window, title: `Pasta: ${title}` }, work),
    save: (ghost) => saveGhostDocuments(ghost.root),
  };
  const session = new DevSession(host);

  async function execute(cmd: Command, args: readonly unknown[]): Promise<void> {
    // この await は重なりの確かめ（session.execute の頭）より前（7.5 の決まりは崩れない）
    const target = await resolveTarget(args);
    const doc = target?.document;
    if (!target || !doc || doc.uri.scheme !== 'file' || doc.isUntitled || !cmd.accepts(doc)) {
      session.stop([`${clock()} ${cmd.title} — ${BAD_DOCUMENT}`], BAD_DOCUMENT);
      return;
    }
    const ghost = await resolveGhost(io, doc.uri.fsPath, findGhostMaster(doc.uri)?.fsPath);
    if (!ghost) {
      session.stop([`${clock()} ${cmd.title} — ${NOT_IN_GHOST}: ${doc.uri.fsPath}`], `${NOT_IN_GHOST}: ${doc.uri.fsPath}`);
      return;
    }
    await session.execute({ title: cmd.title, saves: cmd.saves, run: (ctx) => cmd.run(ctx, target) }, ghost);
  }

  for (const cmd of COMMANDS) {
    context.subscriptions.push(vscode.commands.registerCommand(cmd.id, (...args: unknown[]) => execute(cmd, args)));
  }
}
