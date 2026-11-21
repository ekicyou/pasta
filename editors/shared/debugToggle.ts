// DebugToggle - ＄＊デバッグ を書き、SHIORI を読み込み直し、読み込みの後の有効・無効と要約を返す
//
// vscode にも Tauri にも触らない。VSCode 拡張と pasta-recipe が同じ流れと文を使う。

import { type SaveVariableChange, type SetOutcome, formatSet, setSaveVariable } from './dumpVariables.ts';
import { type FlowContext, type ReloadOutcome, formatReload, reloadAndWait } from './reloadShiori.ts';
import type { Report } from './devSession.ts';

export const DEBUG_SAVE_NAME = 'デバッグ';
export const ENABLE_DEBUG = 'SHIORI デバッグを有効にする';
export const DISABLE_DEBUG = 'SHIORI デバッグを無効にする';

export type DebugState = 'enabled' | 'disabled' | 'unconfirmed';

/** refused・時間切れ・loaded が true でない・行が無い は unconfirmed */
export function debugStateOf(outcome: ReloadOutcome): DebugState {
  if (outcome.kind !== 'done' || !outcome.finished || outcome.loaded !== true || outcome.debug === undefined) {
    return 'unconfirmed';
  }
  return outcome.debug ? 'enabled' : 'disabled';
}

export type ToggleOutcome =
  | { kind: 'notWritten'; set: Exclude<SetOutcome, { kind: 'set' }> }
  | { kind: 'reloaded'; reload: ReloadOutcome; state: DebugState };

const changeOf = (enable: boolean): SaveVariableChange => ({ type: 'boolean', value: String(enable) });

/** ＄＊デバッグ に boolean の 'true'／'false' を書き、set のときだけリロードする。＄＊debug は触らない */
export async function toggleDebug(ctx: FlowContext, enable: boolean): Promise<ToggleOutcome> {
  const set = await setSaveVariable(ctx, DEBUG_SAVE_NAME, changeOf(enable));
  if (set.kind !== 'set') {
    return { kind: 'notWritten', set };
  }
  const reload = await reloadAndWait(ctx);
  return { kind: 'reloaded', reload, state: debugStateOf(reload) };
}

const STATE_LINE = { enabled: 'デバッグは有効です', disabled: 'デバッグは無効です' } as const;
const UNCONFIRMED = 'デバッグの有効・無効を確かめられませんでした';
const NO_LINE = `${UNCONFIRMED}（pasta.log に有効・無効の行がありませんでした）`;
const mismatchText = (enable: boolean) =>
  `デバッグを${enable ? '有効' : '無効'}にできませんでした。環境変数 PASTA_DEBUG が設定されていると、保存変数より先に効きます。ほかの理由はマニュアルの「うまく動かないとき」を見てください`;

export function formatToggle(enable: boolean, outcome: ToggleOutcome): Report {
  const change = changeOf(enable);
  if (outcome.kind === 'notWritten') {
    return formatSet(DEBUG_SAVE_NAME, change, outcome.set);
  }
  const { reload, state } = outcome;
  const summary = formatReload(reload);
  const lines = [...formatSet(DEBUG_SAVE_NAME, change, { kind: 'set' }).lines, ...summary.lines];
  if (state === 'unconfirmed') {
    const loadedWithoutLine = reload.kind === 'done' && reload.finished && reload.loaded === true;
    const text = loadedWithoutLine ? NO_LINE : UNCONFIRMED;
    lines.push(text);
    const report: Report = { lines, notice: summary.notice ?? text };
    return reload.kind === 'refused' ? { ...report, failure: reload.result } : report;
  }
  lines.push(STATE_LINE[state]);
  if ((state === 'enabled') !== enable) {
    const text = mismatchText(enable);
    lines.push(text);
    return { lines, notice: text };
  }
  return summary.notice ? { lines, notice: summary.notice } : { lines };
}
