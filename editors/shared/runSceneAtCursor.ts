// RunScene - 読み込み直しの後、OnPastaSceneAt を送り、台本を取り出す
// （vscode-mcp-client の設計の RunScene。要件 4.4〜4.10）
//
// vscode に触らない純粋なモジュール。時計を持たず、待ちは ctx.sleep の回数で数える。

import { type CallResult, type LogEntry, callTool, describeResult, lastLogId, parseLogEntries, readRaiseEvent } from './mcpClient.ts';
import { DEBUG_HINT, openLogCursor, parsePastaLogLine, readNewLines } from './ghostFiles.ts';
import { type FlowContext, type ReloadOutcome, blocksPlayback, formatReload, reloadAndWait } from './reloadShiori.ts';

/** [fsPath, String(line1)]。line1 は 1 から数えた行（4.6）。locals（TOML のインライン表）があれば 3 つ目に付ける（8.3・8.5） */
export function sceneReferences(fileFsPath: string, line1: number, locals?: string): readonly string[] {
  return locals === undefined ? [fileFsPath, String(line1)] : [fileFsPath, String(line1), locals];
}

export type SceneOutcome =
  | { kind: 'notPlayed'; reload: ReloadOutcome }                                 // 4.4・4.5
  | { kind: 'played'; script: string; viaLog: boolean; reload: ReloadOutcome }   // 4.7・4.8
  | { kind: 'noScript'; reasons: readonly string[]; reload: ReloadOutcome }      // 4.10（pasta.log の OnPastaSceneAt: の警告）
  | { kind: 'unconfirmed'; detail: string; reload: ReloadOutcome }               // 4.9
  | { kind: 'failed'; result: CallResult; reload: ReloadOutcome };               // 7.3

const PLAYED_LABEL = 'Ghost:OnPastaSceneAt';

/** 応答が無いときの引き直し。表示の語が Ghost:OnPastaSceneAt の記録のうち最後のもの（無ければ undefined） */
export function findPlayedScript(entries: readonly LogEntry[]): string | undefined {
  for (let i = entries.length - 1; i >= 0; i--) {
    if (entries[i].label === PLAYED_LABEL) {
      return entries[i].text;
    }
  }
  return undefined;
}

export async function runScene(ctx: FlowContext, fileFsPath: string, line1: number, locals?: string): Promise<SceneOutcome> {
  const reload = await reloadAndWait(ctx);
  if (reload.kind !== 'done' || blocksPlayback(reload)) {
    return { kind: 'notPlayed', reload };
  }
  const { io, endpoint: { port }, ghost: { root }, timing } = ctx;
  const sinceId = await lastLogId(io, port, 'script', root, timing.callTimeoutMs);
  if (typeof sinceId !== 'number') {
    return { kind: 'failed', result: sinceId, reload };
  }
  const cursor = await openLogCursor(io, reload.pastaLog.path); // この再生で増えた分だけを理由に使う
  const reply = readRaiseEvent(await callTool(io, port, 'raise_event', {
    event: 'OnPastaSceneAt', references: sceneReferences(fileFsPath, line1, locals), ghost_name: root,
  }, timing.callTimeoutMs));

  switch (reply.kind) {
    case 'script':
      return { kind: 'played', script: reply.script, viaLog: false, reload };
    case 'failed':
      return { kind: 'failed', result: reply.result, reload };
    case 'noScript': {
      const lines = (await readNewLines(io, cursor))?.lines ?? [];
      const reasons = lines.filter((l) => parsePastaLogLine(l).level === 'WARN' && l.includes('OnPastaSceneAt:'));
      return { kind: 'noScript', reasons, reload };
    }
  }

  // 応答が無い（4.8）: 送る前の番号より後の台本の記録を上限まで引き直す
  let last = reply.detail;
  const maxSleeps = Math.ceil(timing.scriptLogWaitMs / timing.pollIntervalMs);
  for (let slept = 0; ; slept++) {
    const r = await callTool(io, port, 'get_log', { log_type: 'script', ghost_name: root, since_id: sinceId }, timing.callTimeoutMs);
    if (r.kind === 'ok') {
      const script = findPlayedScript(parseLogEntries(r.text));
      if (script !== undefined) {
        return { kind: 'played', script, viaLog: true, reload };
      }
    } else {
      last = describeResult(r);
    }
    if (slept >= maxSleeps) {
      return { kind: 'unconfirmed', detail: last, reload };
    }
    await ctx.sleep(timing.pollIntervalMs);
  }
}

const NO_SCRIPT_GUIDE = 'ゴーストは台本を返しませんでした（204）。pasta.toml に [debug] enabled = true を書いて、SHIORI をリロードしてください。';
const UNCONFIRMED = '台本を確かめられませんでした';

export function formatScene(outcome: SceneOutcome): { lines: string[]; notice?: string } {
  const reload = formatReload(outcome.reload);
  const lines = [...reload.lines];
  switch (outcome.kind) {
    case 'notPlayed':
      lines.push('シーンを再生しませんでした');
      return { lines, notice: reload.notice ?? 'シーンを再生しませんでした' };
    case 'played':
      lines.push(outcome.viaLog ? '台本（応答が無かったため、再生の記録から）:' : '台本:', outcome.script);
      return { lines };
    case 'noScript':
      lines.push(NO_SCRIPT_GUIDE, ...outcome.reasons.map((r) => `  ${r}`));
      return { lines, notice: DEBUG_HINT };
    case 'unconfirmed': {
      const text = `${UNCONFIRMED}（応答が無く、再生の記録にも見つかりませんでした: ${outcome.detail}）`;
      lines.push(text);
      return { lines, notice: UNCONFIRMED };
    }
    case 'failed': {
      const text = `シーンの再生を頼めませんでした: ${describeResult(outcome.result)}`;
      lines.push(text);
      return { lines, notice: text };
    }
  }
}
