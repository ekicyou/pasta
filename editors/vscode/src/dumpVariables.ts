// DumpVariables - OnPastaDumpVariables を送り、書き出したばかりの variables.toml だけを読む
// （vscode-mcp-client の設計の DumpVariables。要件 6.2・6.3）
//
// vscode に触らない純粋なモジュール。読み込み直しも保存もしない（6.4）。時計を持たず、待ちは ctx.sleep の回数で数える。

import * as path from 'path';
import { CallResult, callTool, describeResult, readRaiseEvent } from './mcpClient';
import { DEBUG_HINT, mtimeOf, openLogCursor, pastaLogPath, readNewLines, readText, variablesPath } from './ghostFiles';
import type { FlowContext } from './reloadShiori';

export type DumpOutcome =
  | { kind: 'shown'; text: string; path: string; mtime: number }
  | { kind: 'stale'; path: string; reasons: readonly string[] } // 無い・書き換わらない（6.3）
  | { kind: 'failed'; result: CallResult };

export async function dumpVariables(ctx: FlowContext): Promise<DumpOutcome> {
  const { endpoint: { port }, ghost: { root, master }, timing } = ctx;
  const file = variablesPath(master);
  const before = await mtimeOf(file);
  const cursor = await openLogCursor(pastaLogPath(master, await readText(path.join(master, 'pasta.toml'))).path);
  const reply = readRaiseEvent(await callTool(port, 'raise_event', {
    event: 'OnPastaDumpVariables', references: [], ghost_name: root,
  }, timing.callTimeoutMs));
  if (reply.kind === 'failed') {
    return { kind: 'failed', result: reply.result };
  }

  // 台本・台本なし・応答が無い のどれでも、上限まで更新時刻を見る（6.2・6.3）
  const maxSleeps = Math.ceil(timing.dumpWaitMs / timing.pollIntervalMs);
  for (let slept = 0; ; slept++) {
    const mtime = await mtimeOf(file);
    if (mtime !== undefined && (before === undefined || mtime > before)) {
      const text = await readText(file);
      if (text !== undefined) {
        return { kind: 'shown', text, path: file, mtime };
      }
    }
    if (slept >= maxSleeps) {
      break;
    }
    await ctx.sleep(timing.pollIntervalMs);
  }
  const lines = (await readNewLines(cursor))?.lines ?? [];
  return { kind: 'stale', path: file, reasons: lines.filter((l) => l.includes('OnPastaDumpVariables:')) };
}

export function formatDump(outcome: DumpOutcome): { lines: string[]; notice?: string } {
  switch (outcome.kind) {
    case 'shown':
      return { lines: [`保存変数（${outcome.path}）:`, outcome.text] };
    case 'stale':
      return {
        lines: [
          `${outcome.path} は新しく書き出されませんでした（古い中身は出しません）。${DEBUG_HINT}`,
          ...outcome.reasons.map((r) => `  ${r}`),
        ],
        notice: DEBUG_HINT,
      };
    case 'failed': {
      const text = `保存変数の書き出しを頼めませんでした: ${describeResult(outcome.result)}`;
      return { lines: [text], notice: text };
    }
  }
}
