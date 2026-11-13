// ReloadShiori - SHIORI を読み込み直し、pasta.log の印で終わりを待ち、増えた誤りと警告を集める
// （vscode-mcp-client の設計の ReloadShiori。要件 4.3・4.4・4.5・5.3・5.4・5.5・5.7・5.8・5.9）
//
// vscode に触らない純粋なモジュール。再生は頼まない（5.9）。時計を持たず、待ちは ctx.sleep の回数で数える。

import { CALL_TIMEOUT_MS, type CallResult, type Endpoint, callTool, describeResult, lastLogId, parseLogEntries } from '../../shared/mcpClient.ts';
import type { GhostIO } from '../../shared/ghostIO.ts';
import {
  type GhostTarget, type PastaLogLocation, loadCompletion, openLogCursor, parsePastaLogLine, pastaLogPath, readNewLines, readText,
} from '../../shared/ghostFiles.ts';

export interface Timing {
  readonly loadTimeoutMs: number;     // 20000
  readonly pollIntervalMs: number;    // 250
  readonly scriptLogWaitMs: number;   // 5000（RunScene）
  readonly dumpWaitMs: number;        // 5000（DumpVariables）
  readonly callTimeoutMs: number;     // CALL_TIMEOUT_MS
}
export const DEFAULT_TIMING: Timing = {
  loadTimeoutMs: 20000, pollIntervalMs: 250, scriptLogWaitMs: 5000, dumpWaitMs: 5000, callTimeoutMs: CALL_TIMEOUT_MS,
};

export interface FlowContext {
  readonly io: GhostIO;
  readonly endpoint: Endpoint;
  readonly ghost: GhostTarget;
  readonly timing: Timing;
  readonly sleep: (ms: number) => Promise<void>;
}

export interface LoadIssue { readonly source: 'pasta.log' | 'baseware'; readonly text: string }

export type ReloadOutcome =
  | { kind: 'refused'; result: CallResult }  // reload（か前の get_log）が NG・届かない・形が違う
  | {
      kind: 'done';
      finished: boolean;              // 印を見たか（false＝時間切れ）
      loaded: boolean | undefined;    // 印の loaded の値
      errors: readonly LoadIssue[];
      warnings: readonly LoadIssue[];
      pastaLogReadable: boolean;
      pastaLog: PastaLogLocation;     // 場所と、file_path を読めなかったときの添え書き
    };

export async function reloadAndWait(ctx: FlowContext): Promise<ReloadOutcome> {
  const { io, endpoint: { port }, ghost: { root, master }, timing } = ctx;
  const pastaLog = pastaLogPath(io, master, await readText(io, io.join(master, 'pasta.toml')));
  let cursor = await openLogCursor(io, pastaLog.path);
  const sinceId = await lastLogId(io, port, 'error', root, timing.callTimeoutMs);
  if (typeof sinceId !== 'number') {
    return { kind: 'refused', result: sinceId };
  }
  const reload = await callTool(io, port, 'reload', { target: 'shiori', ghost_name: root }, timing.callTimeoutMs);
  if (reload.kind !== 'ok') {
    return { kind: 'refused', result: reload };
  }

  const errors: LoadIssue[] = [];
  const warnings: LoadIssue[] = [];
  let finished = false;
  let loaded: boolean | undefined;
  let pastaLogReadable = false;
  const maxSleeps = Math.ceil(timing.loadTimeoutMs / timing.pollIntervalMs);
  for (let slept = 0; ; slept++) {
    const read = await readNewLines(io, cursor); // 無い間も引き直し、最後の結果で「読めない」を決める（5.7）
    pastaLogReadable = read !== undefined;
    for (const line of read?.lines ?? []) {
      const mark = loadCompletion(line);
      const level = parsePastaLogLine(line).level;
      if (level === 'ERROR' || mark?.loaded === false) {
        errors.push({ source: 'pasta.log', text: line });
      } else if (level === 'WARN') {
        warnings.push({ source: 'pasta.log', text: line });
      }
      if (mark) {
        finished = true;
        loaded = mark.loaded;
        break; // 印より後の行は数えない
      }
    }
    if (read) {
      cursor = read.next;
    }
    if (finished || slept >= maxSleeps) {
      break;
    }
    await ctx.sleep(timing.pollIntervalMs);
  }

  const after = await callTool(io, port, 'get_log', { log_type: 'error', ghost_name: root, since_id: sinceId }, timing.callTimeoutMs);
  if (after.kind === 'ok') {
    for (const e of parseLogEntries(after.text)) {
      (/warn/i.test(e.label) ? warnings : errors).push({ source: 'baseware', text: `[${e.label}] ${e.text}` });
    }
  } else {
    // 読めなければ誤りが無いとは言えないので、誤りに数えて再生を止める
    errors.push({ source: 'baseware', text: `誤りの記録を読めませんでした: ${describeResult(after)}` });
  }
  return { kind: 'done', finished, loaded, errors, warnings, pastaLogReadable, pastaLog };
}

/** 再生してよくない（頼めなかった・時間切れ・loaded=false・誤りが 1 つ以上） */
export function blocksPlayback(outcome: ReloadOutcome): boolean {
  return outcome.kind === 'refused' || !outcome.finished || outcome.loaded === false || outcome.errors.length > 0;
}

const TIMEOUT_TEXT = '読み込みの終わりを確かめられませんでした';

export function formatReload(outcome: ReloadOutcome): { lines: string[]; notice?: string } {
  if (outcome.kind === 'refused') {
    const text = `SHIORI の読み込み直しを頼めませんでした: ${describeResult(outcome.result)}`;
    return { lines: [text], notice: text };
  }
  const { finished, errors, warnings } = outcome;
  const lines: string[] = [];
  if (!finished) {
    lines.push(`${TIMEOUT_TEXT}（pasta.log に読み込みの終わりの行が出ませんでした。pasta.toml の [logging] の level・filter も確かめてください）`);
  }
  if (finished && errors.length === 0 && warnings.length === 0) {
    lines.push('読み込みは誤り無く終わりました');
  } else {
    lines.push(`誤り ${errors.length} 件・警告 ${warnings.length} 件`);
    const list = (head: string, issues: readonly LoadIssue[]) => {
      if (issues.length > 0) {
        lines.push(head, ...issues.map((i) => `  ${i.source === 'pasta.log' ? '[pasta.log]' : '[ベースウェア]'} ${i.text}`));
      }
    };
    list('誤り:', errors);
    list('警告:', warnings);
  }
  if (!outcome.pastaLogReadable) {
    lines.push(`pasta.log を読めなかったため、ベースウェアの誤りの記録だけで要約しました（${outcome.pastaLog.path}）`);
  }
  if (outcome.pastaLog.note) {
    lines.push(outcome.pastaLog.note);
  }
  const notice = !finished ? TIMEOUT_TEXT
    : errors.length > 0 ? `読み込みの誤りが ${errors.length} 件あります`
    : undefined;
  return notice ? { lines, notice } : { lines };
}
