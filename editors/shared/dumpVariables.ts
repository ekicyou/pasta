// DumpVariables - OnPastaDumpVariables を送り、書き出したばかりの variables.toml だけを読む
// （vscode-mcp-client の設計の DumpVariables。要件 6.2・6.3）。OnPastaSetSaveVariable の書き換えの頼みも持つ（pasta-recipe-dev-actions の 7.1・7.7・7.8）
//
// vscode に触らない純粋なモジュール。読み込み直しも保存もしない（6.4）。時計を持たず、待ちは ctx.sleep の回数で数える。

import { type CallResult, callTool, describeResult, readRaiseEvent } from './mcpClient.ts';
import { DEBUG_HINT, mtimeOf, openLogCursor, pastaLogPath, readNewLines, readText, variablesPath } from './ghostFiles.ts';
import type { FlowContext } from './reloadShiori.ts';
import type { Report } from './devSession.ts';

export type DumpOutcome =
  | { kind: 'shown'; text: string; path: string; mtime: number }
  | { kind: 'stale'; path: string; reasons: readonly string[] } // 無い・書き換わらない（6.3）
  | { kind: 'failed'; result: CallResult };

export async function dumpVariables(ctx: FlowContext): Promise<DumpOutcome> {
  const { io, endpoint: { port }, ghost: { root, master }, timing } = ctx;
  const file = variablesPath(io, master);
  const before = await mtimeOf(io, file);
  const cursor = await openLogCursor(io, pastaLogPath(io, master, await readText(io, io.join(master, 'pasta.toml'))).path);
  const reply = readRaiseEvent(await callTool(io, port, 'raise_event', {
    event: 'OnPastaDumpVariables', references: [], ghost_name: root,
  }, timing.callTimeoutMs));
  if (reply.kind === 'failed') {
    return { kind: 'failed', result: reply.result };
  }

  // 台本・台本なし・応答が無い のどれでも、上限まで更新時刻を見る（6.2・6.3）
  const maxSleeps = Math.ceil(timing.dumpWaitMs / timing.pollIntervalMs);
  for (let slept = 0; ; slept++) {
    const mtime = await mtimeOf(io, file);
    if (mtime !== undefined && (before === undefined || mtime > before)) {
      const text = await readText(io, file);
      if (text !== undefined) {
        return { kind: 'shown', text, path: file, mtime };
      }
    }
    if (slept >= maxSleeps) {
      break;
    }
    await ctx.sleep(timing.pollIntervalMs);
  }
  const lines = (await readNewLines(io, cursor))?.lines ?? [];
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

export type SaveVariableType = 'string' | 'number' | 'boolean';
export type SaveVariableChange = { type: SaveVariableType; value: string } | { type: 'delete' };

export type SetOutcome =
  | { kind: 'set' }                                   // 204（OnPastaSetSaveVariable は成功で台本を返さない）
  | { kind: 'refusedByGhost'; script: string }        // 失敗の表記（【数にできない値：…】 など。7.7）
  | { kind: 'noResponse'; detail: string }            // 送り直さない
  | { kind: 'failed'; result: CallResult };           // 届かない・形が違う（7.8）

/** OnPastaSetSaveVariable を 1 回送る。Reference0=名前・Reference1=値・Reference2=型。削除は [名前, "", "delete"]（7.1・7.3）。ファイルは書かない（7.9） */
export async function setSaveVariable(ctx: FlowContext, name: string, change: SaveVariableChange): Promise<SetOutcome> {
  const { io, endpoint: { port }, ghost: { root }, timing } = ctx;
  const references = change.type === 'delete' ? [name, '', 'delete'] : [name, change.value, change.type];
  const reply = readRaiseEvent(await callTool(io, port, 'raise_event', {
    event: 'OnPastaSetSaveVariable', references, ghost_name: root,
  }, timing.callTimeoutMs));
  switch (reply.kind) {
    case 'noScript': return { kind: 'set' };
    case 'script': return { kind: 'refusedByGhost', script: reply.script };
    case 'noResponse': return { kind: 'noResponse', detail: reply.detail };
    case 'failed': return { kind: 'failed', result: reply.result };
  }
}

export function formatSet(name: string, change: SaveVariableChange, outcome: SetOutcome): Report {
  switch (outcome.kind) {
    case 'set':
      return { lines: [change.type === 'delete' ? `＄＊${name} を消しました` : `＄＊${name} を ${change.value}（${change.type}）にしました`] };
    case 'refusedByGhost':
      return { lines: [`ゴーストが断りました: ${outcome.script}`], notice: '保存変数を書き換えられませんでした' };
    case 'noResponse': {
      const text = `応答が無かったため、書き換えを確かめられませんでした: ${outcome.detail}`;
      return { lines: [text], notice: text };
    }
    case 'failed': {
      const text = `保存変数の書き換えを頼めませんでした: ${describeResult(outcome.result)}`;
      return { lines: [text], notice: text, failure: outcome.result };
    }
  }
}
