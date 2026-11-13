// MCP client for the SSP / areka endpoint (POST http://127.0.0.1:{port}/api/mcp/v1).
// 拡張とアプリの共有: 外へ出るのは io.post だけ。Wire shapes: .kiro/specs/completed/vscode-mcp-client/research.md.

import type { GhostIO } from './ghostIO.ts';

/** The only destination host. There is no setting to change host or port (requirement 3.9). 入出力の実装（拡張・アプリ）が固定する */
export const HOST = '127.0.0.1';
export const MCP_PATH = '/api/mcp/v1';
export const CALL_TIMEOUT_MS = 15000;
/** 9801, 9821, 9802, 9822, …, 9810, 9830 (same order as areka's port.rs). */
export const CANDIDATE_PORTS: readonly number[] = Array.from({ length: 10 }, (_, i) => [9801 + i, 9821 + i]).flat();
/** Cut-off for one candidate during a scan; all candidates are asked at once (requirement 3.8). */
export const SCAN_TIMEOUT_MS = 2000;

export interface ToolArgsMap {
  get_property: { property_name: 'baseware.name'; ghost_name: string };
  get_log: { log_type: 'error' | 'script'; ghost_name: string; since_id?: number; max_count?: number };
  reload: { target: 'shiori'; ghost_name: string };
  raise_event: {
    event: 'OnPastaSceneAt' | 'OnPastaDumpVariables' | 'OnPastaSetSaveVariable';
    references: readonly string[];
    ghost_name: string;
  };
}
export type ToolName = keyof ToolArgsMap;

export type CallResult =
  | { kind: 'ok'; text: string }                         // isError: false (empty text included)
  | { kind: 'ng'; text: string }                         // isError: true (NG:…)
  | { kind: 'rpcError'; code: number; message: string }  // JSON-RPC error
  | { kind: 'unreachable'; detail: string }              // could not connect before the request was written
  | { kind: 'noResponse'; detail: string }               // dropped after writing, empty body, or timeout
  | { kind: 'badShape'; detail: string };                // non-200, not JSON, or wrong result shape

/** CallResult の失敗を 1 行で */
export function describeResult(r: CallResult): string {
  switch (r.kind) {
    case 'ok': case 'ng': return r.text;
    case 'rpcError': return `JSON-RPC error ${r.code}: ${r.message}`;
    default: return r.detail;
  }
}

let nextId = 1;

/** The parts of a JSON-RPC answer this client reads. */
interface RpcMessage {
  error?: { code?: unknown; message?: unknown };
  result?: { content?: { text?: unknown }[]; isError?: unknown };
}
type RpcOutcome = CallResult | { kind: 'result'; result: NonNullable<RpcMessage['result']> };

/** Turns `\x` escapes JSON does not define into `\\x` (SSP 2.9.05–2.9.07 get_log(script) leaves `\` raw). */
const fixEscapes = (json: string) =>
  json.replace(/\\([\s\S])/g, (m, c: string) => ('"\\/bfnrtu'.includes(c) ? m : '\\\\' + c));

function parseLenient(body: string): unknown {
  try {
    return JSON.parse(body);
  } catch {
    return JSON.parse(fixEscapes(body)); // throws again when still broken
  }
}

/** One JSON-RPC request. Never throws; never retries (a replay would play a script twice). */
async function rpc(io: GhostIO, port: number, method: string, params: unknown, timeoutMs: number): Promise<RpcOutcome> {
  const res = await io.post(port, JSON.stringify({ jsonrpc: '2.0', id: nextId++, method, params }), timeoutMs);
  if (res.kind !== 'response') return res;
  const text = res.body;
  if (res.status !== 200) return { kind: 'badShape', detail: `HTTP ${res.status}: ${text}` };
  if (text.trim() === '') return { kind: 'noResponse', detail: 'empty response body' };
  let msg: RpcMessage | null;
  try { msg = parseLenient(text) as RpcMessage | null; } catch (e) { return { kind: 'badShape', detail: `invalid JSON: ${(e as Error).message}` }; }
  if (msg?.error && typeof msg.error.code === 'number') {
    return { kind: 'rpcError', code: msg.error.code, message: String(msg.error.message ?? '') };
  }
  if (msg?.result == null || typeof msg.result !== 'object') return { kind: 'badShape', detail: `no result: ${text}` };
  return { kind: 'result', result: msg.result };
}

export async function callTool<N extends ToolName>(
  io: GhostIO, port: number, name: N, args: ToolArgsMap[N], timeoutMs = CALL_TIMEOUT_MS,
): Promise<CallResult> {
  const r = await rpc(io, port, 'tools/call', { name, arguments: args }, timeoutMs);
  if (r.kind !== 'result') return r;
  const text = r.result.content?.[0]?.text;
  if (typeof text !== 'string') return { kind: 'badShape', detail: `unexpected result: ${JSON.stringify(r.result)}` };
  return r.result.isError === true ? { kind: 'ng', text } : { kind: 'ok', text };
}

export interface Endpoint {
  readonly port: number;
  readonly basewareName: string; // raw get_property(baseware.name) value
}

const askName = async (io: GhostIO, port: number, root: string, timeoutMs: number): Promise<Endpoint | undefined> => {
  const r = await callTool(io, port, 'get_property', { property_name: 'baseware.name', ghost_name: root }, timeoutMs);
  return r.kind === 'ok' && r.text !== '' ? { port, basewareName: r.text } : undefined;
};

/** initialize, then get_property(baseware.name, ghost_name=root). undefined when the ghost is not running there. */
export async function probe(io: GhostIO, port: number, root: string, timeoutMs = SCAN_TIMEOUT_MS): Promise<Endpoint | undefined> {
  const deadline = Date.now() + timeoutMs; // both round trips share one cut-off
  const init = await rpc(io, port, 'initialize', {
    protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: io.clientName, version: '1' },
  }, timeoutMs);
  if (init.kind !== 'result') return undefined;
  return askName(io, port, root, Math.max(1, deadline - Date.now()));
}

/** Re-checks a remembered endpoint with a single get_property (no initialize). */
export function identify(io: GhostIO, port: number, root: string, timeoutMs = SCAN_TIMEOUT_MS): Promise<Endpoint | undefined> {
  return askName(io, port, root, timeoutMs);
}

export interface LogEntry { id: number; label: string; name: string; text: string }

const LOG_HEAD = /^#(\d+) \S+ \S+ \[(.*?)\] (.*?) : ([\s\S]*)$/;

/** get_log body (`#id time [label] name : text`, continuation lines start with a tab). (no log entries) is []. */
export function parseLogEntries(text: string): LogEntry[] {
  const entries: LogEntry[] = [];
  for (const line of text.replace(/\r?\n$/, '').split(/\r?\n/)) {
    const m = LOG_HEAD.exec(line);
    if (m) entries.push({ id: Number(m[1]), label: m[2], name: m[3], text: m[4] });
    else if (entries.length > 0) entries[entries.length - 1].text += '\n' + line.replace(/^\t/, '');
  }
  return entries;
}

/** Newest id via get_log(max_count=1); 0 when there are no records. */
export async function lastLogId(io: GhostIO, port: number, logType: 'error' | 'script', root: string, timeoutMs = CALL_TIMEOUT_MS): Promise<number | CallResult> {
  const r = await callTool(io, port, 'get_log', { log_type: logType, ghost_name: root, max_count: 1 }, timeoutMs);
  if (r.kind !== 'ok') return r;
  const last = parseLogEntries(r.text).pop();
  if (last) return last.id;
  return r.text.trim() === '(no log entries)' ? 0 : { kind: 'badShape', detail: `unexpected get_log text: ${r.text}` };
}

export type RaiseEventReply =
  | { kind: 'script'; script: string }  // after the `Script: ` line head
  | { kind: 'noScript' }                // `OK:the ghost returned no script.`
  | { kind: 'noResponse'; detail: string }
  | { kind: 'failed'; result: CallResult };

/** Relay (mcp.exe) and areka's own time limit report no response as text. */
const NO_RESPONSE = /No response from server|did not respond within/;

export function readRaiseEvent(result: CallResult): RaiseEventReply {
  if (result.kind === 'noResponse') return { kind: 'noResponse', detail: result.detail };
  if (result.kind !== 'ok' && result.kind !== 'ng') return { kind: 'failed', result };
  if (NO_RESPONSE.test(result.text)) return { kind: 'noResponse', detail: result.text };
  if (result.kind === 'ng') return { kind: 'failed', result };
  const m = /(?:^|\n)Script: ([\s\S]*)$/.exec(result.text);
  if (m) return { kind: 'script', script: m[1].replace(/\r?\n$/, '') };
  if (result.text.startsWith('OK:the ghost returned no script')) return { kind: 'noScript' };
  return { kind: 'failed', result };
}

/** Probes all ports at once and returns the responders in `ports` order, one per baseware name. */
export async function findBasewares(
  io: GhostIO, root: string, ports: readonly number[] = CANDIDATE_PORTS, timeoutMs = SCAN_TIMEOUT_MS,
): Promise<Endpoint[]> {
  const found = await Promise.all(ports.map((p) => probe(io, p, root, timeoutMs)));
  const seen = new Set<string>();
  // shortcut: 同じ名前の別のプロセス（SSP 2 つ）が同じゴーストを動かすと最初の 1 つに寄る。プロセスを見分ける property が SSP に出たら替える
  // (SSP 2.9.28 answers on both 9801 and 9821 from one process, so without this the same SSP is listed twice.)
  return found.filter((e): e is Endpoint => e !== undefined && !seen.has(e.basewareName) && !!seen.add(e.basewareName));
}
